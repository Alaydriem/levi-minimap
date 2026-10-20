use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DebugConfig {
    pub telemetry: bool,
    pub telemetry_interval_ms: i32,
    pub capture_probes: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self { telemetry: false, telemetry_interval_ms: 1000, capture_probes: false }
    }
}
