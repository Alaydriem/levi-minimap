use std::sync::Arc;
use std::time::{Duration, Instant};

use minimap_core::config::{Config, ConfigLoader, WriteBack};
use minimap_core::errors::WaypointError;
use minimap_core::services::cave::LayerSelector;
use minimap_core::services::input::InputController;
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::input::{KeyInput, KeyResult, MouseAction, MouseInput};
use minimap_core::structs::pos::ChunkPos;
use minimap_store::services::{CachePaths, StoreService};
use minimap_store::structs::{StoreCommand, StoreEvent, StoreSettings, WorldIdentity};

use crate::errors::RuntimeError;
use crate::services::{ExportService, WaypointSync, WorldSession};
use crate::structs::{
    CursorAction, ProbeRequest, RuntimePaths, RuntimeStats, ScanPlan, Shared, SubmitStatus, TickInput,
};

// everything the game thread calls: world lifecycle, ticks, probes, keys, config saves
pub struct GameSide {
    shared: Arc<Shared>,
    config: Config,
    paths: RuntimePaths,
    input: InputController,
    session: Option<WorldSession>,
    config_store: StoreService,
    config_changed_at: Option<Instant>,
    waypoints: WaypointSync,
    store_events: Vec<StoreEvent>,
    exports: Arc<ExportService>,
    layers: LayerSelector,
}

impl GameSide {
    pub const SAVE_DELAY: Duration = Duration::from_secs(2);
    const SHUTDOWN: Duration = Duration::from_secs(5);
    const MAX_STORE_EVENTS: usize = 16;

    pub fn new(
        shared: Arc<Shared>,
        config: Config,
        paths: RuntimePaths,
        write_back: WriteBack,
        exports: Arc<ExportService>,
    ) -> Result<Self, RuntimeError> {
        let config_store = StoreService::start(None, StoreSettings::new(&config.store))?;
        let game = Self {
            input: InputController::new(&config),
            waypoints: WaypointSync::new(Arc::clone(&shared)),
            store_events: Vec::with_capacity(Self::MAX_STORE_EVENTS),
            exports,
            layers: LayerSelector::new(&config),
            shared,
            config,
            paths,
            session: None,
            config_store,
            config_changed_at: None,
        };
        match write_back {
            WriteBack::None => {}
            WriteBack::Create => game.save_config(false)?,
            WriteBack::ReplaceInvalid => game.save_config(true)?,
        }
        Ok(game)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn waypoints_loaded(&self) -> bool {
        self.waypoints.is_loaded()
    }

    pub fn enter_world(&mut self, identity: &WorldIdentity, dim: i32) -> Result<(), RuntimeError> {
        self.leave_world();
        if self.config.waypoints.enabled {
            self.waypoints.open(CachePaths::waypoints_file(identity, &self.paths.data_dir), &self.config_store);
        }
        let root = self.config.terrain.enable_disk_cache.then(|| CachePaths::root(identity, &self.paths.data_dir));
        self.exports.enter_world(&identity.label());
        self.layers.reset();
        self.shared.layer.store(dim, std::sync::atomic::Ordering::Relaxed);
        let exports = Arc::clone(&self.exports);
        self.session = Some(WorldSession::start(Arc::clone(&self.shared), &self.config, root, dim, exports)?);
        Ok(())
    }

    pub fn leave_world(&mut self) {
        if let Some(session) = self.session.take() {
            session.finish();
        }
        self.waypoints.close(&self.config_store);
        self.exports.leave_world();
        Shared::lock(&self.shared.ui).reset();
        if let Ok(mut chunks) = self.shared.chunks.write() {
            chunks.clear();
        }
        Shared::lock(&self.shared.tiles).clear();
        Shared::lock(&self.shared.known_empty).clear();
        Shared::lock(&self.shared.biome_keys).clear();
        Shared::lock(&self.shared.hover_biomes).clear();
        Shared::lock(&self.shared.player).present = false;
        let mut view = Shared::lock(&self.shared.view);
        view.world_map_open = false;
        view.anchor = None;
    }

    // the overlay switches of the tools block are saved like the coordinates switch
    fn sync_view_config(&mut self) {
        let (grid, slime) = {
            let view = Shared::lock(&self.shared.view);
            (view.show_chunk_grid, view.show_slime_chunks)
        };
        let world_map = &mut self.config.world_map;
        if world_map.show_chunk_grid != grid || world_map.show_slime_chunks != slime {
            world_map.show_chunk_grid = grid;
            world_map.show_slime_chunks = slime;
            self.config_changed_at = Some(Instant::now());
        }
    }

    pub fn tick(&mut self, input: &TickInput, plan: &mut ScanPlan) {
        *Shared::lock(&self.shared.player) = input.player;
        plan.candidates.clear();
        plan.cursor = self.cursor_action(input);
        self.sync_view_config();
        if self.config_changed_at.is_some_and(|at| at.elapsed() >= Self::SAVE_DELAY) {
            self.config_changed_at = None;
            if let Err(error) = self.save_config(false) {
                log::warn!("config not saved: {error}");
            }
        }
        self.drain_store_events();
        self.waypoints.tick(input, &self.config_store);
        let Some(session) = self.session.as_mut() else { return };
        if !input.player.present {
            return;
        }
        let layer = self.layers.select(input.player.dim, input.player.block().1);
        if session.dim() != layer.id {
            session.change_layer(&layer);
            self.shared.layer.store(layer.id, std::sync::atomic::Ordering::Relaxed);
        }
        if self.shared.area_export.swap(false, std::sync::atomic::Ordering::SeqCst) {
            session.request_area_export();
        }
        if self.config.terrain.enable {
            let (bx, _, bz) = input.player.block();
            session.tick(ChunkPos::from_block(bx, bz, layer.id), plan);
        }
    }

    pub fn submit_chunk(&mut self, pos: ChunkPos, status: SubmitStatus, probe: &ChunkProbe) {
        if let Some(session) = self.session.as_mut() {
            session.submit_chunk(pos, status, probe);
        }
    }

    pub fn biome_name(&mut self, id: u16, name: &str) {
        if let Some(session) = self.session.as_mut() {
            session.biome_name(id, name);
        }
    }

    pub fn block_changed(&self, x: i32, z: i32, dim: i32) -> ProbeRequest {
        self.session.as_ref().map_or(ProbeRequest::None, |s| s.block_changed(x, z, dim))
    }

    pub fn submit_column(&mut self, pos: ChunkPos, index: usize, sample: ColumnSample) {
        if let Some(session) = self.session.as_mut() {
            session.submit_column(pos, index, sample);
        }
    }

    pub fn key(&mut self, input: KeyInput) -> KeyResult {
        let map_open = Shared::lock(&self.shared.view).world_map_open;
        if (map_open || input.mouse_grabbed) && self.ui_key(input) {
            return KeyResult { consume: true, ..KeyResult::default() };
        }
        let (result, radius, coordinates) = {
            let mut view = Shared::lock(&self.shared.view);
            (self.input.on_key(&mut view, input), view.minimap_radius, view.show_coordinates)
        };
        if result.coordinates_changed {
            self.config.mini_map.show_coordinates = coordinates;
            self.config.world_map.show_coordinates = coordinates;
            self.config_changed_at = Some(Instant::now());
            let state = if coordinates { "shown" } else { "hidden" };
            Shared::lock(&self.shared.ui).show_done(format!("Coordinates {state}"));
        }
        if result.radius_changed {
            self.config.mini_map.radius_chunks = radius;
            self.config_changed_at = Some(Instant::now());
        }
        if result.add_waypoint
            && let Err(WaypointError::Full(cap)) = self.waypoints.add_at_player()
        {
            Shared::lock(&self.shared.ui).show_message(format!("This world already has {cap} waypoints"));
        }
        if result.toggle_cave {
            let state = if self.layers.toggle() { "on" } else { "off" };
            Shared::lock(&self.shared.ui).show_done(format!("Cave view {state}"));
        }
        if result.action == MouseAction::GrabMouse {
            Shared::lock(&self.shared.ui).close();
        }
        result
    }

    // the menu, the dialog and the undo see keys before the maps do
    fn ui_key(&self, input: KeyInput) -> bool {
        let mut ui = Shared::lock(&self.shared.ui);
        let mut store = Shared::lock(&self.shared.waypoints);
        ui.key(input, &mut store)
    }

    pub fn mouse(&mut self, input: MouseInput) -> bool {
        if Shared::lock(&self.shared.ui).wheel(input) {
            return true;
        }
        let mut view = Shared::lock(&self.shared.view);
        self.input.on_mouse(&mut view, input)
    }

    pub fn stats(&self) -> RuntimeStats {
        self.session.as_ref().map_or_else(RuntimeStats::default, WorldSession::stats)
    }

    pub fn shutdown(&mut self) {
        self.leave_world();
        if self.config_changed_at.take().is_some()
            && let Err(error) = self.save_config(false)
        {
            log::warn!("config not saved: {error}");
        }
        self.config_store.shutdown(Self::SHUTDOWN);
    }

    // answers from the game side's store: waypoint files; config, telemetry and waypoint write failures
    fn drain_store_events(&mut self) {
        self.config_store.drain_events(&mut self.store_events, Self::MAX_STORE_EVENTS);
        for event in self.store_events.drain(..) {
            match event {
                StoreEvent::WaypointsLoaded { path, waypoints } => self.waypoints.on_loaded(&path, waypoints),
                StoreEvent::Failed(message) => log::warn!("store: {message}"),
                _ => {}
            }
        }
    }

    // while the game holds the mouse, the cursor is kept at the client centre
    fn cursor_action(&self, input: &TickInput) -> CursorAction {
        let map_open = Shared::lock(&self.shared.view).world_map_open;
        match (map_open, input.foreground, input.cursor_hidden) {
            (true, _, _) | (false, false, _) => CursorAction::None,
            (false, true, true) => CursorAction::ClipToCentre,
            (false, true, false) => CursorAction::Release,
        }
    }

    // telemetry lines go through the store thread too; a full queue drops the line
    pub fn append_line(&self, path: &std::path::Path, line: String, truncate: bool) {
        let command = StoreCommand::AppendLine { path: path.to_path_buf(), line, truncate };
        if self.config_store.try_send(command).is_err() {
            log::warn!("telemetry line dropped: the store queue is full");
        }
    }

    fn save_config(&self, keep_invalid_copy: bool) -> Result<(), RuntimeError> {
        let json = ConfigLoader::to_json(&self.config)?;
        let command = StoreCommand::WriteConfig { path: self.paths.config_file.clone(), json, keep_invalid_copy };
        if self.config_store.try_send(command).is_err() {
            log::warn!("config save skipped: the store queue is full");
        }
        Ok(())
    }
}
