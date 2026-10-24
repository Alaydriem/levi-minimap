use egui::{Align2, Sense, Ui, vec2};
use levi_i18n::text::Text;
use levi_ui::draw::{Draw, TextFit, TextStyle, TypeSize};
use levi_ui::font::FontFace;
use levi_ui::text::UiText;
use levi_ui::tokens::{ColorToken, color};

/// A section caption in spaced capitals ("OVERLAYS", "NAME"), across the width of the card. The message gives its
/// words in capitals, and the translation keeps the case its translator chose.
pub struct Caption;

impl Caption {
    const STYLE: TextStyle = TextStyle::new(TypeSize::Caption, FontFace::Semibold).tracked();
    const HEIGHT: f32 = 16.0;

    pub fn show(ui: &mut Ui, text: &Text) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), Self::HEIGHT), Sense::hover());
        let words = UiText::resolve(ui.ctx(), text);
        let faint = <color::TextFaint as ColorToken>::VALUE;
        let galley = Draw::galley(ui.painter(), &words, Self::STYLE, faint, TextFit::Truncate(rect.width()));
        let at = Align2::LEFT_CENTER.anchor_size(rect.left_center(), galley.size());
        ui.painter().galley(at.min, galley, faint);
    }
}
