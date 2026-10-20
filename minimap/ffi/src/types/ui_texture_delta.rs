// an upload to UI texture `texture`: the whole texture (created or replaced) when full is 1, else
// the patch at (x, y); pixels are width * height RGBA bytes, premultiplied
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LmUiTextureDelta {
    pub texture: u32,
    pub full: u8,
    pub reserved: [u8; 3],
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: *const u8,
    pub len: usize,
}
