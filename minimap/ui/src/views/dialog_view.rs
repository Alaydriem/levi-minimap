use egui::text::{LayoutJob, TextFormat};
use egui::{
    Align, Align2, Area, Color32, Context, Id, Key, LayerId, Layout, Order, RichText, Sense, Stroke, TextEdit, Ui,
    pos2, vec2,
};
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::ui::{DialogTarget, MapAction, WaypointDialog};

use crate::runtime::UiDrafts;
use crate::views::ToolsView;
use studio_ui::theme::{Icon, Metrics, StudioPalette, Typeface};
use studio_ui::widgets::{CardFrame, IconButton, TextButton, Toggle};

// the waypoint dialog over a dimmed map: the name, the colour, the two minimap switches, the
// position, Cancel and Save. Enter saves and Escape cancels, in order with the typed text
pub struct DialogView;

impl DialogView {
    pub const EDGE: &'static str = "Arrow at the edge when far away";
    pub const DISTANCE: &'static str = "Show distance on the arrow";
    const SWATCH: f32 = 15.0;
    const SWATCH_GAP: f32 = 46.0;
    const SWATCH_ROW: f32 = 40.0;
    pub const SWATCHES_PER_ROW: usize = 6;

    pub fn show(ctx: &Context, dialog: &WaypointDialog, drafts: &mut UiDrafts, actions: &mut Vec<MapAction>) {
        let opened = drafts.dialog_serial != dialog.serial;
        if opened {
            drafts.name.clone_from(&dialog.initial_name);
            drafts.dialog_serial = dialog.serial;
        }
        Self::dim(ctx);
        let id = Id::unique("lm.dialog");
        Area::new(id).anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0)).order(Order::Foreground).show(ctx, |ui| {
            CardFrame::card().inner_margin(egui::Margin::same(24)).show(ui, |ui| {
                ui.set_width(Metrics::DIALOG_WIDTH - 48.0);
                Self::title(ui, dialog, actions);
                ui.add_space(Metrics::GAP_M);
                ToolsView::caps(ui, "Name");
                Self::name(ui, drafts, opened);
                ui.add_space(Metrics::GAP_M);
                ToolsView::caps(ui, "Colour");
                Self::swatches(ui, dialog.colour, actions);
                ui.add_space(Metrics::GAP_M);
                ToolsView::caps(ui, "On the minimap");
                if Self::switch(ui, Self::EDGE, dialog.show_on_edge) {
                    actions.push(MapAction::DialogEdge);
                }
                if Self::switch(ui, Self::DISTANCE, dialog.show_distance) {
                    actions.push(MapAction::DialogDistance);
                }
                ui.add_space(Metrics::GAP_M);
                ToolsView::caps(ui, "Position");
                ui.label(Self::position(dialog.position));
                ui.add_space(Metrics::GAP_L);
                Self::buttons(ui, actions);
            });
        });
        ctx.move_to_top(LayerId::new(Order::Foreground, id));
        let (enter, escape) = ctx.input(|input| (input.key_pressed(Key::Enter), input.key_pressed(Key::Escape)));
        if enter {
            actions.push(MapAction::SaveDialog);
        } else if escape {
            actions.push(MapAction::CancelDialog);
        }
        // a dialog that closes gives the keyboard back to the game in the same frame; a refused
        // save takes the focus again in the next frame
        if actions.iter().any(|a| matches!(a, MapAction::SaveDialog | MapAction::CancelDialog)) {
            ctx.memory_mut(|memory| memory.stop_text_input());
        }
    }

    // a full-screen layer under the dialog: it dims the map and takes every press off it
    fn dim(ctx: &Context) {
        Area::new(Id::unique("lm.dim")).fixed_pos(pos2(0.0, 0.0)).order(Order::Foreground).show(ctx, |ui| {
            let screen = ctx.content_rect();
            let (rect, _) = ui.allocate_exact_size(screen.size(), Sense::click());
            ui.painter().rect_filled(rect, 0, StudioPalette::DIM);
        });
    }

    pub fn heading(dialog: &WaypointDialog) -> &'static str {
        match dialog.target {
            DialogTarget::New { .. } => "New waypoint",
            DialogTarget::Edit(_) => "Edit waypoint",
        }
    }

    fn title(ui: &mut Ui, dialog: &WaypointDialog, actions: &mut Vec<MapAction>) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(Self::heading(dialog)).font(Typeface::semibold(19.0)).color(StudioPalette::TEXT));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if IconButton::show(ui, Icon::CLOSE, 13.0).clicked() {
                    actions.push(MapAction::CancelDialog);
                }
            });
        });
    }

    fn name(ui: &mut Ui, drafts: &mut UiDrafts, focus: bool) {
        let field = TextEdit::singleline(&mut drafts.name)
            .id_salt("lm.dialog.name")
            .font(Typeface::body())
            .char_limit(WaypointStore::MAX_NAME_CHARS)
            .desired_width(f32::INFINITY)
            .margin(egui::Margin::symmetric(12, 9));
        let response = ui.add(field);
        let free = ui.ctx().memory(|memory| memory.focused().is_none());
        if focus || free {
            response.request_focus();
        }
    }

    // the palette in rows of SWATCHES_PER_ROW
    fn swatches(ui: &mut Ui, selected: [u8; 4], actions: &mut Vec<MapAction>) {
        let rows = WaypointStore::PALETTE.len().div_ceil(Self::SWATCHES_PER_ROW);
        let size = vec2(Self::SWATCH_GAP * Self::SWATCHES_PER_ROW as f32, Self::SWATCH_ROW * rows as f32);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        for (i, colour) in WaypointStore::PALETTE.iter().enumerate() {
            let (column, row) = (i % Self::SWATCHES_PER_ROW, i / Self::SWATCHES_PER_ROW);
            let x = rect.left() + 18.0 + column as f32 * Self::SWATCH_GAP;
            let centre = pos2(x, rect.top() + Self::SWATCH_ROW * (row as f32 + 0.5));
            let hit = egui::Rect::from_center_size(centre, vec2(Self::SWATCH * 2.4, Self::SWATCH * 2.4));
            let response = ui.interact(hit, ui.scope_id().with(("lm.swatch", i)), Sense::click());
            let painter = ui.painter();
            if *colour == selected {
                painter.circle_stroke(centre, Self::SWATCH + 4.0, Stroke::new(2.0, StudioPalette::WHITE));
            }
            painter.circle_filled(
                centre,
                Self::SWATCH,
                Color32::from_rgba_unmultiplied(colour[0], colour[1], colour[2], colour[3]),
            );
            if response.clicked() {
                actions.push(MapAction::DialogColour(*colour));
            }
        }
    }

    // a label and a switch; true when either was clicked
    fn switch(ui: &mut Ui, label: &str, on: bool) -> bool {
        let mut value = on;
        ui.horizontal(|ui| {
            ui.set_min_height(Metrics::ROW + 4.0);
            let text = RichText::new(label).font(Typeface::body()).color(StudioPalette::TEXT);
            let label_clicked = ui.add(egui::Label::new(text).sense(Sense::click()).selectable(false)).clicked();
            let switched =
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| Toggle::show(ui, &mut value).clicked());
            label_clicked || switched.inner
        })
        .inner
    }

    // "X 316   Y 71   Z -58" in mono with the axis letters in their colours
    pub fn position(position: (i32, i32, i32)) -> LayoutJob {
        let font = Typeface::mono(14.0);
        let mut job = LayoutJob::default();
        for (axis, value, colour, gap) in [
            ("X ", position.0, StudioPalette::AXIS_X, "   "),
            ("Y ", position.1, StudioPalette::AXIS_Y, "   "),
            ("Z ", position.2, StudioPalette::AXIS_Z, ""),
        ] {
            job.append(axis, 0.0, TextFormat::simple(font.clone(), colour));
            job.append(&format!("{value}{gap}"), 0.0, TextFormat::simple(font.clone(), StudioPalette::TEXT));
        }
        job
    }

    fn buttons(ui: &mut Ui, actions: &mut Vec<MapAction>) {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if TextButton::primary(ui, "Save").clicked() {
                actions.push(MapAction::SaveDialog);
            }
            if TextButton::secondary(ui, "Cancel").clicked() {
                actions.push(MapAction::CancelDialog);
            }
        });
    }
}
