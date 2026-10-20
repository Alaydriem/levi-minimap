use std::sync::Mutex;

use minimap_runtime::services::Runtime;

use crate::services::{GameBuffers, RenderBuffers};

// the opaque handle C++ holds; game-thread and render-thread buffers have separate locks
pub struct LmHandle {
    pub runtime: Runtime,
    pub game: Mutex<GameBuffers>,
    pub render: Mutex<RenderBuffers>,
}

impl LmHandle {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime, game: Mutex::new(GameBuffers::new()), render: Mutex::new(RenderBuffers::new()) }
    }
}
