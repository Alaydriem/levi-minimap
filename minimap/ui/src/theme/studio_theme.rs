use egui::{Context, CornerRadius, Stroke, Theme, Visuals};

use crate::theme::{FontLoader, FontSource, Metrics, StudioPalette};

// installs the Studio Dark look on a context: fonts, dark visuals and the motion time
pub struct StudioTheme;

impl StudioTheme {
    pub fn install(ctx: &Context, source: FontSource) {
        ctx.set_fonts(FontLoader::definitions(source));
        ctx.set_theme(Theme::Dark);
        ctx.style_mut_of(Theme::Dark, |style| {
            style.visuals = Self::visuals();
            style.animation_time = Metrics::MOTION_SECONDS;
        });
    }

    fn visuals() -> Visuals {
        let mut visuals = Visuals::dark();
        visuals.panel_fill = StudioPalette::BG_PANEL;
        visuals.window_fill = StudioPalette::BG_RAISED;
        visuals.extreme_bg_color = StudioPalette::BG_FIELD;
        visuals.window_stroke = Stroke::new(1.0, StudioPalette::LINE_STRONG);
        visuals.window_corner_radius = CornerRadius::same(Metrics::RADIUS_PANEL);
        visuals.selection.bg_fill = StudioPalette::ACCENT_SOFT;
        visuals.selection.stroke = Stroke::new(1.5, StudioPalette::ACCENT);
        visuals.text_cursor.stroke = Stroke::new(1.5, StudioPalette::TEXT);
        visuals.override_text_color = Some(StudioPalette::TEXT);
        let widgets = &mut visuals.widgets;
        for state in [&mut widgets.inactive, &mut widgets.hovered, &mut widgets.active, &mut widgets.open] {
            state.corner_radius = CornerRadius::same(Metrics::RADIUS_CONTROL);
        }
        widgets.inactive.bg_fill = StudioPalette::BG_FIELD;
        widgets.inactive.weak_bg_fill = StudioPalette::BG_FIELD;
        widgets.inactive.bg_stroke = Stroke::new(1.0, StudioPalette::LINE_STRONG);
        widgets.hovered.bg_fill = StudioPalette::BG_HOVER;
        widgets.hovered.weak_bg_fill = StudioPalette::BG_HOVER;
        widgets.hovered.bg_stroke = Stroke::new(1.0, StudioPalette::LINE_STRONG);
        widgets.active.bg_fill = StudioPalette::BG_ACTIVE;
        widgets.active.weak_bg_fill = StudioPalette::BG_ACTIVE;
        widgets.active.bg_stroke = Stroke::new(1.5, StudioPalette::ACCENT);
        visuals
    }
}
