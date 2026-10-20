use minimap_core::structs::ui::HintKeys;
use minimap_ui::views::HintView;

use crate::support::Scene;

#[test]
fn the_hint_bar_names_the_configured_toggle_key() {
    assert!(HintView::text(HintKeys { toggle: 0x4D }).starts_with("M close"));
    assert!(HintView::text(HintKeys { toggle: 0x70 }).starts_with("F1 close"));
}

#[test]
fn the_hint_bar_takes_no_input() {
    let mut scene = Scene::new(false);
    scene.settle();
    let at = scene.centre_of(&HintView::text(HintKeys { toggle: 0x4D }));

    scene.frame(at, false, &[]);
    scene.frame(at, false, &[]);

    assert!(!scene.mesh.wants_pointer);
}
