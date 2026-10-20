use egui::{Response, Sense, Ui, pos2, vec2};

use crate::theme::StudioPalette;
use crate::widgets::Draw;

// the on/off switch of the mockups; a click flips `on`
pub struct Toggle;

impl Toggle {
    pub const SIZE: (f32, f32) = (38.0, 20.0);
    const KNOB: f32 = 7.0;

    pub fn show(ui: &mut Ui, on: &mut bool) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(vec2(Self::SIZE.0, Self::SIZE.1), Sense::click());
        if response.clicked() {
            *on = !*on;
            response.mark_changed();
        }
        let t = ui.ctx().animate_bool(response.id, *on);
        let painter = ui.painter();
        let radius = (Self::SIZE.1 * 0.5) as u8;
        let track = StudioPalette::BG_FIELD.lerp_to_gamma(StudioPalette::ACCENT, t);
        Draw::fill(painter, rect, radius, track);
        if t < 1.0 {
            Draw::outline(painter, rect, radius, 1.0, StudioPalette::alpha(StudioPalette::LINE_STRONG, 1.0 - t));
        }
        let left = rect.left() + Self::SIZE.1 * 0.5;
        let right = rect.right() - Self::SIZE.1 * 0.5;
        painter.circle_filled(pos2(left + (right - left) * t, rect.center().y), Self::KNOB, StudioPalette::WHITE);
        response
    }
}
