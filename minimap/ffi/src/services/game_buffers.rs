use bytemuck::Zeroable;
use minimap_runtime::structs::ScanPlan;

use crate::types::{LmBlockColor, LmChunkPos, LmChunkSurface};

// game-thread buffers the shim reads or writes between calls
pub struct GameBuffers {
    pub staging: Box<LmChunkSurface>,
    pub plan: ScanPlan,
    pub candidates: Vec<LmChunkPos>,
    pub block_colors: Vec<LmBlockColor>,
}

impl GameBuffers {
    pub fn new() -> Self {
        Self {
            staging: Box::new(LmChunkSurface::zeroed()),
            plan: ScanPlan::default(),
            candidates: Vec::with_capacity(256),
            block_colors: Vec::new(),
        }
    }
}

impl Default for GameBuffers {
    fn default() -> Self {
        Self::new()
    }
}
