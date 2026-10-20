use egui::{Align2, Context, Id, LayerId, Order, Rect, pos2, vec2};
use minimap_core::structs::ui::{MessageKind, MessageLine};

use crate::theme::{Icon, Metrics, StudioPalette, Typeface};
use crate::widgets::Draw;

// the message line at the top centre, over either map: a check for something done, an info mark
// for a note. It takes no input, so it is painted straight on a foreground layer and shows in the
// frame that set it
pub struct ToastView;

impl ToastView {
    const PAD: (f32, f32) = (14.0, 9.0);
    const ICON: f32 = 14.0;
    const ICON_GAP: f32 = 22.0;

    pub fn show(ctx: &Context, message: &MessageLine) {
        let painter = ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("lm.toast")));
        let galley = Draw::galley(&painter, &message.text, Typeface::body(), StudioPalette::TEXT);
        let size = vec2(galley.size().x + Self::ICON_GAP + Self::PAD.0 * 2.0, galley.size().y + Self::PAD.1 * 2.0);
        let top = pos2(ctx.content_rect().center().x - size.x * 0.5, Metrics::GAP_L);
        let pill = Rect::from_min_size(top, size);
        Draw::shadow(&painter, pill, Metrics::RADIUS_PILL, 16.0, vec2(0.0, 4.0), StudioPalette::SHADOW);
        Draw::fill(&painter, pill, Metrics::RADIUS_PILL, StudioPalette::BG_RAISED);
        Draw::outline(&painter, pill, Metrics::RADIUS_PILL, 1.0, StudioPalette::LINE_STRONG);
        let (icon, colour) = match message.kind {
            MessageKind::Done => (Icon::CHECK, StudioPalette::SUCCESS),
            MessageKind::Info => (Icon::INFO, StudioPalette::TEXT_DIM),
        };
        let y = pill.center().y;
        Draw::icon(&painter, pos2(pill.left() + Self::PAD.0 + Self::ICON * 0.5, y), icon, Self::ICON, colour);
        let text = pos2(pill.left() + Self::PAD.0 + Self::ICON_GAP, y);
        Draw::place(&painter, galley, text, Align2::LEFT_CENTER);
    }
}
