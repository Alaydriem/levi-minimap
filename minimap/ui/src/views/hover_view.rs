use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Color32, Layout, RichText, Ui};
use minimap_core::structs::ui::HoverInfo;

use crate::theme::{Metrics, StudioPalette, Typeface};

// the block, chunk and biome under the cursor, as label and value rows; dashes when the cursor is
// not on the map
pub struct HoverView;

impl HoverView {
    pub const NO_DATA: &'static str = "\u{2014}";
    const VALUE_SIZE: f32 = 14.0;

    pub fn show(ui: &mut Ui, hover: Option<HoverInfo<'_>>) {
        let block = hover.map(|h| Self::axes(h.block));
        Self::row(ui, "Block", block);
        let chunk = hover.map(|h| Self::plain(&format!("{}, {}", h.chunk.0, h.chunk.1), StudioPalette::TEXT));
        Self::row(ui, "Chunk", chunk);
        let biome = hover.and_then(|h| h.biome).map(|name| {
            let mut job = LayoutJob::default();
            job.append(name, 0.0, TextFormat::simple(Typeface::semibold(Typeface::UI_SIZE), StudioPalette::TEXT));
            job
        });
        Self::row(ui, "Biome", biome);
    }

    fn row(ui: &mut Ui, label: &str, value: Option<LayoutJob>) {
        ui.horizontal(|ui| {
            ui.set_min_height(Metrics::ROW);
            ui.label(RichText::new(label).font(Typeface::body()).color(StudioPalette::TEXT_DIM));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| match value {
                Some(job) => ui.label(job),
                None => ui.label(RichText::new(Self::NO_DATA).font(Typeface::body()).color(StudioPalette::TEXT_FAINT)),
            });
        });
    }

    // "X 316  Z -58" with the axis letters in their colours
    fn axes(block: (i32, i32)) -> LayoutJob {
        let font = Typeface::mono(Self::VALUE_SIZE);
        let mut job = LayoutJob::default();
        job.append("X ", 0.0, TextFormat::simple(font.clone(), StudioPalette::AXIS_X));
        job.append(&format!("{}  ", block.0), 0.0, TextFormat::simple(font.clone(), StudioPalette::TEXT));
        job.append("Z ", 0.0, TextFormat::simple(font.clone(), StudioPalette::AXIS_Z));
        job.append(&block.1.to_string(), 0.0, TextFormat::simple(font, StudioPalette::TEXT));
        job
    }

    fn plain(text: &str, colour: Color32) -> LayoutJob {
        let mut job = LayoutJob::default();
        job.append(text, 0.0, TextFormat::simple(Typeface::mono(Self::VALUE_SIZE), colour));
        job
    }
}
