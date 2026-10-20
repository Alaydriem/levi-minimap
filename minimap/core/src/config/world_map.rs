use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WorldMapConfig {
    pub toggle_key: i32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub zoom_step: f32,
    pub max_new_textures_per_frame: i32,
    pub texture_idle_frames: i32,
    pub show_coordinates: bool,
    pub show_nether_coordinates: bool,
    // turn the map so the player's facing is at the top
    pub rotate_with_player: bool,
    // waypoint names are drawn from this zoom up, in pixels per block
    pub waypoint_label_zoom: f32,
    // regions whose biomes the hover info keeps after reading them from disk
    pub hover_biome_regions: i32,
    pub show_chunk_grid: bool,
    pub show_slime_chunks: bool,
    // regions whose slime chunk masks are kept between frames
    pub slime_cache_regions: i32,
    // keys, typed text and wheel turns held for the next UI frame; the oldest is dropped past it
    pub ui_event_capacity: i32,
}

impl Default for WorldMapConfig {
    fn default() -> Self {
        Self {
            toggle_key: 0x4D,
            min_zoom: 0.2,
            max_zoom: 40.0,
            zoom_step: 1.15,
            max_new_textures_per_frame: 4,
            texture_idle_frames: 600,
            show_coordinates: true,
            show_nether_coordinates: true,
            rotate_with_player: false,
            waypoint_label_zoom: 0.5,
            hover_biome_regions: 16,
            show_chunk_grid: false,
            show_slime_chunks: false,
            slime_cache_regions: 1024,
            ui_event_capacity: 64,
        }
    }
}
