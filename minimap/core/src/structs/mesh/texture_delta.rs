// an upload to a UI texture: the whole texture when `full`, else the patch at (x, y); its RGBA
// pixels (premultiplied) are `len` bytes from `offset` in the mesh's pixel buffer
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UiTextureDelta {
    pub texture: u32,
    pub full: bool,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub offset: usize,
    pub len: usize,
}
