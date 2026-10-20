use crate::types::{LmBlockColor, LmChunkPos};

// candidates, block_colors and biome_requests stay valid until the next lm_tick; cursor: 0 none, 1 clip to
// centre, 2 release. Each biome request is a game biome ID the shim answers with lm_biome_name
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LmScanPlan {
    pub candidates: *const LmChunkPos,
    pub count: u32,
    pub limit: u32,
    pub camera_height: i32,
    pub transparent_water: u8,
    pub cursor: u8,
    pub block_colors: *const LmBlockColor,
    pub block_color_count: u32,
    pub biome_requests: *const u16,
    pub biome_request_count: u32,
}
