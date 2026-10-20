// where the UI fonts come from: the Windows system fonts, or only egui's built-in fonts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontSource {
    System,
    BuiltIn,
}
