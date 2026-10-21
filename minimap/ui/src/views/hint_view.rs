use egui::{Align2, Area, Context, Id, Order, RichText, vec2};
use minimap_core::structs::ui::HintKeys;

use studio_ui::theme::{Metrics, StudioPalette, Typeface};
use studio_ui::widgets::CardFrame;

// the pill at the bottom right that names the world map's controls; it takes no input
pub struct HintView;

impl HintView {
    pub fn text(keys: HintKeys<'_>) -> String {
        format!("{} close  \u{b7}  right-click for actions  \u{b7}  drag to pan", keys.toggle)
    }

    pub fn show(ctx: &Context, keys: HintKeys<'_>) {
        Area::new(Id::unique("lm.hint"))
            .anchor(Align2::RIGHT_BOTTOM, vec2(-Metrics::GAP_L, -Metrics::GAP_L))
            .order(Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                CardFrame::pill().show(ui, |ui| {
                    let text = RichText::new(Self::text(keys)).font(Typeface::ui(Typeface::NOTE_SIZE));
                    ui.label(text.color(StudioPalette::TEXT_DIM));
                });
            });
    }
}
