use egui::{Color32, RichText, Ui};
use levi_i18n::text::Text;
use levi_ui::draw::TextStyle;
use levi_ui::text::UiText;

/// The cards' egui labels, from a `Text` in the context's language.
pub struct Words;

impl Words {
    pub fn rich(ui: &Ui, text: &Text, style: TextStyle, color: Color32) -> RichText {
        RichText::new(UiText::resolve(ui.ctx(), text)).font(style.font_id()).color(color)
    }
}
