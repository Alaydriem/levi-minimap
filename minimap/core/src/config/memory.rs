use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MemoryConfig {
    pub chunk_surface_budget_mb: i32,
    pub tile_budget_mb: i32,
    pub tile_overrun_percent: i32,
    pub gpu_texture_budget: i32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self { chunk_surface_budget_mb: 64, tile_budget_mb: 256, tile_overrun_percent: 25, gpu_texture_budget: 1024 }
    }
}
