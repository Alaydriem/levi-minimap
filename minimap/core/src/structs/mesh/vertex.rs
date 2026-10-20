// one UI vertex in physical pixels; rgba is premultiplied sRGB, bytes in R, G, B, A order
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiVertex {
    pub x: f32,
    pub y: f32,
    pub u: f32,
    pub v: f32,
    pub rgba: u32,
}
