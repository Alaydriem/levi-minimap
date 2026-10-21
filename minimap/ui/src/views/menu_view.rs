use egui::{Align2, Area, Context, Id, Order, Sense, Ui, pos2, vec2};
use minimap_core::structs::ui::{MapAction, MapMenu, MenuItem, MenuTarget, UiFrame};

use studio_ui::theme::{Icon, Metrics, StudioPalette, Typeface};
use studio_ui::widgets::{CardFrame, Draw};

// the right-click menu at the point that opened it: a header naming the block or the waypoint,
// then one row per item; a left press anywhere else closes it
pub struct MenuView;

impl MenuView {
    pub const ROW_HEIGHT: f32 = 34.0;
    const OFFSET: f32 = 6.0;

    pub fn show(ctx: &Context, frame: &UiFrame<'_>, menu: &MapMenu, actions: &mut Vec<MapAction>) {
        let ppp = ctx.pixels_per_point().max(f32::EPSILON);
        let at = pos2(menu.at.0 / ppp + Self::OFFSET, menu.at.1 / ppp + Self::OFFSET);
        let shown = Area::new(Id::unique("lm.menu")).fixed_pos(at).order(Order::Foreground).show(ctx, |ui| {
            CardFrame::card().inner_margin(egui::Margin::symmetric(6, 10)).show(ui, |ui| {
                ui.set_width(Metrics::MENU_WIDTH - 12.0);
                Self::header(ui, frame, menu.target);
                for item in &menu.items {
                    if *item == MenuItem::ExportView {
                        ui.separator();
                    }
                    if Self::row(ui, *item, frame.ui.waypoints_shown()) {
                        actions.push(MapAction::Choose(*item));
                    }
                }
            });
        });
        let pressed_outside = ctx.input(|input| {
            input.pointer.primary_pressed()
                && input.pointer.interact_pos().is_some_and(|pos| !shown.response.rect.contains(pos))
        });
        if pressed_outside {
            actions.push(MapAction::CloseMenu);
        }
    }

    // "BLOCK x, z" for the ground, the waypoint's name for a marker
    pub fn title(frame: &UiFrame<'_>, target: MenuTarget) -> String {
        match target {
            MenuTarget::Ground { x, z, .. } => format!("Block {x}, {z}"),
            MenuTarget::Marker(id) => {
                frame.waypoints.iter().find(|w| w.id == id).map_or_else(|| "Waypoint".to_string(), |w| w.name.clone())
            }
        }
    }

    fn header(ui: &mut Ui, frame: &UiFrame<'_>, target: MenuTarget) {
        let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 22.0), Sense::hover());
        let at = pos2(rect.left() + 8.0, rect.center().y);
        Draw::caps(ui.painter(), at, Align2::LEFT_CENTER, &Self::title(frame, target), StudioPalette::TEXT_FAINT);
    }

    pub fn icon(item: MenuItem, waypoints_shown: bool) -> char {
        match item {
            MenuItem::AddWaypoint => Icon::ADD,
            MenuItem::CopyCoordinates => Icon::COPY,
            MenuItem::ToggleWaypoints if waypoints_shown => Icon::HIDE,
            MenuItem::ToggleWaypoints => Icon::SHOW,
            MenuItem::EditWaypoint => Icon::EDIT,
            MenuItem::RemoveWaypoint => Icon::DELETE,
            MenuItem::ExportView => Icon::CAMERA,
            MenuItem::ExportArea => Icon::SHARE,
        }
    }

    // one item; true when clicked
    fn row(ui: &mut Ui, item: MenuItem, waypoints_shown: bool) -> bool {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::ROW_HEIGHT), Sense::click());
        let painter = ui.painter();
        let hot = response.hovered();
        if hot {
            Draw::fill(painter, rect, Metrics::RADIUS_CONTROL, StudioPalette::BG_HOVER);
        }
        let colour = if hot { StudioPalette::TEXT } else { StudioPalette::TEXT_DIM };
        let y = rect.center().y;
        Draw::icon(painter, pos2(rect.left() + 18.0, y), Self::icon(item, waypoints_shown), 15.0, colour);
        let label = item.label(waypoints_shown);
        Draw::text(
            painter,
            pos2(rect.left() + 38.0, y),
            Align2::LEFT_CENTER,
            label,
            Typeface::body(),
            StudioPalette::TEXT,
        );
        response.clicked()
    }
}
