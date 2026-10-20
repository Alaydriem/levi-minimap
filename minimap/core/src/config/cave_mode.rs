use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CaveModeConfig {
    // Win32 virtual-key code that switches between the surface and the cave view
    pub toggle_key: i32,
    // RGBA of columns with no open space under the band top
    pub solid_color: [u8; 4],
    // blocks per band: 8, 16 or 32
    pub band_size: i32,
    // blocks the feet may go past a band's edge before the next band is shown
    pub band_margin: i32,
    // blocks scanned around the player in the cave view
    pub scan_radius: i32,
}

impl Default for CaveModeConfig {
    fn default() -> Self {
        Self { toggle_key: 0x4B, solid_color: [96, 96, 96, 255], band_size: 16, band_margin: 2, scan_radius: 64 }
    }
}
