// one draw call of a UI frame: `index_count` indices from `index_offset`, each added to
// `vertex_offset`, clipped to `clip` (x0, y0, x1, y1 in physical pixels)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UiDraw {
    pub clip: [f32; 4],
    pub texture: u32,
    // EGUI: `texture` is a UI texture; MAP: it is a map texture id, with straight alpha
    pub kind: u8,
    // SamplerKind as a byte: 0 point, 1 linear
    pub sampler: u8,
    pub reserved: [u8; 2],
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
}

impl UiDraw {
    pub const EGUI: u8 = 0;
    pub const MAP: u8 = 1;
    pub const POINT: u8 = 0;
    pub const LINEAR: u8 = 1;
}
