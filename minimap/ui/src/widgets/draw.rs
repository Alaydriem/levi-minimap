use std::sync::Arc;

use egui::epaint::RectShape;
use egui::text::{LayoutJob, TextFormat};
use egui::{Align2, Color32, CornerRadius, FontId, Galley, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2, pos2};

use crate::theme::{StudioPalette, Typeface};

// the painting helpers the views share
pub struct Draw;

impl Draw {
    pub fn text(painter: &Painter, pos: Pos2, anchor: Align2, text: &str, font: FontId, color: Color32) -> Rect {
        painter.text(pos, anchor, text, font, color)
    }

    pub fn galley(painter: &Painter, text: &str, font: FontId, color: Color32) -> Arc<Galley> {
        painter.layout_no_wrap(text.to_owned(), font, color)
    }

    // paints a laid-out galley anchored like `text`; returns its rect
    pub fn place(painter: &Painter, galley: Arc<Galley>, pos: Pos2, anchor: Align2) -> Rect {
        let rect = anchor.anchor_size(pos, galley.size());
        painter.galley(rect.min, galley, StudioPalette::TEXT);
        rect
    }

    // an UPPERCASE section label with letter spacing
    pub fn caps(painter: &Painter, pos: Pos2, anchor: Align2, text: &str, color: Color32) -> Rect {
        let format = TextFormat {
            font_id: Typeface::semibold(Typeface::SECTION_SIZE),
            color,
            extra_letter_spacing: 0.7,
            ..Default::default()
        };
        let galley = painter.layout_job(LayoutJob::single_section(text.to_uppercase(), format));
        Self::place(painter, galley, pos, anchor)
    }

    // several differently coloured runs on one line
    pub fn runs(painter: &Painter, runs: &[(&str, FontId, Color32)]) -> Arc<Galley> {
        let mut job = LayoutJob::default();
        for (text, font, color) in runs {
            job.append(text, 0.0, TextFormat { font_id: font.clone(), color: *color, ..Default::default() });
        }
        painter.layout_job(job)
    }

    pub fn icon(painter: &Painter, center: Pos2, icon: char, size: f32, color: Color32) -> Rect {
        let mut buffer = [0u8; 4];
        painter.text(center, Align2::CENTER_CENTER, icon.encode_utf8(&mut buffer), Typeface::icon(size), color)
    }

    pub fn hline(painter: &Painter, x0: f32, x1: f32, y: f32, color: Color32) {
        let y = y.floor() + 0.5;
        painter.line_segment([pos2(x0, y), pos2(x1, y)], Stroke::new(1.0, color));
    }

    pub fn fill(painter: &Painter, rect: Rect, radius: u8, color: Color32) {
        painter.rect_filled(rect, CornerRadius::same(radius), color);
    }

    pub fn outline(painter: &Painter, rect: Rect, radius: u8, width: f32, color: Color32) {
        painter.rect_stroke(rect, CornerRadius::same(radius), Stroke::new(width, color), StrokeKind::Inside);
    }

    // a soft drop shadow under a rounded rect
    pub fn shadow(painter: &Painter, rect: Rect, radius: u8, blur: f32, offset: Vec2, color: Color32) {
        let corner = CornerRadius::same(radius.saturating_add(blur as u8 / 2));
        painter.add(RectShape::filled(rect.translate(offset).expand(blur * 0.25), corner, color).with_blur_width(blur));
    }

    // the card behind every map panel: shadow, translucent fill and a strong outline
    pub fn card(painter: &Painter, rect: Rect) {
        Self::shadow(painter, rect, 10, 24.0, Vec2::new(0.0, 6.0), StudioPalette::SHADOW);
        Self::fill(painter, rect, 10, StudioPalette::CARD);
        Self::outline(painter, rect, 10, 1.0, StudioPalette::LINE_STRONG);
    }
}
