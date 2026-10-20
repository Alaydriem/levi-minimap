// a text laid out in a UI frame: `len` bytes from `start` of the mesh's texts, and its screen
// rect as [x0, y0, x1, y1] in physical pixels
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct UiTextRect {
    pub start: u32,
    pub len: u32,
    pub rect: [f32; 4],
}
