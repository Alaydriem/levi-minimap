use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ShadowConfig {
    pub render_style: i32,
    pub shadow_level: i32,
    pub render_scale: i32,
    pub pcf_radius: i32,
    pub light_azimuth: f32,
    pub light_zenith: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            render_style: 2,
            shadow_level: 130,
            render_scale: 2,
            pcf_radius: 1,
            light_azimuth: 315.0,
            light_zenith: 60.0,
        }
    }
}
