use egui::{Align2, Area, Id, Rect, pos2, vec2};
use minimap_ui::theme::{Icon, StudioPalette, Typeface};
use minimap_ui::widgets::{Draw, IconButton, TextButton, Toggle};

use crate::support::Headless;

#[test]
fn every_widget_and_helper_tessellates() {
    let ctx = Headless::context();
    let mut on = true;

    let mut kit = |ctx: &egui::Context| {
        Area::new(Id::unique("kit")).fixed_pos(pos2(20.0, 20.0)).show(ctx, |ui| {
            Toggle::show(ui, &mut on);
            IconButton::show(ui, Icon::CLOSE, 12.0);
            TextButton::primary(ui, "Save");
            TextButton::secondary(ui, "Cancel");
            let painter = ui.painter();
            let card = Rect::from_min_size(pos2(300.0, 20.0), vec2(200.0, 120.0));
            Draw::card(painter, card);
            Draw::caps(painter, card.min, Align2::LEFT_TOP, "Under cursor", StudioPalette::TEXT_FAINT);
            Draw::icon(painter, card.center(), Icon::CHECK, 14.0, StudioPalette::SUCCESS);
            Draw::hline(painter, card.left(), card.right(), card.bottom(), StudioPalette::LINE);
            let runs = [
                ("X ", Typeface::mono(14.0), StudioPalette::AXIS_X),
                ("12", Typeface::mono(14.0), StudioPalette::TEXT),
            ];
            Draw::place(painter, Draw::runs(painter, &runs), card.center(), Align2::CENTER_CENTER);
        });
    };
    // an area is laid out invisibly in its first pass
    let _ = Headless::frame(&ctx, [800.0, 600.0], Vec::new(), &mut kit);
    let output = Headless::frame(&ctx, [800.0, 600.0], Vec::new(), &mut kit);
    let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);

    assert!(!primitives.is_empty());
}
