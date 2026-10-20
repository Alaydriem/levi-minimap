use bytemuck::{Pod, Zeroable};

// the probe's output for one chunk; same layout as minimap_core's ChunkProbe (3,776 bytes): the
// surface fields, bytes the probe leaves zero (core's biome palette and padding), then a raw game
// biome ID per column
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LmChunkSurface {
    pub surface: [u32; 256],
    pub water: [u32; 256],
    pub height: [i16; 256],
    pub solid: [i16; 256],
    pub skipped: [u64; 4],
    pub retry: [u64; 4],
    pub min_height: i16,
    pub max_height: i16,
    pub flags: u32,
    pub reserved: [u8; 120],
    pub biome: [u16; 256],
}

const _: () = assert!(std::mem::size_of::<LmChunkSurface>() == 3776);
