#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LmColumnSample {
    pub surface: u32,
    pub water: u32,
    pub height: i16,
    pub solid: i16,
    pub retry: u8,
    pub reserved: u8,
    // the raw game biome ID at the column's surface
    pub biome: u16,
}
