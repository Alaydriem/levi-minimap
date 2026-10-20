// Criterion benches for every hot path. Each bench only drives struct methods.
use std::hint::black_box;
use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use minimap_bench::services::{BenchInputs, BenchTerrain, OverlayScenes};
use minimap_bench::structs::BenchInput;
use minimap_core::config::{Config, MiniMapConfig, TerrainConfig};
use minimap_core::services::bake::{ChunkBaker, RegionBaker};
use minimap_core::services::biome::{BiomeKeyTable, BiomeResolver};
use minimap_core::services::cave::{CaveSurface, LayerSelector};
use minimap_core::services::draw::{
    ChunkGridPainter, MinimapComposer, MinimapPainter, SlimeOverlayPainter, WaypointPainter, WorldMapPainter,
};
use minimap_core::services::export::{AreaLayout, AreaRow};
use minimap_core::services::memory::{ChunkStore, TileStore};
use minimap_core::services::slime::SlimeChunks;
use minimap_core::services::ui::MapUi;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::bake::{BakeOutput, BakeSettings, RegionBakeOutput};
use minimap_core::structs::chunk::ColumnSample;
use minimap_core::structs::draw::{DrawList, ExploredChunks, MinimapFrame, WorldMapFrame};
use minimap_core::structs::mesh::UiMesh;
use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_core::structs::ui::{HintKeys, HoverInfo, MapAction, MenuTarget, UiFrame, UiFrameInput};
use minimap_core::structs::view::CameraPose;
use minimap_core::structs::view::ViewState;
use minimap_core::structs::waypoint::WaypointSource;
use minimap_store::services::{Journal, RegionFiles};
use minimap_store::structs::JournalRecord;
use minimap_ui::runtime::UiRuntime;
use minimap_ui::theme::FontSource;

struct MinimapBenches;

impl MinimapBenches {
    fn settings(input: &BenchInput, scale: i32, pcf: i32) -> BakeSettings {
        let mut terrain = input.terrain.clone();
        terrain.shadow.render_scale = scale;
        terrain.shadow.pcf_radius = pcf;
        BakeSettings::new(&terrain)
    }

    // the input's chunk closest to its centre that has all 8 neighbours
    fn centre(input: &BenchInput) -> ChunkPos {
        let n = input.chunks.len().max(1) as i64;
        let (sx, sz) =
            input.chunks.iter().fold((0i64, 0i64), |(x, z), (p, _)| (x + i64::from(p.x), z + i64::from(p.z)));
        let dim = input.chunks.first().map_or(0, |(p, _)| p.dim);
        let target = ChunkPos::new((sx / n) as i32, (sz / n) as i32, dim);
        input
            .chunks
            .iter()
            .map(|(p, _)| *p)
            .min_by_key(|p| (p.x - target.x).abs() + (p.z - target.z).abs())
            .unwrap_or(target)
    }

    fn probe_submit(c: &mut Criterion) {
        let mut group = c.benchmark_group("probe_submit");
        let pos = ChunkPos::new(0, 0, 0);
        let probe = BenchTerrain::probe(pos);
        let mut changed = probe;
        changed.write_column(17, ColumnSample::new(0xFF00_00FF, 0, 120, 120, false, 0));
        group.bench_function("new_chunk", |b| {
            let mut evicted = Vec::new();
            b.iter(|| {
                let mut store = ChunkStore::new(4);
                black_box(store.submit(pos, &probe, 1, pos, &mut evicted));
            });
        });
        let mut store = ChunkStore::new(4);
        let mut evicted = Vec::new();
        store.submit(pos, &probe, 1, pos, &mut evicted);
        group.bench_function("unchanged", |b| b.iter(|| black_box(store.submit(pos, &probe, 2, pos, &mut evicted))));
        group.bench_function("one_column_changed", |b| {
            let mut flip = false;
            b.iter(|| {
                flip = !flip;
                let incoming = if flip { &changed } else { &probe };
                black_box(store.submit(pos, incoming, 3, pos, &mut evicted));
            });
        });
        let with_biomes = BenchTerrain::probe_with_biomes(pos);
        let mut keys = BiomeKeyTable::new(BiomeKeyTable::CAPACITY);
        keys.mark_loaded(1, Vec::new());
        let mut resolver = BiomeResolver::new();
        resolver.answer(1, "plains");
        resolver.answer(4, "forest");
        group.bench_function("palette_fill", |b| {
            let mut surface = with_biomes.surface;
            b.iter(|| {
                surface.biome = minimap_core::structs::biome::BiomePalette::empty();
                black_box(resolver.fill(&with_biomes.biome_ids, &mut surface, &mut keys));
            });
        });
        let mut named = with_biomes.surface;
        resolver.fill(&with_biomes.biome_ids, &mut named, &mut keys);
        let stored = minimap_core::structs::chunk::ChunkSurface::empty().merge(&named).surface;
        group.bench_function("merge_rescan_same_biomes", |b| b.iter(|| black_box(stored.merge(&named))));
        group.bench_function("palette_build", |b| {
            let mut evicted = Vec::new();
            b.iter(|| {
                let mut surface = with_biomes.surface;
                resolver.fill(&with_biomes.biome_ids, &mut surface, &mut keys);
                let mut store = ChunkStore::new(4);
                black_box(store.submit(pos, &surface, 1, pos, &mut evicted));
            });
        });
        group.finish();
    }

    // the world map tools on a 1920 x 1080 frame: the slime test, both overlays at the gate and at
    // the closest zoom (the slime cache warm), and the waypoint list with 512 waypoints
    fn world_map_tools(c: &mut Criterion) {
        let mut group = c.benchmark_group("world_map_tools");
        let mut chunk = 0i32;
        group.bench_function("slime_chunk_test", |b| {
            b.iter(|| {
                chunk = chunk.wrapping_add(7919);
                black_box(SlimeChunks::is_slime(chunk, chunk.wrapping_mul(31)))
            });
        });
        let grid = ChunkGridPainter::new(&MiniMapConfig::default());
        let mut draw = DrawList::new();
        for (name, zoom) in [("gate", OverlayScenes::GATE_ZOOM), ("max_zoom", OverlayScenes::MAX_ZOOM)] {
            let frame = OverlayScenes::world_frame(zoom);
            group.bench_function(format!("world_map_grid_{name}"), |b| {
                b.iter(|| {
                    draw.clear();
                    grid.paint(&frame, Some((0, 0)), &ExploredChunks::everywhere(), &mut draw);
                });
            });
            let mut slime = SlimeOverlayPainter::new(4096);
            for _ in 0..16 {
                draw.clear();
                slime.paint(&frame, &ExploredChunks::everywhere(), &mut draw);
            }
            group.bench_function(format!("world_map_slime_{name}"), |b| {
                b.iter(|| {
                    draw.clear();
                    slime.paint(&frame, &ExploredChunks::everywhere(), &mut draw);
                });
            });
        }
        group.finish();
    }

    fn chunk_bake(c: &mut Criterion) {
        let mut group = c.benchmark_group("chunk_bake");
        for input in BenchInputs::all() {
            let store = BenchTerrain::store(&input.chunks);
            let pos = Self::centre(&input);
            let Some((_, surface)) = input.chunks.iter().find(|(p, _)| *p == pos) else { continue };
            for scale in [1, 2, 4, 16] {
                for pcf in [0, 1, 4, 8] {
                    let settings = Self::settings(&input, scale, pcf);
                    let id = BenchmarkId::new(input.name.as_str(), format!("scale{scale}_pcf{pcf}"));
                    let mut baker = ChunkBaker::new();
                    let mut out = BakeOutput::new();
                    group.bench_with_input(id, &settings, |b, settings| {
                        b.iter(|| baker.bake(pos, surface, &store, settings, &mut out));
                    });
                }
            }
        }
        group.finish();
    }

    fn region_bake(c: &mut Criterion) {
        let mut group = c.benchmark_group("region_bake");
        group.sample_size(10);
        for input in BenchInputs::all() {
            let store = BenchTerrain::store(&input.chunks);
            let region = Self::centre(&input).region();
            for (scale, pcf) in [(2, 1), (4, 1)] {
                let settings = Self::settings(&input, scale, pcf);
                let id = BenchmarkId::new(input.name.as_str(), format!("scale{scale}_pcf{pcf}"));
                let mut baker = RegionBaker::new();
                let mut out = RegionBakeOutput::new();
                group.bench_with_input(id, &settings, |b, settings| {
                    b.iter(|| baker.bake(region, &store, settings, &mut out));
                });
            }
        }
        group.finish();
    }

    // tiles for every region of the synthetic input, baked once
    fn tiles(input: &BenchInput) -> TileStore {
        let store = BenchTerrain::store(&input.chunks);
        let settings = BakeSettings::new(&TerrainConfig::default());
        let mut tiles = TileStore::new(64, 25);
        let mut baker = ChunkBaker::new();
        let mut out = BakeOutput::new();
        let mut evicted = Vec::new();
        for (pos, surface) in &input.chunks {
            baker.bake(*pos, surface, &store, &settings, &mut out);
            tiles.write_chunk(*pos, &out.pixels, settings.settings_hash(), &mut evicted);
        }
        tiles
    }

    fn minimap_compose(c: &mut Criterion) {
        let mut group = c.benchmark_group("minimap_compose");
        let input = BenchInputs::synthetic();
        let tiles = Self::tiles(&input);
        for radius in [4, 8, 16] {
            let mut composer = MinimapComposer::new();
            group.bench_function(BenchmarkId::new("radius", radius), |b| {
                b.iter(|| {
                    composer.invalidate();
                    black_box(composer.update((8, 8), 0, radius, &tiles, None::<&ChunkStore>));
                });
            });
        }
        group.finish();
    }

    fn draw_list(c: &mut Criterion) {
        let mut group = c.benchmark_group("draw_list");
        let painter = MinimapPainter::new(&MiniMapConfig::default());
        let camera = CameraPose { x: 8.5, z: 8.5, yaw: 30.0 };
        let minimap = MinimapFrame {
            display_width: 1920.0,
            camera,
            player_block: (8, 70, 8),
            radius_chunks: 8,
            font_size: 18.0,
            texture: Some(1),
            image_origin: (-120, -120),
            image_side: 258,
            dim: 0,
            show_coordinates: true,
            angle: 0.0,
        };
        let mut draw = DrawList::new();
        group.bench_function("minimap", |b| {
            b.iter(|| {
                draw.clear();
                painter.paint(&minimap, &mut draw);
            });
        });
        let regions: Vec<(RegionPos, Option<u32>)> =
            (0..800).map(|i| (RegionPos::new(i % 40 - 20, i / 40 - 10, 0), Some(i as u32 + 2))).collect();
        let world = WorldMapFrame {
            display: (1920.0, 1080.0),
            centre: (0.0, 0.0),
            zoom: 0.2,
            camera,
            player_block: (8, 70, 8),
            font_size: 18.0,
            show_coordinates: true,
            show_nether_coordinates: true,
            dim: 0,
            angle: 0.0,
            regions: &regions,
        };
        let turned = MinimapFrame { angle: 0.7, ..minimap };
        group.bench_function("minimap_turned", |b| {
            b.iter(|| {
                draw.clear();
                painter.paint(&turned, &mut draw);
            });
        });
        let world_turned = WorldMapFrame { angle: 0.7, ..world };
        group.bench_function("world_map_800_regions_turned", |b| {
            b.iter(|| {
                draw.clear();
                WorldMapPainter::paint(&world_turned, &mut draw);
            });
        });
        group.bench_function("world_map_800_regions", |b| {
            b.iter(|| {
                draw.clear();
                WorldMapPainter::paint(&world, &mut draw);
            });
        });
        // 512 waypoints on a 64-block grid around the player: some inside the disc, the rest at the edge
        let mut store = WaypointStore::new(512, 1);
        for i in 0..512 {
            let pos = ((i % 32 - 16) * 64, 70, (i / 32 - 8) * 64);
            let _ = store.add("", 0, pos, WaypointSource::Hotkey, 0);
        }
        let mut waypoints = WaypointPainter::new(&Config::default());
        let disc = painter.disc(minimap.display_width);
        group.bench_function("minimap_512_waypoints", |b| {
            b.iter(|| {
                draw.clear();
                painter.paint(&minimap, &mut draw);
                waypoints.paint_minimap(&minimap, disc, store.waypoints(), &mut draw);
            });
        });
        group.bench_function("world_map_512_waypoints", |b| {
            b.iter(|| {
                draw.clear();
                waypoints.paint_world_map(&world, store.waypoints(), &mut draw);
            });
        });
        group.finish();
    }

    // a 1920 x 1080 minimap frame through egui, with no events; the cursor is off the minimap
    fn ui_input() -> UiFrameInput<'static> {
        UiFrameInput {
            display: (1920.0, 1080.0),
            pixels_per_point: 1.0,
            time_s: 0.0,
            cursor: (960.0, 900.0),
            left_down: false,
            right_down: false,
            foreground: true,
            events: &[],
            draw_cursor: false,
            collect_text: false,
        }
    }

    // the map draw list painted, turned into egui shapes, tessellated and flattened, as one frame
    // costs on the render thread; three frames run first so the font atlas is built
    fn egui_frame(c: &mut Criterion) {
        let mut group = c.benchmark_group("egui_frame");
        let painter = MinimapPainter::new(&MiniMapConfig::default());
        let camera = CameraPose { x: 8.5, z: 8.5, yaw: 30.0 };
        let minimap = MinimapFrame {
            display_width: 1920.0,
            camera,
            player_block: (8, 70, 8),
            radius_chunks: 8,
            font_size: 18.0,
            texture: Some(1),
            image_origin: (-120, -120),
            image_side: 258,
            dim: 0,
            show_coordinates: true,
            angle: 0.0,
        };
        let turned = MinimapFrame { angle: 0.7, ..minimap };
        let input = Self::ui_input();
        let mut runtime = UiRuntime::new(FontSource::System);
        let mut mesh = UiMesh::new();
        let mut draw = DrawList::new();
        for frame in [&minimap, &turned] {
            draw.clear();
            painter.paint(frame, &mut draw);
            for _ in 0..3 {
                runtime.run_with(&input, &draw, |_| {}, &mut mesh);
            }
        }
        for (name, frame) in [("minimap", &minimap), ("minimap_turned", &turned)] {
            group.bench_function(name, |b| {
                b.iter(|| {
                    draw.clear();
                    painter.paint(frame, &mut draw);
                    runtime.run_with(&input, &draw, |_| {}, &mut mesh);
                    black_box(mesh.vertices.len());
                });
            });
        }
        let mut store = WaypointStore::new(512, 1);
        for i in 0..512 {
            let pos = ((i % 32 - 16) * 64, 70, (i / 32 - 8) * 64);
            let _ = store.add("", 0, pos, WaypointSource::Hotkey, 0);
        }
        let mut waypoints = WaypointPainter::new(&Config::default());
        let disc = painter.disc(minimap.display_width);
        group.bench_function("minimap_512_waypoints", |b| {
            b.iter(|| {
                draw.clear();
                painter.paint(&minimap, &mut draw);
                waypoints.paint_minimap(&minimap, disc, store.waypoints(), &mut draw);
                runtime.run_with(&input, &draw, |_| {}, &mut mesh);
                black_box(mesh.vertices.len());
            });
        });
        let regions: Vec<(RegionPos, Option<u32>)> =
            (0..800).map(|i| (RegionPos::new(i % 40 - 20, i / 40 - 10, 0), Some(i as u32 + 2))).collect();
        let world = WorldMapFrame {
            display: (1920.0, 1080.0),
            centre: (0.0, 0.0),
            zoom: 0.2,
            camera,
            player_block: (8, 70, 8),
            font_size: 18.0,
            show_coordinates: true,
            show_nether_coordinates: true,
            dim: 0,
            angle: 0.0,
            regions: &regions,
        };
        group.bench_function("world_map_800_regions", |b| {
            b.iter(|| {
                draw.clear();
                WorldMapPainter::paint(&world, &mut draw);
                runtime.run_with(&input, &draw, |_| {}, &mut mesh);
                black_box(mesh.vertices.len());
            });
        });
        Self::egui_panels(&mut group, &mut runtime, &world, &mut mesh);
        draw.clear();
        painter.paint(&minimap, &mut draw);
        group.sample_size(10);
        group.bench_function("first_frame", |b| {
            b.iter_batched(
                || UiRuntime::new(FontSource::System),
                |mut fresh| {
                    fresh.run_with(&input, &draw, |_| {}, &mut mesh);
                    black_box(mesh.pixels.len());
                },
                BatchSize::LargeInput,
            );
        });
        group.finish();
    }

    // the world map with the tools card and the list of 512 waypoints open
    fn egui_panels(
        group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
        runtime: &mut UiRuntime,
        world: &WorldMapFrame<'_>,
        mesh: &mut UiMesh,
    ) {
        let waypoints = OverlayScenes::waypoints();
        let rows = OverlayScenes::rows(&waypoints, "");
        let mut store = WaypointStore::new(8, 1);
        let mut view = ViewState::new(5);
        let mut ui = MapUi::new(1.0, 64);
        ui.apply(MapAction::ToggleList, "", 0, &mut view, &mut store, 0);
        // the same panels with the right-click menu open as well, as draw_list/world_map_panels has
        let mut panels = MapUi::new(1.0, 64);
        panels.apply(MapAction::ToggleList, "", 0, &mut view, &mut store, 0);
        panels.open_menu((960.0, 540.0), MenuTarget::Ground { dim: 0, x: 0, y: 64, z: 0 });
        let input = Self::ui_input();
        let mut draw = DrawList::new();
        let mut markers = WaypointPainter::new(&Config::default());
        let mut run = |runtime: &mut UiRuntime, draw: &mut DrawList, ui: &MapUi, with_markers: bool| {
            draw.clear();
            WorldMapPainter::paint(world, draw);
            if with_markers {
                markers.paint_world_map(world, &waypoints, draw);
            }
            let frame = UiFrame {
                panels: true,
                ui,
                view: &view,
                waypoints: &waypoints,
                rows: &rows,
                in_dimension: rows.len(),
                player_block: (0, 70, 0),
                dim: 0,
                hover: Some(HoverInfo { block: (-1, 0), chunk: (-1, 0), biome: Some("Plains") }),
                keys: HintKeys { toggle: 0x4D },
            };
            black_box(runtime.run(&input, draw, Some(&frame), mesh).len());
        };
        for _ in 0..3 {
            run(runtime, &mut draw, &ui, false);
        }
        group.bench_function("waypoint_list_512", |b| b.iter(|| run(runtime, &mut draw, &ui, false)));
        for _ in 0..3 {
            run(runtime, &mut draw, &panels, true);
        }
        group.bench_function("world_map_panels", |b| b.iter(|| run(runtime, &mut draw, &panels, true)));
    }

    // the game-thread work cave mode adds: preparing each probed chunk, and picking the layer each tick
    fn cave(c: &mut Criterion) {
        let mut group = c.benchmark_group("cave");
        let rock = CaveSurface::new([96, 96, 96, 255]);
        let probe = BenchTerrain::chunks(ChunkPos::new(0, 0, 0), 0)
            .into_iter()
            .next()
            .map(|(_, surface)| surface)
            .unwrap_or_else(|| panic!("a probe"));
        group.bench_function("prepare_chunk", |b| {
            b.iter(|| {
                let mut surface = probe;
                rock.prepare_chunk(&mut surface, 32);
                black_box(surface.surface[0]);
            });
        });
        let mut layers = LayerSelector::new(&Config::default());
        layers.toggle();
        let mut y = 0;
        group.bench_function("select_layer", |b| {
            b.iter(|| {
                y = (y + 1) % 128;
                black_box(layers.select(0, y));
            });
        });
        group.finish();
    }

    // an explored-area export of a 16 x 16 region world: rows composed from tiles and PNG-encoded
    // in memory, at 1 and 4 blocks per pixel
    fn export(c: &mut Criterion) {
        let mut group = c.benchmark_group("export");
        group.sample_size(10);
        let regions: Vec<RegionPos> = (0..16).flat_map(|z| (0..16).map(move |x| RegionPos::new(x, z, 0))).collect();
        let tile: Vec<u8> = (0..256 * 256).flat_map(|i: u32| [(i % 251) as u8, (i / 256) as u8, 90, 255]).collect();
        for max_pixels in [4096u32, 1024] {
            let layout = AreaLayout::new(&regions, max_pixels).unwrap_or_else(|| panic!("a layout"));
            let mut row = vec![0u8; layout.row_bytes()];
            let mut out = Vec::with_capacity(layout.width as usize * layout.height as usize * 4);
            let name = format!("area_16x16_regions_scale_{}", layout.blocks_per_pixel);
            group.bench_function(name, |b| {
                b.iter(|| {
                    out.clear();
                    let mut encoder = png::Encoder::new(&mut out, layout.width, layout.height);
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_compression(png::Compression::Fast);
                    let mut writer = encoder.write_header().unwrap_or_else(|error| panic!("png: {error}"));
                    let mut stream = writer.stream_writer().unwrap_or_else(|error| panic!("png: {error}"));
                    for _ in layout.z0..=layout.z1 {
                        for column in 0..16 {
                            AreaRow::place(
                                &mut row,
                                layout.width as usize,
                                column,
                                &tile,
                                layout.blocks_per_pixel as usize,
                            );
                        }
                        std::io::Write::write_all(&mut stream, &row).unwrap_or_else(|error| panic!("png: {error}"));
                    }
                    stream.finish().unwrap_or_else(|error| panic!("png: {error}"));
                    writer.finish().unwrap_or_else(|error| panic!("png: {error}"));
                    black_box(out.len());
                });
            });
        }
        group.finish();
    }

    fn store(c: &mut Criterion) {
        let mut group = c.benchmark_group("store");
        let dir = tempfile::tempdir().unwrap_or_else(|error| panic!("temp dir: {error}"));
        let chunks = BenchTerrain::chunks(ChunkPos::new(8, 8, 0), 8);
        let records: Vec<JournalRecord> = chunks
            .iter()
            .map(|(pos, surface)| JournalRecord::new(*pos, 7, *surface, Some([0xFF33_6633; 256])))
            .collect();
        let mut journal =
            Journal::open(&dir.path().join("append"), u64::MAX).unwrap_or_else(|error| panic!("journal: {error}"));
        group.throughput(Throughput::Elements(1));
        let mut next = 0;
        group.bench_function("journal_append", |b| {
            b.iter(|| {
                next = (next + 1) % records.len();
                journal.append(&records[next]).unwrap_or_else(|error| panic!("append: {error}"));
            });
        });
        let files = RegionFiles::new(dir.path().join("regions"));
        group.sample_size(10);
        group.bench_function("compaction_256_chunks", |b| {
            b.iter_batched(
                || {
                    let path = dir.path().join(format!("compact-{}", next));
                    next += 1;
                    let mut journal = Journal::open(&path, u64::MAX).unwrap_or_else(|error| panic!("journal: {error}"));
                    for record in records.iter().filter(|r| r.pos.region() == RegionPos::new(0, 0, 0)) {
                        journal.append(record).unwrap_or_else(|error| panic!("append: {error}"));
                    }
                    journal
                },
                |mut journal| journal.compact(&files).unwrap_or_else(|error| panic!("compact: {error}")),
                criterion::BatchSize::PerIteration,
            );
        });
        group.bench_function("region_load", |b| {
            b.iter(|| black_box(files.read(RegionPos::new(0, 0, 0)).unwrap_or_else(|error| panic!("read: {error}"))));
        });
        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20).measurement_time(Duration::from_secs(2)).warm_up_time(Duration::from_millis(500));
    targets = MinimapBenches::probe_submit, MinimapBenches::chunk_bake, MinimapBenches::region_bake,
        MinimapBenches::minimap_compose, MinimapBenches::draw_list, MinimapBenches::export, MinimapBenches::cave, MinimapBenches::store,
        MinimapBenches::world_map_tools, MinimapBenches::egui_frame
}
criterion_main!(benches);
