use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BakeConfig {
    pub worker_threads: i32,
    pub queue_capacity: i32,
}

impl Default for BakeConfig {
    fn default() -> Self {
        Self { worker_threads: 0, queue_capacity: 4096 }
    }
}
