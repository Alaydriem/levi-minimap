// kind 1 upload `pixels` (RGBA, width * height * 4 bytes) into texture `tex`, creating or resizing it
// kind 2 release texture `tex`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LmTextureOp {
    pub kind: u8,
    pub tex: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: *const u8,
    pub len: usize,
}
