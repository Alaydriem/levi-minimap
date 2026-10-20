#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LmChunkPos {
    pub x: i32,
    pub z: i32,
    pub dim: i32,
}
