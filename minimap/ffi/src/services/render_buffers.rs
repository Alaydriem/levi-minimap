use minimap_runtime::structs::FrameOutput;

use crate::types::{LmTextureOp, LmUiText, LmUiTextureDelta};

// render-thread buffers behind the LmFrame pointers; valid until the next lm_frame
pub struct RenderBuffers {
    pub output: FrameOutput,
    pub textures: Vec<LmTextureOp>,
    pub ui_textures: Vec<LmUiTextureDelta>,
    pub texts: Vec<LmUiText>,
}

impl RenderBuffers {
    pub fn new() -> Self {
        Self {
            output: FrameOutput::new(),
            textures: Vec::with_capacity(16),
            ui_textures: Vec::with_capacity(4),
            texts: Vec::new(),
        }
    }
}

impl Default for RenderBuffers {
    fn default() -> Self {
        Self::new()
    }
}
