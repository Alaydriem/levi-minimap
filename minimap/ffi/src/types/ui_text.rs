// a text the frame laid out: len bytes from start of LmFrame.text_bytes, inside rect (x0, y0, x1, y1)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LmUiText {
    pub start: u32,
    pub len: u32,
    pub rect: [f32; 4],
}
