pub mod bake;
pub mod cave_mode;
pub mod clamp;
pub mod debug;
pub mod export;
pub mod loaded;
pub mod loader;
pub mod memory;
pub mod mini_map;
pub mod shadow;
pub mod smooth_camera;
pub mod store;
pub mod terrain;
pub mod warning;
pub mod waypoints;
pub mod world_map;
pub mod write_back;

pub use bake::BakeConfig;
pub use cave_mode::CaveModeConfig;
pub use clamp::ConfigClamp;
pub use debug::DebugConfig;
pub use export::ExportConfig;
pub use loaded::LoadedConfig;
pub use loader::ConfigLoader;
pub use memory::MemoryConfig;
pub use mini_map::MiniMapConfig;
pub use shadow::ShadowConfig;
pub use smooth_camera::SmoothCameraConfig;
pub use store::StoreConfig;
pub use terrain::TerrainConfig;
pub use warning::ConfigWarning;
pub use waypoints::WaypointsConfig;
pub use world_map::WorldMapConfig;
pub use write_back::WriteBack;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
    pub version: i32,
    pub font_size: f32,
    pub mini_map: MiniMapConfig,
    pub smooth_camera: SmoothCameraConfig,
    pub terrain: TerrainConfig,
    pub world_map: WorldMapConfig,
    pub memory: MemoryConfig,
    pub bake: BakeConfig,
    pub store: StoreConfig,
    pub debug: DebugConfig,
    pub waypoints: WaypointsConfig,
    pub export: ExportConfig,
    pub cave_mode: CaveModeConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            font_size: 18.0,
            mini_map: MiniMapConfig::default(),
            smooth_camera: SmoothCameraConfig::default(),
            terrain: TerrainConfig::default(),
            world_map: WorldMapConfig::default(),
            memory: MemoryConfig::default(),
            bake: BakeConfig::default(),
            store: StoreConfig::default(),
            debug: DebugConfig::default(),
            waypoints: WaypointsConfig::default(),
            export: ExportConfig::default(),
            cave_mode: CaveModeConfig::default(),
        }
    }
}
