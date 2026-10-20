use minimap_core::structs::draw::{DrawList, TextureOp};
use minimap_core::structs::mesh::UiMesh;

// one frame for the shim; buffers keep their capacity between frames
#[derive(Debug, Default)]
pub struct FrameOutput {
    // the map layers the painters drew; the UI draws them under its panels
    pub draw: DrawList,
    // the whole frame, tessellated
    pub mesh: UiMesh,
    // the UI's own time to build and tessellate this frame
    pub ui_build_us: u32,
    pub textures: Vec<TextureOp>,
    // pixel data for Upload ops, indexed by TextureOp::Upload::buffer; only the first `buffers_used` are live
    pub buffers: Vec<Vec<u8>>,
    pub buffers_used: usize,
    pub cursor_flags: u32,
}

impl FrameOutput {
    // the world map is open: the system cursor is shown and the UI draws the arrow
    pub const DRAW_CURSOR: u32 = 1;
    pub const RELEASE_CLIP: u32 = 2;
    // a dialog has the keyboard: the shim sends typed characters with lm_text and keeps them from the game
    pub const TEXT_INPUT: u32 = 4;
    // after drawing this frame, the shim copies the back buffer and calls lm_view_captured
    pub const CAPTURE_VIEW: u32 = 8;

    pub fn new() -> Self {
        Self { draw: DrawList::new(), ..Self::default() }
    }

    pub fn clear(&mut self) {
        self.draw.clear();
        self.mesh.clear();
        self.ui_build_us = 0;
        self.textures.clear();
        self.buffers_used = 0;
        self.cursor_flags = 0;
    }

    // copies pixels into the next free buffer and returns its index
    pub fn stage(&mut self, pixels: &[u8]) -> u32 {
        if self.buffers_used == self.buffers.len() {
            self.buffers.push(Vec::new());
        }
        let buffer = &mut self.buffers[self.buffers_used];
        buffer.clear();
        buffer.extend_from_slice(pixels);
        self.buffers_used += 1;
        (self.buffers_used - 1) as u32
    }
}
