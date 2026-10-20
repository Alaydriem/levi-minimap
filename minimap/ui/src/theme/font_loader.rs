use std::path::Path;
use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

use crate::theme::{FontSource, Typeface};

// the Windows fonts of the Studio Dark look; a missing file falls back to egui's built-in fonts
pub struct FontLoader;

impl FontLoader {
    pub const FONT_DIR: &'static str = r"C:\Windows\Fonts";
    const UI: &'static str = "segoeui.ttf";
    // seguisb.ttf is Segoe UI Semibold; segoeuib.ttf is Bold, used only when Semibold is missing
    const SEMIBOLD: [&'static str; 2] = ["seguisb.ttf", "segoeuib.ttf"];
    const MONO: &'static str = "CascadiaMono.ttf";
    const ICONS: &'static str = "SegoeIcons.ttf";

    pub fn definitions(source: FontSource) -> FontDefinitions {
        let mut defs = FontDefinitions::default();
        let builtin_proportional = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
        let builtin_mono = defs.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
        let load = source == FontSource::System;
        let ui = Self::insert(&mut defs, load, "segoe-ui", &[Self::UI]);
        let semibold = Self::insert(&mut defs, load, "segoe-ui-semibold", &Self::SEMIBOLD);
        let mono = Self::insert(&mut defs, load, "cascadia-mono", &[Self::MONO]);
        let icons = Self::insert(&mut defs, load, "segoe-fluent-icons", &[Self::ICONS]);

        let ui_chain = Self::chain(&[ui.clone(), icons.clone()], &builtin_proportional);
        let semibold_chain = Self::chain(&[semibold, ui.clone(), icons.clone()], &builtin_proportional);
        let mono_chain = Self::chain(&[mono, ui, icons.clone()], &builtin_mono);
        let icon_chain = Self::chain(&[icons], &builtin_proportional);

        defs.families.insert(FontFamily::Proportional, ui_chain);
        defs.families.insert(FontFamily::Monospace, mono_chain.clone());
        defs.families.insert(FontFamily::Name(Typeface::SEMIBOLD.into()), semibold_chain);
        defs.families.insert(FontFamily::Name(Typeface::MONO.into()), mono_chain);
        defs.families.insert(FontFamily::Name(Typeface::ICONS.into()), icon_chain);
        defs
    }

    // registers the first file of `files` that can be read under `key`
    fn insert(defs: &mut FontDefinitions, load: bool, key: &str, files: &[&str]) -> Option<String> {
        if !load {
            return None;
        }
        for file in files {
            let path = Path::new(Self::FONT_DIR).join(file);
            if let Ok(bytes) = std::fs::read(&path) {
                defs.font_data.insert(key.to_owned(), Arc::new(FontData::from_owned(bytes)));
                return Some(key.to_owned());
            }
        }
        log::warn!("font {} not found in {}; using the built-in font", files.join(" or "), Self::FONT_DIR);
        None
    }

    fn chain(preferred: &[Option<String>], fallback: &[String]) -> Vec<String> {
        preferred.iter().flatten().cloned().chain(fallback.iter().cloned()).collect()
    }
}
