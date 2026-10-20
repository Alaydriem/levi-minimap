use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::ui::{MapAction, MenuItem, MenuTarget};
use minimap_ui::theme::Icon;

use crate::support::Scene;

#[test]
fn a_done_message_shows_a_check_in_the_frame_that_set_it() {
    let mut scene = Scene::new(false);
    let mut store = WaypointStore::new(8, 1);
    scene.ui.open_menu((900.0, 400.0), MenuTarget::Ground { dim: 0, x: 1, y: 64, z: 2 });
    scene.ui.apply(MapAction::Choose(MenuItem::CopyCoordinates), "", 0, &mut scene.view, &mut store, 0);

    scene.frame((1200.0, 700.0), false, &[]);

    assert!(scene.mesh.draws_text("Copied 1, ~, 2"));
    assert!(scene.mesh.draws_text(&Icon::CHECK.to_string()));
}

#[test]
fn a_note_shows_an_info_mark() {
    let mut scene = Scene::new(false);
    scene.ui.show_message("An export is already running".to_string());

    scene.frame((1200.0, 700.0), false, &[]);

    assert!(scene.mesh.draws_text("An export is already running"));
    assert!(scene.mesh.draws_text(&Icon::INFO.to_string()));
    assert!(!scene.mesh.draws_text(&Icon::CHECK.to_string()));
}
