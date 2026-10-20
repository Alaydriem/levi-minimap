use minimap_core::structs::ui::MapAction;

use crate::support::Scene;

#[test]
fn the_slime_row_is_hidden_outside_the_overworld() {
    let mut scene = Scene::new(false);
    scene.dim = 1;
    scene.settle();

    assert!(!scene.mesh.draws_text("Slime chunks"));
    assert!(scene.mesh.draws_text("Nether"));
    assert!(scene.mesh.draws_text("Chunk grid"));
}

#[test]
fn a_click_on_a_switch_label_returns_its_action() {
    let mut scene = Scene::new(false);
    scene.settle();

    let grid = scene.click("Chunk grid");
    let slime = scene.click("Slime chunks");
    let list = scene.click("Waypoints");

    assert_eq!(
        (grid, slime, list),
        (vec![MapAction::ToggleGrid], vec![MapAction::ToggleSlime], vec![MapAction::ToggleList])
    );
}

#[test]
fn the_overlay_rows_say_zoom_in_below_the_overlay_zoom() {
    let mut scene = Scene::new(false);
    scene.view.zoom = 0.2;
    scene.settle();

    assert!(scene.mesh.draws_text("zoom in"));
}

#[test]
fn without_a_hover_the_info_rows_show_dashes() {
    let mut scene = Scene::new(false);
    scene.settle();

    let dashes = scene.mesh.text_rects.iter().filter(|t| scene.mesh.text_of(t) == "\u{2014}").count();
    assert_eq!(dashes, 3);
}
