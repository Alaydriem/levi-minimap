use minimap_core::structs::ui::HintKeys;
use minimap_ui::runtime::LayerPainter;
use minimap_ui::views::HintView;

use crate::support::Scene;

#[test]
fn the_hint_bar_names_the_configured_toggle_key() {
    assert!(HintView::text(HintKeys { toggle: "M" }).starts_with("M close"));
    assert!(HintView::text(HintKeys { toggle: "Ctrl+Shift+F1" }).starts_with("Ctrl+Shift+F1 close"));
}

// the hint bar takes no input, so a press on it reaches the map under it
#[test]
fn the_hint_bar_does_not_cover_the_map() {
    let mut scene = Scene::new(false);
    scene.settle();
    let at = scene.centre_of(&HintView::text(HintKeys { toggle: "M" }));

    scene.frame(at, false, &[]);
    scene.frame(at, false, &[]);

    assert!(!LayerPainter::covers(scene.harness.context(), at));
}
