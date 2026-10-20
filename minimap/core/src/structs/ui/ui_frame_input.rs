use crate::structs::ui::UiEvent;

// what one UI frame is given: the screen, the scale, the pointer and the queued input
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiFrameInput<'a> {
    pub display: (f32, f32),
    // physical pixels per UI point
    pub pixels_per_point: f32,
    pub time_s: f64,
    // client-area cursor position in pixels
    pub cursor: (f32, f32),
    pub left_down: bool,
    pub right_down: bool,
    pub foreground: bool,
    pub events: &'a [UiEvent],
    // the arrow pointer is drawn on top of everything
    pub draw_cursor: bool,
    // the laid-out text is listed in the mesh, for tests
    pub collect_text: bool,
}
