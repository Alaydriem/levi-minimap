use egui::{Area, Context, Id, pos2};
use minimap_ui::widgets::Toggle;

use crate::support::Headless;

const SIZE: [f32; 2] = [400.0, 300.0];

fn show(ctx: &Context, on: &mut bool) {
    Area::new(Id::unique("toggle")).fixed_pos(pos2(10.0, 10.0)).show(ctx, |ui| {
        Toggle::show(ui, on);
    });
}

#[test]
fn toggle_click_flips_value_and_flips_back() {
    let ctx = Headless::context();
    let mut on = false;
    let centre = pos2(10.0 + Toggle::SIZE.0 * 0.5, 10.0 + Toggle::SIZE.1 * 0.5);

    Headless::click(&ctx, SIZE, centre, |ctx| show(ctx, &mut on));
    let first = on;
    Headless::click(&ctx, SIZE, centre, |ctx| show(ctx, &mut on));

    assert!(first);
    assert!(!on);
}

#[test]
fn click_beside_the_toggle_leaves_it() {
    let ctx = Headless::context();
    let mut on = false;

    Headless::click(&ctx, SIZE, pos2(200.0, 200.0), |ctx| show(ctx, &mut on));

    assert!(!on);
}
