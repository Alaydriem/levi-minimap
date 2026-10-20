use egui::{Color32, Context, LayerId, Pos2, Shape, Stroke, vec2};

// the arrow pointer, drawn on the top layer while the game hides the system cursor
pub struct CursorPainter;

impl CursorPainter {
    pub const HEIGHT: f32 = 18.0;
    const WIDTH: f32 = 13.0;

    pub fn paint(ctx: &Context, tip: Pos2) {
        let points = vec![tip, tip + vec2(0.0, Self::HEIGHT), tip + vec2(Self::WIDTH, Self::WIDTH)];
        let painter = ctx.layer_painter(LayerId::debug());
        painter.add(Shape::convex_polygon(points, Color32::WHITE, Stroke::new(1.2, Color32::BLACK)));
    }
}
