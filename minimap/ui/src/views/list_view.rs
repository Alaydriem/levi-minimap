use egui::{Align, Align2, Area, Context, Id, Layout, Order, RichText, ScrollArea, Sense, TextEdit, Ui, pos2, vec2};
use minimap_core::services::ui::{WaypointDistance, WaypointFilter};
use minimap_core::structs::ui::{MapAction, UiFrame};
use minimap_core::structs::waypoint::Waypoint;

use crate::runtime::UiDrafts;
use crate::theme::{Icon, Metrics, StudioPalette, Typeface};
use crate::widgets::{CardFrame, Draw, IconButton};

// the waypoint list under the tools card on the left: a header with the count and a close button,
// the search field, the rows of the player's dimension and a footer hint
pub struct ListView;

impl ListView {
    pub const ROW_HEIGHT: f32 = 44.0;
    pub const MIN_ROWS: f32 = 2.0;
    // the header, the search field, the footer and the card's padding, around the rows
    const CHROME: f32 = 150.0;
    pub const EMPTY: &'static str = "No waypoints here";
    pub const SEARCH_HINT: &'static str = "Search waypoints";
    pub const FOOTER: &'static str = "Click a row to centre the map  \u{b7}  wheel scrolls";
    pub const HERE: &'static str = "Here";

    // `top` is the tools card's bottom edge; the rows fill the height left on the screen
    pub fn show(ctx: &Context, frame: &UiFrame<'_>, top: f32, drafts: &mut UiDrafts, actions: &mut Vec<MapAction>) {
        let top = top + Metrics::GAP_S;
        let rows_height = (ctx.content_rect().bottom() - Metrics::GAP_L - top - Self::CHROME)
            .max((Self::ROW_HEIGHT + 4.0) * Self::MIN_ROWS);
        let area = Area::new(Id::unique("lm.list")).fixed_pos(pos2(Metrics::GAP_L, top)).order(Order::Middle);
        area.show(ctx, |ui| {
            CardFrame::card().show(ui, |ui| {
                ui.set_width(Metrics::TOOLS_WIDTH - f32::from(CardFrame::PADDING) * 2.0);
                Self::header(ui, frame.in_dimension, actions);
                ui.add_space(Metrics::GAP_S);
                Self::search(ui, drafts);
                ui.add_space(Metrics::GAP_S);
                Self::rows(ui, frame, rows_height, actions);
                ui.add_space(Metrics::GAP_S);
                ui.separator();
                ui.label(RichText::new(Self::FOOTER).font(Typeface::ui(12.5)).color(StudioPalette::TEXT_FAINT));
            });
        });
    }

    fn header(ui: &mut Ui, count: usize, actions: &mut Vec<MapAction>) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Waypoints").font(Typeface::semibold(16.0)).color(StudioPalette::TEXT));
            let label = format!("{count} in this dimension");
            ui.label(RichText::new(label).font(Typeface::ui(Typeface::SMALL_SIZE)).color(StudioPalette::TEXT_DIM));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if IconButton::show(ui, Icon::CLOSE, 12.0).clicked() {
                    actions.push(MapAction::ToggleList);
                }
            });
        });
    }

    fn search(ui: &mut Ui, drafts: &mut UiDrafts) {
        CardFrame::field().show(ui, |ui| {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
                Draw::icon(ui.painter(), rect.center(), Icon::SEARCH, 14.0, StudioPalette::TEXT_DIM);
                let hint = RichText::new(Self::SEARCH_HINT).color(StudioPalette::TEXT_FAINT);
                let field = TextEdit::singleline(&mut drafts.search)
                    .id_salt("lm.search")
                    .hint_text(hint)
                    .font(Typeface::body())
                    .frame(egui::Frame::NONE)
                    .char_limit(WaypointFilter::QUERY_CHARS)
                    .desired_width(f32::INFINITY);
                ui.add(field);
            });
        });
    }

    fn rows(ui: &mut Ui, frame: &UiFrame<'_>, height: f32, actions: &mut Vec<MapAction>) {
        if frame.rows.is_empty() {
            ui.label(RichText::new(Self::EMPTY).font(Typeface::body()).color(StudioPalette::TEXT_FAINT));
            return;
        }
        ScrollArea::vertical().id_salt("lm.rows").max_height(height).auto_shrink([false, true]).show_rows(
            ui,
            Self::ROW_HEIGHT,
            frame.rows.len(),
            |ui, range| {
                for &index in &frame.rows[range] {
                    let Some(waypoint) = frame.waypoints.get(index) else { continue };
                    if Self::row(ui, waypoint, frame.player_block) {
                        let (x, z) = (f64::from(waypoint.x) + 0.5, f64::from(waypoint.z) + 0.5);
                        actions.push(MapAction::CentreOn { x, z });
                    }
                }
            },
        );
    }

    // one waypoint: its colour, name, distance and coordinates; true when clicked
    fn row(ui: &mut Ui, waypoint: &Waypoint, player: (i32, i32, i32)) -> bool {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), Self::ROW_HEIGHT), Sense::click());
        let painter = ui.painter();
        if response.hovered() {
            Draw::fill(painter, rect, Metrics::RADIUS_CONTROL, StudioPalette::BG_HOVER);
        }
        let x0 = rect.left() + 10.0;
        painter.circle_filled(pos2(x0, rect.center().y), 6.0, StudioPalette::rgba(waypoint.colour));
        let name = pos2(x0 + 16.0, rect.top() + 6.0);
        Draw::text(painter, name, Align2::LEFT_TOP, &waypoint.name, Typeface::body(), StudioPalette::TEXT);
        let blocks = WaypointDistance::blocks(player, (waypoint.x, waypoint.y, waypoint.z));
        let distance =
            if WaypointDistance::is_here(blocks) { Self::HERE.to_string() } else { format!("{blocks} m away") };
        let sub = pos2(x0 + 16.0, rect.top() + 25.0);
        Draw::text(painter, sub, Align2::LEFT_TOP, &distance, Typeface::ui(12.5), StudioPalette::TEXT_FAINT);
        let coords = format!("{}, {}", waypoint.x, waypoint.z);
        let right = pos2(rect.right() - 8.0, rect.center().y);
        Draw::text(painter, right, Align2::RIGHT_CENTER, &coords, Typeface::mono(13.0), StudioPalette::TEXT_DIM);
        response.clicked()
    }
}
