use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::config::ShadowConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TerrainConfig {
    pub scan_max_chunks_per_frame: i32,
    pub scan_radius: i32,
    pub enable: bool,
    pub rescan_interval_frames: i32,
    pub region_bake_threshold: i32,
    pub enable_disk_cache: bool,
    pub enable_transparent_water: bool,
    pub water_opacity_per_depth: f32,
    pub water_max_opacity: f32,
    pub shadow: ShadowConfig,
    // block name to the RGBA colour drawn for it in place of the game's map colour
    pub block_colors: BTreeMap<String, [u8; 4]>,
    // biome name to the RGBA colour of a column with no blocks, in place of the game's map grass colour
    pub biome_color_overrides: BTreeMap<String, [u8; 4]>,
}

impl TerrainConfig {
    pub const MAX_BLOCK_COLORS: usize = 64;
    pub const MAX_BIOME_COLOR_OVERRIDES: usize = 64;
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            scan_max_chunks_per_frame: 32,
            scan_radius: 128,
            enable: true,
            rescan_interval_frames: 1200,
            region_bake_threshold: 64,
            enable_disk_cache: true,
            enable_transparent_water: true,
            water_opacity_per_depth: 0.1,
            water_max_opacity: 0.6,
            shadow: ShadowConfig::default(),
            block_colors: BTreeMap::from([("minecraft:lava".to_string(), [230, 110, 20, 255])]),
            // the End's void: near black with a purple tint, in place of the game's map grass green
            biome_color_overrides: BTreeMap::from([("the_end".to_string(), [16, 12, 26, 255])]),
        }
    }
}
