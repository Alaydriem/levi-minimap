use minimap_core::services::draw::GpuResidency;
use minimap_core::structs::chunk::ColumnSample;
use minimap_core::structs::draw::{DrawCommand, TextureOp};
use minimap_core::structs::input::{KeyInput, MouseAction};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::{CursorAction, FrameOutput, ProbeRequest};
use minimap_store::services::{BiomeKeyFile, CachePaths};

use crate::support::SyntheticWorld;

const M: i32 = 0x4D;

fn minimap_uploaded(frame: &FrameOutput) -> bool {
    frame.textures.iter().any(|op| matches!(op, TextureOp::Upload { tex, .. } if *tex == GpuResidency::MINIMAP_TEXTURE))
}

fn region_uploaded(frame: &FrameOutput) -> bool {
    frame.textures.iter().any(|op| matches!(op, TextureOp::Upload { tex, .. } if *tex > GpuResidency::MINIMAP_TEXTURE))
}

#[test]
fn a_missing_config_is_written_with_the_defaults() {
    let world = SyntheticWorld::new();
    let runtime = Runtime::new(world.paths()).unwrap();
    runtime.shutdown();
    let text = std::fs::read_to_string(world.paths().config_file).unwrap();
    assert!(text.contains("\"widgetScale\""));
}

#[test]
fn a_session_scans_bakes_and_draws_the_terrain_in_the_minimap() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    let baked = world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 0);
    assert!(baked, "no bake finished: {:?}", runtime.stats());
    let uploaded = world.run_until(&runtime, 200, |_, frame| minimap_uploaded(frame));
    assert!(uploaded);
    let side = world.frame.textures.iter().find_map(|op| match op {
        TextureOp::Upload { tex: 1, width, .. } => Some(*width),
        _ => None,
    });
    assert_eq!(side, Some(162));
    assert!(world.frame.draw.commands.iter().any(|c| matches!(c, DrawCommand::ImageRounded { .. })));
    assert_eq!(world.plan.cursor, CursorAction::ClipToCentre);
    runtime.shutdown();
}

#[test]
fn opening_the_world_map_uploads_region_textures_and_frees_the_cursor() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 0);
    let open = runtime.key(KeyInput::new(M, true, false, true));
    assert_eq!(open.action, MouseAction::ReleaseMouse);
    assert!(world.run_until(&runtime, 200, |_, frame| region_uploaded(frame)));
    assert_eq!(world.frame.cursor_flags, FrameOutput::DRAW_CURSOR | FrameOutput::RELEASE_CLIP);
    world.tick(&runtime);
    assert_eq!(world.plan.cursor, CursorAction::None);
    runtime.shutdown();
}

#[test]
fn leaving_and_entering_again_loads_the_map_from_disk_without_any_probe() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 4);
    runtime.leave_world();
    assert!(SyntheticWorld::exists(&world.world_dir().join("levi_minimap_cache")));

    runtime.enter_world(&world.identity(), 0).unwrap();
    runtime.key(KeyInput::new(M, true, false, true));
    let mut loaded = false;
    for _ in 0..500 {
        let input = minimap_runtime::structs::TickInput {
            player: world.player,
            foreground: true,
            cursor_hidden: false,
            dead: false,
        };
        runtime.tick(&input, &mut world.plan);
        world.draw(&runtime);
        if region_uploaded(&world.frame) && runtime.stats().tiles_loaded > 0 {
            loaded = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(loaded, "{:?}", runtime.stats());
    assert_eq!(runtime.stats().chunks_submitted, 0);
    runtime.shutdown();
}

#[test]
fn a_long_walk_never_holds_more_surfaces_than_the_budget() {
    let mut world = SyntheticWorld::new();
    world.write_config(r#"{ "memory": { "chunkSurfaceBudgetMb": 8 }, "terrain": { "scanRadius": 16, "shadow": { "renderStyle": 0 } } }"#);
    let runtime = world.runtime();
    let capacity = runtime.stats().surface_capacity;
    assert!(capacity > 0 && capacity < 3000, "{capacity}");
    for step in 0..(capacity as i32 / 3 + 200) {
        world.player.x = (step * 16) as f32 + 8.5;
        world.tick(&runtime);
        assert!(runtime.stats().surfaces_resident <= capacity);
    }
    assert!(runtime.stats().surfaces_resident > capacity / 2);
    runtime.shutdown();
}

#[test]
fn a_dimension_change_starts_an_empty_chunk_store() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 50, |rt, _| rt.stats().surfaces_resident > 20);
    world.player.dim = 1;
    let input = minimap_runtime::structs::TickInput {
        player: world.player,
        foreground: true,
        cursor_hidden: true,
        dead: false,
    };
    runtime.tick(&input, &mut world.plan);
    assert!(runtime.stats().surfaces_resident < 10, "{:?}", runtime.stats());
    assert_eq!(world.plan.camera_height, 127);
    runtime.shutdown();
}

#[test]
fn a_block_change_asks_for_one_column_only_in_a_known_chunk() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 50, |rt, _| rt.stats().surfaces_resident > 0);
    assert_eq!(runtime.block_changed(3, 4, 0), ProbeRequest::Column { pos: ChunkPos::new(0, 0, 0), index: 4 * 16 + 3 });
    assert_eq!(runtime.block_changed(100_000, 0, 0), ProbeRequest::None);
    assert_eq!(runtime.block_changed(3, 4, 1), ProbeRequest::None);
    runtime.submit_column(ChunkPos::new(0, 0, 0), 4 * 16 + 3, ColumnSample::new(0xFF00_0000, 0, 120, 120, false, 0));
    runtime.shutdown();
}

#[test]
fn void_columns_use_a_config_override_or_the_game_fallback() {
    let world = SyntheticWorld::new();
    world.write_config(r#"{ "terrain": { "biomeColorOverrides": { "ocean": [0, 0, 112, 255] } } }"#);
    let runtime = Runtime::new(world.paths()).unwrap();
    assert_eq!(runtime.resolve_void_color("minecraft:ocean", 7), 0xFF70_0000);
    assert_eq!(runtime.resolve_void_color("minecraft:plains", 7), 7);
    runtime.shutdown();
}

#[test]
fn zoom_keys_are_saved_to_config_on_shutdown() {
    let world = SyntheticWorld::new();
    let runtime = world.runtime();
    runtime.key(KeyInput::new(0xBD, true, false, true));
    runtime.shutdown();
    let text = std::fs::read_to_string(world.paths().config_file).unwrap();
    assert!(text.contains("\"radiusChunks\": 6"), "{text}");
}

#[test]
fn telemetry_rows_are_written_with_the_shared_header() {
    let mut world = SyntheticWorld::new();
    world.write_config(r#"{ "debug": { "telemetry": true, "telemetryIntervalMs": 100 } }"#);
    let runtime = world.runtime();
    let started = std::time::Instant::now();
    while started.elapsed() < std::time::Duration::from_millis(450) {
        world.tick(&runtime);
        world.draw(&runtime);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    runtime.shutdown();
    let text = std::fs::read_to_string(world.paths().data_dir.join(Runtime::TELEMETRY_FILE)).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], minimap_runtime::structs::TelemetryRow::HEADER);
    assert!(lines.len() >= 3, "{text}");
    assert!(lines[1..].iter().all(|l| l.split(',').count() == 19));
}

#[test]
fn block_colours_from_the_config_are_offered_to_the_probe_packed() {
    let world = SyntheticWorld::new();
    world.write_config(r#"{ "terrain": { "blockColors": { "minecraft:lava": [230, 110, 20, 255] } } }"#);
    let runtime = Runtime::new(world.paths()).unwrap();
    let packed = minimap_core::structs::Rgba::new(230, 110, 20, 255).packed();
    assert_eq!(runtime.block_colors(), &[("minecraft:lava".to_string(), packed)]);
    runtime.shutdown();
}

#[test]
fn a_player_up_minimap_is_drawn_as_a_turned_disc() {
    let mut world = SyntheticWorld::new();
    world.write_config(r#"{ "miniMap": { "rotateWithPlayer": true } }"#);
    let runtime = world.runtime();
    world.player.yaw = 30.0;
    let drawn = world.run_until(&runtime, 2000, |_, frame| {
        frame.draw.commands.iter().any(|c| matches!(c, DrawCommand::ImageDisc { .. }))
    });
    assert!(drawn, "no disc: {:?}", runtime.stats());
    assert!(!world.frame.draw.commands.iter().any(|c| matches!(c, DrawCommand::ImageRounded { .. })));
    runtime.shutdown();
}

#[test]
fn the_plan_keeps_asking_for_probed_biome_ids_the_game_has_not_named() {
    let mut world = SyntheticWorld::new();
    world.answer_biomes = false;
    let runtime = world.runtime();
    let mut asked = Vec::new();
    for _ in 0..200 {
        world.tick(&runtime);
        asked.extend(world.plan.biome_requests.iter().copied());
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let repeats = asked.len();
    asked.sort_unstable();
    asked.dedup();
    assert_eq!(asked, vec![1, 4]);
    assert!(repeats > 2, "an unanswered ID is asked again: {repeats}");
    runtime.shutdown();
}

#[test]
fn a_probed_column_keeps_its_biome_after_the_game_names_it() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    let named = world.run_until(&runtime, 1000, |rt, _| {
        rt.column_biome(8, 8).as_deref() == Some("plains") && rt.column_biome(40, 8).as_deref() == Some("forest")
    });
    assert!(named, "plains {:?}, forest {:?}", runtime.column_biome(8, 8), runtime.column_biome(40, 8));
    runtime.shutdown();
}

#[test]
fn biome_keys_survive_leaving_and_entering_the_world() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    assert!(world.run_until(&runtime, 1000, |rt, _| rt.column_biome(8, 8).as_deref() == Some("plains")));
    runtime.leave_world();
    let keys = CachePaths::biome_keys_file(&CachePaths::root(&world.identity(), &world.paths().data_dir));
    let (table_id, names) = BiomeKeyFile::load(&keys).unwrap().expect("the key table is saved with the world");
    assert!(names.iter().any(|name| name == "plains"), "{names:?}");
    runtime.enter_world(&world.identity(), 0).unwrap();
    let restored = world.run_until(&runtime, 1000, |rt, _| rt.column_biome(8, 8).as_deref() == Some("plains"));
    assert!(restored, "{:?}", runtime.column_biome(8, 8));
    runtime.leave_world();
    assert_eq!(BiomeKeyFile::load(&keys).unwrap().map(|t| t.0), Some(table_id));
    runtime.shutdown();
}
