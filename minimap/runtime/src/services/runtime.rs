use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use minimap_core::config::{Config, ConfigLoader};
use minimap_core::services::biome::{BiomeColors, HoverBiomeCache};
use minimap_core::services::export::ViewImage;
use minimap_core::services::memory::{ChunkStore, KnownEmptySet, TileStore};
use minimap_core::services::ui::MapUi;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::Rgba;
use minimap_core::structs::bake::BakeSettings;
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::input::{KeyInput, KeyResult, MouseInput};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::ViewState;
use minimap_core::structs::waypoint::Waypoint;
use minimap_store::services::ConfigFile;
use minimap_store::structs::WorldIdentity;

use crate::errors::RuntimeError;
use crate::services::{CaptureSink, CaptureWriter, ExportService, GameSide, RenderSide, TelemetryRecorder};
use crate::structs::{
    CaptureEvent, CaptureHeader, FrameInput, FrameOutput, ProbeRequest, RuntimePaths, RuntimeStats, ScanPlan, Shared,
    SubmitStatus, TelemetryRow, TickInput,
};

// the whole mod behind one handle; game-thread calls and render-thread calls take separate locks
pub struct Runtime {
    shared: Arc<Shared>,
    game: Mutex<GameSide>,
    render: Mutex<RenderSide>,
    exports: Arc<ExportService>,
    biomes: BiomeColors,
    // block name and packed colour, in name order, for the probe
    block_colors: Vec<(String, u32)>,
    capture: Mutex<Option<CaptureWriter>>,
    sink: Option<CaptureSink>,
    telemetry: Option<Mutex<TelemetryRecorder>>,
    telemetry_path: PathBuf,
}

impl Runtime {
    const MIN_KNOWN_EMPTY: usize = 16_384;
    const MIB: usize = 1024 * 1024;
    pub const TELEMETRY_FILE: &'static str = "telemetry.csv";
    pub const CAPTURE_DIR: &'static str = "captures";

    pub fn new(paths: RuntimePaths) -> Result<Self, RuntimeError> {
        let text = ConfigFile::read(&paths.config_file).unwrap_or_else(|error| {
            log::warn!("config.json not read: {error}");
            None
        });
        let loaded = ConfigLoader::load(text.as_deref());
        for warning in &loaded.warnings {
            log::warn!("config {}: {}", warning.key, warning.message);
        }
        let config = loaded.config.clone();
        let biomes = BiomeColors::from_overrides(&config.terrain.biome_color_overrides);
        let settings = BakeSettings::new(&config.terrain);
        let memory = &config.memory;
        let chunk_capacity =
            ChunkStore::capacity_for_budget(memory.chunk_surface_budget_mb as usize * Self::MIB, settings.scale);
        let tile_capacity = TileStore::capacity_for_budget(memory.tile_budget_mb as usize * Self::MIB);
        let shared = Arc::new(Shared::new(
            ChunkStore::new(chunk_capacity),
            TileStore::new(tile_capacity, memory.tile_overrun_percent.max(0) as usize),
            KnownEmptySet::new((tile_capacity * 4).max(Self::MIN_KNOWN_EMPTY)),
            settings,
            ViewState {
                show_coordinates: config.mini_map.show_coordinates || config.world_map.show_coordinates,
                show_chunk_grid: config.world_map.show_chunk_grid,
                show_slime_chunks: config.world_map.show_slime_chunks,
                ..ViewState::new(config.mini_map.radius_chunks)
            },
            WaypointStore::new(config.waypoints.max_per_world.max(1) as usize, Self::seed()),
            MapUi::new(config.mini_map.widget_scale, config.world_map.ui_event_capacity.max(1) as usize),
        ));
        *Shared::lock(&shared.hover_biomes) =
            HoverBiomeCache::new(config.world_map.hover_biome_regions.max(1) as usize);
        let block_colors = config
            .terrain
            .block_colors
            .iter()
            .map(|(name, [r, g, b, a])| (name.clone(), Rgba::new(*r, *g, *b, *a).packed()))
            .collect();
        let capture = Self::start_capture(&config, &paths);
        let telemetry = config.debug.telemetry.then(|| {
            let interval = Duration::from_millis(config.debug.telemetry_interval_ms.max(0) as u64);
            Mutex::new(TelemetryRecorder::new(interval, Instant::now()))
        });
        let telemetry_path = paths.data_dir.join(Self::TELEMETRY_FILE);
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
        let export_dir = ExportService::folder(&config.export.folder, home.as_deref(), &paths.data_dir);
        let exports = Arc::new(ExportService::start(&export_dir, Arc::clone(&shared))?);
        let render = RenderSide::new(Arc::clone(&shared), &config, Arc::clone(&exports));
        let game = GameSide::new(Arc::clone(&shared), config, paths, loaded.write_back, Arc::clone(&exports))?;
        if telemetry.is_some() {
            game.append_line(&telemetry_path, TelemetryRow::HEADER.to_string(), true);
        }
        Ok(Self {
            shared,
            game: Mutex::new(game),
            render: Mutex::new(render),
            exports,
            biomes,
            block_colors,
            sink: capture.as_ref().map(CaptureWriter::sink),
            capture: Mutex::new(capture),
            telemetry,
            telemetry_path,
        })
    }

    pub fn enter_world(&self, identity: &WorldIdentity, dim: i32) -> Result<(), RuntimeError> {
        self.record(|| CaptureEvent::Enter { dim });
        Shared::lock(&self.game).enter_world(identity, dim)
    }

    pub fn leave_world(&self) {
        self.record(|| CaptureEvent::Leave);
        Shared::lock(&self.game).leave_world();
    }

    pub fn tick(&self, input: &TickInput, plan: &mut ScanPlan) {
        self.record(|| CaptureEvent::Tick(*input));
        let started = Instant::now();
        // a row is written before this tick runs, so it closes the previous tick with its probing
        if let Some(telemetry) = &self.telemetry
            && Shared::lock(telemetry).due(started)
        {
            let stats = self.stats();
            let row = Shared::lock(telemetry).take_row(started, &stats, self.shared.bake_timing.totals());
            Shared::lock(&self.game).append_line(&self.telemetry_path, row.to_csv(), false);
        }
        Shared::lock(&self.game).tick(input, plan);
        if let Some(telemetry) = &self.telemetry {
            Shared::lock(telemetry).on_tick(started, Instant::now());
        }
    }

    pub fn submit_chunk(&self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe) {
        self.record(|| CaptureEvent::Submit { pos, status, probe: Box::new(*probe) });
        if status == SubmitStatus::Loaded
            && let Some(telemetry) = &self.telemetry
        {
            Shared::lock(telemetry).on_probe(Instant::now());
        }
        Shared::lock(&self.game).submit_chunk(pos, status, probe);
    }

    pub fn block_changed(&self, x: i32, z: i32, dim: i32) -> ProbeRequest {
        self.record(|| CaptureEvent::Block { x, z, dim });
        Shared::lock(&self.game).block_changed(x, z, dim)
    }

    pub fn submit_column(&self, pos: ChunkPos, index: usize, sample: ColumnSample) {
        self.record(|| CaptureEvent::Column { pos, index: index as u32, sample });
        Shared::lock(&self.game).submit_column(pos, index, sample);
    }

    // the game's internal name for a biome ID the plan asked about; empty when it has none
    pub fn biome_name(&self, id: u16, name: &str) {
        self.record(|| CaptureEvent::BiomeName { id, name: name.to_string() });
        Shared::lock(&self.game).biome_name(id, name);
    }

    // the internal biome name of a resident column on the shown layer
    pub fn column_biome(&self, block_x: i32, block_z: i32) -> Option<String> {
        let layer = self.shared.layer.load(std::sync::atomic::Ordering::Relaxed);
        let key = self.shared.chunks.read().ok()?.column_biome(block_x, block_z, layer)?;
        Shared::lock(&self.shared.biome_keys).name(key).map(str::to_string)
    }

    // colour of a column with no blocks: the biome's terrain.biomeColorOverrides entry, else the game's
    pub fn resolve_void_color(&self, biome: &str, fallback: u32) -> u32 {
        self.biomes.lookup(biome).map_or(fallback, |c| c.packed())
    }

    // colours the probe draws in place of the game's map colour, by block name
    pub fn block_colors(&self) -> &[(String, u32)] {
        &self.block_colors
    }

    pub fn key(&self, input: KeyInput) -> KeyResult {
        self.record(|| CaptureEvent::Key(input));
        Shared::lock(&self.game).key(input)
    }

    pub fn mouse(&self, input: MouseInput) -> bool {
        self.record(|| CaptureEvent::Mouse(input));
        Shared::lock(&self.game).mouse(input)
    }

    // one UTF-16 unit typed while a dialog has the keyboard
    pub fn text(&self, unit: u32) {
        Shared::lock(&self.shared.ui).text(unit);
    }

    // the back buffer of a CAPTURE_VIEW frame, as the shim mapped it; format is its DXGI format
    pub fn view_captured(&self, pixels: &[u8], size: (u32, u32), stride: u32, format: u32) {
        let (width, height) = size;
        let dim = self.shared.layer.load(std::sync::atomic::Ordering::Relaxed);
        let exported = ViewImage::from_capture(pixels, width as usize, height as usize, stride as usize, format)
            .map_err(|error| format!("Export failed: {error}"))
            .and_then(|rgba| self.exports.export_view(rgba, width, height, dim));
        if let Err(message) = exported {
            Shared::lock(&self.shared.ui).show_message(message);
        }
    }

    pub fn frame(&self, input: &FrameInput, out: &mut FrameOutput) {
        self.record(|| CaptureEvent::Frame(*input));
        Shared::lock(&self.render).frame(input, out);
        if let Some(telemetry) = &self.telemetry {
            Shared::lock(telemetry).on_frame(input, out.ui_build_us);
        }
    }

    // a copy of the current world's waypoints, every dimension
    pub fn waypoints(&self) -> Vec<Waypoint> {
        Shared::lock(&self.shared.waypoints).waypoints().to_vec()
    }

    // the current world's waypoints file has been read, so changes are being saved
    pub fn waypoints_loaded(&self) -> bool {
        Shared::lock(&self.game).waypoints_loaded()
    }

    pub fn stats(&self) -> RuntimeStats {
        let base = Shared::lock(&self.game).stats();
        let (tiles_resident, tile_capacity, tile_overrun) = {
            let tiles = Shared::lock(&self.shared.tiles);
            (tiles.len(), tiles.capacity(), tiles.overrun())
        };
        let (surfaces_resident, surface_capacity) =
            self.shared.chunks.try_read().map_or((0, 0), |c| (c.len(), c.capacity()));
        let render = Shared::lock(&self.render);
        RuntimeStats {
            frames: render.frames(),
            gpu_textures: render.gpu_textures(),
            surfaces_resident,
            surface_capacity,
            tiles_resident,
            tile_capacity,
            tile_overrun,
            ..base
        }
    }

    pub fn shutdown(&self) {
        Shared::lock(&self.game).shutdown();
        self.exports.shutdown();
        if let Some(writer) = Shared::lock(&self.capture).take() {
            let report = writer.finish();
            log::info!("capture closed: {} events written, {} dropped", report.written, report.dropped);
        }
    }

    // waypoint ids start from the clock, so two sessions do not make the same ids
    fn seed() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64)
    }

    fn start_capture(config: &Config, paths: &RuntimePaths) -> Option<CaptureWriter> {
        if !config.debug.capture_probes {
            return None;
        }
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let path = paths.data_dir.join(Self::CAPTURE_DIR).join(format!("capture-{secs}.lmcap"));
        match CaptureWriter::start(&path, &CaptureHeader::new(&config.terrain)) {
            Ok(writer) => {
                log::info!("recording a capture to {}", path.display());
                Some(writer)
            }
            Err(error) => {
                log::warn!("capture not started: {error}");
                None
            }
        }
    }

    // builds the event only when a capture is being recorded
    fn record(&self, event: impl FnOnce() -> CaptureEvent) {
        if let Some(sink) = &self.sink {
            sink.record(event());
        }
    }
}
