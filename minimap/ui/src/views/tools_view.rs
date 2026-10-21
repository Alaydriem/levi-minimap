use egui::{Align, Align2, Area, Context, Id, Label, Layout, Order, RichText, Sense, Ui, pos2, vec2};
use minimap_core::services::draw::OverlayGate;
use minimap_core::services::ui::DimensionName;
use minimap_core::structs::ui::{MapAction, UiFrame};

use crate::views::HoverView;
use studio_ui::theme::{Icon, Metrics, StudioPalette, Typeface};
use studio_ui::widgets::{CardFrame, Draw, Toggle};

// the top-left card: the dimension, the overlay switches, the waypoint list switch and what is
// under the cursor
pub struct ToolsView;

impl ToolsView {
    pub const ZOOM_IN: &'static str = "zoom in";

    // returns the card's bottom edge, where the waypoint list starts
    pub fn show(ctx: &Context, frame: &UiFrame<'_>, actions: &mut Vec<MapAction>) -> f32 {
        let area =
            Area::new(Id::unique("lm.tools")).fixed_pos(pos2(Metrics::GAP_L, Metrics::GAP_L)).order(Order::Foreground);
        let shown = area.show(ctx, |ui| {
            CardFrame::card().show(ui, |ui| {
                ui.set_width(Metrics::TOOLS_WIDTH - f32::from(CardFrame::PADDING) * 2.0);
                Self::header(ui, frame.dim);
                ui.add_space(Metrics::GAP_S);
                Self::caps(ui, "Overlays");
                let gated = !OverlayGate::open(frame.view.zoom);
                if Self::toggle_row(ui, "Chunk grid", frame.view.show_chunk_grid, gated) {
                    actions.push(MapAction::ToggleGrid);
                }
                if frame.dim == DimensionName::OVERWORLD
                    && Self::toggle_row(ui, "Slime chunks", frame.view.show_slime_chunks, gated)
                {
                    actions.push(MapAction::ToggleSlime);
                }
                if Self::waypoints_row(ui, frame.in_dimension, frame.ui.list_open()) {
                    actions.push(MapAction::ToggleList);
                }
                ui.add_space(Metrics::GAP_S);
                ui.separator();
                ui.add_space(Metrics::GAP_S);
                Self::caps(ui, "Under cursor");
                HoverView::show(ui, frame.hover);
            });
        });
        shown.response.rect.bottom()
    }

    fn header(ui: &mut Ui, dim: i32) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("World map").font(Typeface::semibold(16.0)).color(StudioPalette::TEXT));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let name = RichText::new(DimensionName::of(dim)).font(Typeface::ui(Typeface::NOTE_SIZE));
                ui.label(name.color(StudioPalette::TEXT_DIM));
            });
        });
    }

    pub fn caps(ui: &mut Ui, text: &str) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 16.0), Sense::hover());
        Draw::caps(ui.painter(), rect.left_center(), Align2::LEFT_CENTER, text, StudioPalette::TEXT_FAINT);
    }

    // a label with a switch at the right; true when the label or the switch was clicked
    fn toggle_row(ui: &mut Ui, label: &str, on: bool, gated: bool) -> bool {
        let mut value = on;
        ui.horizontal(|ui| {
            ui.set_min_height(Metrics::ROW);
            let text = RichText::new(label).font(Typeface::body()).color(StudioPalette::TEXT);
            let label_clicked = ui.add(Label::new(text).sense(Sense::click()).selectable(false)).clicked();
            if gated {
                let hint = RichText::new(Self::ZOOM_IN).font(Typeface::ui(12.5)).color(StudioPalette::TEXT_FAINT);
                ui.label(hint);
            }
            let switched =
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| Toggle::show(ui, &mut value).clicked());
            label_clicked || switched.inner
        })
        .inner
    }

    // the waypoint list switch: an icon, the label, the count and a chevron; true when clicked
    fn waypoints_row(ui: &mut Ui, count: usize, open: bool) -> bool {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
        let painter = ui.painter();
        let row = rect.expand2(vec2(Metrics::GAP_S * 0.75, 0.0));
        if open {
            Draw::fill(painter, row, Metrics::RADIUS_CONTROL, StudioPalette::ACCENT_SOFT);
        } else if response.hovered() {
            Draw::fill(painter, row, Metrics::RADIUS_CONTROL, StudioPalette::BG_HOVER);
        }
        let icon = if open { StudioPalette::ACCENT } else { StudioPalette::TEXT_DIM };
        let y = rect.center().y;
        Draw::icon(painter, pos2(rect.left() + 8.0, y), Icon::PIN, 15.0, icon);
        let label = pos2(rect.left() + 26.0, y);
        Draw::text(painter, label, Align2::LEFT_CENTER, "Waypoints", Typeface::body(), StudioPalette::TEXT);
        let badge = egui::Rect::from_center_size(pos2(rect.right() - 36.0, y), vec2(28.0, 18.0));
        Draw::fill(painter, badge, 9, StudioPalette::BG_FIELD);
        let count = count.to_string();
        Draw::text(
            painter,
            badge.center(),
            Align2::CENTER_CENTER,
            &count,
            Typeface::mono(12.0),
            StudioPalette::TEXT_DIM,
        );
        Draw::icon(painter, pos2(rect.right() - 8.0, y), Icon::CHEVRON, 12.0, StudioPalette::TEXT_DIM);
        response.clicked()
    }
}
