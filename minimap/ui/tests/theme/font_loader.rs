use std::path::Path;

use egui::FontFamily;
use minimap_ui::theme::{FontLoader, FontSource, Typeface};

#[test]
fn built_in_source_registers_every_family_with_a_fallback() {
    let defs = FontLoader::definitions(FontSource::BuiltIn);

    for family in [Typeface::SEMIBOLD, Typeface::MONO, Typeface::ICONS] {
        let chain = defs.families.get(&FontFamily::Name(family.into()));
        assert!(chain.is_some_and(|c| !c.is_empty()), "{family} has no font");
    }
    assert!(!defs.font_data.contains_key("segoe-ui"));
}

#[test]
fn system_source_puts_segoe_ui_first_when_it_is_installed() {
    if !Path::new(FontLoader::FONT_DIR).join("segoeui.ttf").exists() {
        return;
    }

    let defs = FontLoader::definitions(FontSource::System);

    let chain = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    assert_eq!(chain.first().map(String::as_str), Some("segoe-ui"));
}
