use egui::{FontFamily, FontId};

// font ids of the Studio Dark type scale
pub struct Typeface;

impl Typeface {
    pub const SEMIBOLD: &'static str = "ui-semibold";
    pub const MONO: &'static str = "ui-mono";
    pub const ICONS: &'static str = "ui-icons";

    pub const UI_SIZE: f32 = 15.0;
    pub const SMALL_SIZE: f32 = 13.0;
    pub const SECTION_SIZE: f32 = 11.5;

    pub fn ui(size: f32) -> FontId {
        FontId::new(size, FontFamily::Proportional)
    }

    pub fn body() -> FontId {
        Self::ui(Self::UI_SIZE)
    }

    pub fn semibold(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name(Self::SEMIBOLD.into()))
    }

    pub fn mono(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name(Self::MONO.into()))
    }

    pub fn icon(size: f32) -> FontId {
        FontId::new(size, FontFamily::Name(Self::ICONS.into()))
    }
}
