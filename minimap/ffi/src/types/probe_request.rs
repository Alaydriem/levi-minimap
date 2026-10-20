// kind: 0 nothing to probe, 1 probe the column `index` of the chunk at (x, z, dim)
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LmProbeRequest {
    pub kind: u8,
    pub reserved: [u8; 3],
    pub x: i32,
    pub z: i32,
    pub dim: i32,
    pub index: u32,
}
