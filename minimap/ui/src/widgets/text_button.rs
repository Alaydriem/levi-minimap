use egui::{Align2, Color32, Response, Sense, Ui, vec2};

use crate::theme::{Metrics, StudioPalette, Typeface};
use crate::widgets::Draw;

// the dialog buttons: a filled accent primary and an outlined secondary
pub struct TextButton;

impl TextButton {
    const PAD: f32 = 18.0;

    pub fn primary(ui: &mut Ui, label: &str) -> Response {
        let galley = Draw::galley(ui.painter(), label, Typeface::semibold(Typeface::UI_SIZE), StudioPalette::WHITE);
        let size = vec2((galley.size().x + Self::PAD * 2.0).max(96.0), Metrics::BUTTON);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let fill = match (response.is_pointer_button_down_on(), response.hovered()) {
            (true, _) => StudioPalette::ACCENT_PRESS,
            (false, true) => StudioPalette::ACCENT_HOVER,
            (false, false) => StudioPalette::ACCENT,
        };
        let painter = ui.painter();
        Draw::fill(painter, rect, Metrics::RADIUS_CONTROL, fill);
        Draw::place(painter, galley, rect.center(), Align2::CENTER_CENTER);
        response
    }

    pub fn secondary(ui: &mut Ui, label: &str) -> Response {
        let galley = Draw::galley(ui.painter(), label, Typeface::body(), StudioPalette::TEXT);
        let size = vec2((galley.size().x + Self::PAD * 2.0).max(96.0), Metrics::BUTTON);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let painter = ui.painter();
        let fill = if response.hovered() { StudioPalette::BG_HOVER } else { Color32::TRANSPARENT };
        Draw::fill(painter, rect, Metrics::RADIUS_CONTROL, fill);
        Draw::outline(painter, rect, Metrics::RADIUS_CONTROL, 1.0, StudioPalette::LINE_STRONG);
        Draw::place(painter, galley, rect.center(), Align2::CENTER_CENTER);
        response
    }
}
