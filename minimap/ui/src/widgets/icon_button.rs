use egui::{Response, Sense, Ui, Vec2};

use crate::theme::{Metrics, StudioPalette};
use crate::widgets::Draw;

// a square button showing one icon glyph; the background shows on hover
pub struct IconButton;

impl IconButton {
    pub fn show(ui: &mut Ui, icon: char, size: f32) -> Response {
        let side = size + Metrics::GAP_S * 1.5;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click());
        let painter = ui.painter();
        if response.hovered() {
            Draw::fill(painter, rect, Metrics::RADIUS_CONTROL, StudioPalette::BG_HOVER);
        }
        let colour = if response.hovered() { StudioPalette::TEXT } else { StudioPalette::TEXT_DIM };
        Draw::icon(painter, rect.center(), icon, size, colour);
        response
    }
}
