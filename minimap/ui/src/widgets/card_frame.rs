use egui::{Frame, Margin, Shadow, Stroke};

use crate::theme::{Metrics, StudioPalette};

// the frames of the map panels: the translucent card and the dark pill over the map
pub struct CardFrame;

impl CardFrame {
    pub const PADDING: i8 = 16;

    pub fn card() -> Frame {
        Frame::new()
            .fill(StudioPalette::CARD)
            .stroke(Stroke::new(1.0, StudioPalette::LINE_STRONG))
            .corner_radius(Metrics::RADIUS_PANEL)
            .inner_margin(Margin::same(Self::PADDING))
            .shadow(Shadow { offset: [0, 6], blur: 24, spread: 0, color: StudioPalette::SHADOW })
    }

    pub fn pill() -> Frame {
        Frame::new()
            .fill(StudioPalette::PILL)
            .corner_radius(Metrics::RADIUS_PILL)
            .inner_margin(Margin::symmetric(14, 8))
    }

    pub fn field() -> Frame {
        Frame::new()
            .fill(StudioPalette::BG_FIELD)
            .stroke(Stroke::new(1.0, StudioPalette::LINE_STRONG))
            .corner_radius(Metrics::RADIUS_CONTROL)
            .inner_margin(Margin::symmetric(10, 6))
    }
}
