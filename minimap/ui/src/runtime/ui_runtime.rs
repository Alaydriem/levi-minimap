use egui::{Context, LayerId, Order, OutputCommand, pos2};
use minimap_core::structs::draw::DrawList;
use minimap_core::structs::mesh::UiMesh;
use minimap_core::structs::ui::{MapAction, UiFrame, UiFrameInput};

use crate::map::{CursorPainter, DrawListShapes};
use crate::runtime::{MeshFlattener, RawInputBuilder, TextCollector, UiDrafts};
use crate::theme::{FontSource, StudioTheme};
use crate::views::MapScreen;

// owns the egui context: one frame in, one tessellated UiMesh out. The map layers are drawn on
// the background layer, the panels above them, the cursor on top
pub struct UiRuntime {
    ctx: Context,
    input: RawInputBuilder,
    shapes: DrawListShapes,
    drafts: UiDrafts,
    // what this frame's panels asked for; keeps its capacity
    actions: Vec<MapAction>,
}

impl UiRuntime {
    pub fn new(fonts: FontSource) -> Self {
        let ctx = Context::default();
        StudioTheme::install(&ctx, fonts);
        Self {
            ctx,
            input: RawInputBuilder::new(),
            shapes: DrawListShapes::new(),
            drafts: UiDrafts::new(),
            actions: Vec::new(),
        }
    }

    // the map layers and, when given, the world map panels; returns what the panels asked for
    pub fn run(
        &mut self,
        input: &UiFrameInput<'_>,
        layers: &DrawList,
        frame: Option<&UiFrame<'_>>,
        out: &mut UiMesh,
    ) -> &[MapAction] {
        let mut actions = std::mem::take(&mut self.actions);
        actions.clear();
        let mut drafts = std::mem::take(&mut self.drafts);
        self.run_with(
            input,
            layers,
            |ctx| {
                if let Some(frame) = frame {
                    MapScreen::show(ctx, frame, &mut drafts, &mut actions);
                }
            },
            out,
        );
        self.drafts = drafts;
        self.actions = actions;
        &self.actions
    }

    // the map layers and any panels `panels` adds, for tests and benches that bring their own
    pub fn run_with(
        &mut self,
        input: &UiFrameInput<'_>,
        layers: &DrawList,
        mut panels: impl FnMut(&Context),
        out: &mut UiMesh,
    ) {
        out.clear();
        let ppp = input.pixels_per_point.max(f32::EPSILON);
        let raw = self.input.build(input);
        let shapes = &mut self.shapes;
        let mut output = self.ctx.run_pass(raw, |ctx| {
            shapes.paint(layers, &ctx.layer_painter(LayerId::background()), ppp);
            panels(ctx);
            if input.draw_cursor && input.foreground {
                CursorPainter::paint(ctx, pos2(input.cursor.0 / ppp, input.cursor.1 / ppp));
            }
        });
        out.wants_pointer = self.ctx.egui_wants_pointer_input();
        out.wants_keyboard = self.ctx.egui_wants_keyboard_input();
        for command in &output.platform_output.commands {
            if let OutputCommand::CopyText(text) = command {
                out.clipboard.clone_from(text);
            }
        }
        MeshFlattener::textures(&output.textures_delta, out);
        output.textures_delta.clear();
        if input.collect_text {
            TextCollector::collect(&output.shapes, ppp, out);
        }
        let primitives = self.ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point);
        MeshFlattener::meshes(&primitives, output.pixels_per_point, input.display, out);
    }

    // a panel from the last frame is under this screen point, in physical pixels; the point is
    // this frame's, so a press right after the cursor reaches a panel still lands on it. The
    // background layer is the map, not a panel
    pub fn covers(&self, at: (f32, f32), pixels_per_point: f32) -> bool {
        let ppp = pixels_per_point.max(f32::EPSILON);
        self.ctx.layer_id_at(pos2(at.0 / ppp, at.1 / ppp)).is_some_and(|layer| layer.order != Order::Background)
    }

    // the waypoint list's search as typed
    pub fn search(&self) -> &str {
        &self.drafts.search
    }

    pub fn drafts(&self) -> &UiDrafts {
        &self.drafts
    }

    // the world was left or the map closed: typed text is dropped
    pub fn reset_drafts(&mut self) {
        self.drafts.clear();
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }
}
