// one draw call: index_count indices from index_offset, each added to vertex_offset, scissored to
// clip (x0, y0, x1, y1 in physical pixels)
//   kind 0: texture is a UI texture from LmUiTextureDelta, premultiplied alpha
//   kind 1: texture is a map texture from LmTextureOp, straight alpha
//   sampler 0 point, 1 linear
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LmUiDraw {
    pub clip: [f32; 4],
    pub texture: u32,
    pub kind: u8,
    pub sampler: u8,
    pub reserved: [u8; 2],
    pub index_offset: u32,
    pub index_count: u32,
    pub vertex_offset: u32,
}
