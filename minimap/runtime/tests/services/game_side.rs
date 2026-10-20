use minimap_core::services::cave::LayerIds;
use minimap_core::structs::input::KeyInput;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::FrameOutput;

use crate::support::SyntheticWorld;

const K: i32 = 0x4B;
const M: i32 = 0x4D;

fn press(runtime: &Runtime, code: i32) {
    runtime.key(KeyInput::new(code, true, false, true));
}

fn draws_text(frame: &FrameOutput, text: &str) -> bool {
    frame.mesh.draws_text(text)
}

#[test]
fn k_scans_the_band_around_the_feet_from_its_top_and_k_again_goes_back_to_the_surface() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    assert_eq!(world.plan.camera_height, 320);
    press(&runtime, K);
    world.tick(&runtime);
    assert_eq!(world.plan.camera_height, 95);
    assert!(world.plan.candidates.iter().all(|c| c.dim == LayerIds::cave(0, 5)));
    world.draw(&runtime);
    assert!(draws_text(&world.frame, "Cave view on"));
    press(&runtime, K);
    world.tick(&runtime);
    assert_eq!(world.plan.camera_height, 320);
    assert!(world.plan.candidates.iter().all(|c| c.dim == 0));
    runtime.shutdown();
}

#[test]
fn the_cave_badge_names_the_band_on_the_minimap_and_the_world_map() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    world.draw(&runtime);
    assert!(!draws_text(&world.frame, "CAVE \u{b7} Y 80 to 95"));
    press(&runtime, K);
    world.tick(&runtime);
    world.draw(&runtime);
    assert!(draws_text(&world.frame, "CAVE \u{b7} Y 80 to 95"));
    press(&runtime, M);
    world.tick(&runtime);
    world.draw(&runtime);
    assert_ne!(world.frame.cursor_flags & FrameOutput::DRAW_CURSOR, 0);
    assert!(draws_text(&world.frame, "CAVE \u{b7} Y 80 to 95"));
    runtime.shutdown();
}

#[test]
fn a_cave_band_is_saved_in_its_own_layer_folder() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    press(&runtime, K);
    assert!(world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 4));
    runtime.leave_world();
    runtime.shutdown();
    let cache = world.world_dir().join("levi_minimap_cache");
    let layer = LayerIds::cave(0, 5).to_string();
    assert!(cache.join("regions").join(&layer).exists() || cache.join("journal").join(&layer).exists());
    assert!(!cache.join("regions").join("0").exists());
}

#[test]
fn ctrl_k_hides_the_coordinates_and_the_choice_is_saved_to_config() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    world.draw(&runtime);
    assert!(draws_text(&world.frame, "8, 80, 8"));
    runtime.key(KeyInput::new(K, true, true, true));
    world.draw(&runtime);
    assert!(!draws_text(&world.frame, "8, 80, 8"));
    assert!(draws_text(&world.frame, "Coordinates hidden"));
    runtime.shutdown();
    let saved = std::fs::read_to_string(world.paths().config_file).unwrap();
    assert!(!saved.contains("\"showCoordinates\": true"), "{saved}");
}
