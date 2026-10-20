use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WaypointsConfig {
    pub enabled: bool,
    // Win32 virtual-key code that adds a waypoint at the player
    pub add_key: i32,
    pub max_per_world: i32,
    pub show_on_minimap: bool,
    pub max_edge_arrows: i32,
    // waypoint names are drawn at fontSize times this
    pub label_scale: f32,
}

impl Default for WaypointsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            add_key: 0x4E,
            max_per_world: 512,
            show_on_minimap: true,
            max_edge_arrows: 16,
            label_scale: 1.0,
        }
    }
}
