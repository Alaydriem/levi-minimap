use crate::types::{LmTextureOp, LmUiDraw, LmUiText, LmUiTextureDelta, LmUiVertex};

// valid until the next lm_frame. Apply the map texture ops and the UI texture deltas, draw the
// draw calls, then free the UI textures in ui_freed
// cursor_flags: bit 0 the world map is open (the UI draws the arrow), bit 1 release the cursor
// clip, bit 2 typed text goes to lm_text, bit 3 copy this frame for an export
// clipboard: UTF-8 text for the system clipboard; empty when there is none
// texts: only when LmFrameInput.collect_text is set
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LmFrame {
    pub textures: *const LmTextureOp,
    pub texture_count: u32,
    pub vertices: *const LmUiVertex,
    pub vertex_count: u32,
    pub indices: *const u32,
    pub index_count: u32,
    pub draws: *const LmUiDraw,
    pub draw_count: u32,
    pub ui_textures: *const LmUiTextureDelta,
    pub ui_texture_count: u32,
    pub ui_freed: *const u32,
    pub ui_freed_count: u32,
    pub clipboard: *const u8,
    pub clipboard_len: u32,
    pub texts: *const LmUiText,
    pub text_count: u32,
    pub text_bytes: *const u8,
    pub text_bytes_len: u32,
    pub cursor_flags: u32,
}

impl LmFrame {
    pub fn empty() -> Self {
        Self {
            textures: std::ptr::null(),
            texture_count: 0,
            vertices: std::ptr::null(),
            vertex_count: 0,
            indices: std::ptr::null(),
            index_count: 0,
            draws: std::ptr::null(),
            draw_count: 0,
            ui_textures: std::ptr::null(),
            ui_texture_count: 0,
            ui_freed: std::ptr::null(),
            ui_freed_count: 0,
            clipboard: std::ptr::null(),
            clipboard_len: 0,
            texts: std::ptr::null(),
            text_count: 0,
            text_bytes: std::ptr::null(),
            text_bytes_len: 0,
            cursor_flags: 0,
        }
    }
}
