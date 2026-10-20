#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LmFrameInput {
    pub display_width: f32,
    pub display_height: f32,
    pub dt: f32,
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub left_down: u8,
    pub foreground: u8,
    pub right_down: u8,
    // telemetry: the shim's own work in the previous frame, the UI's vertex count, process memory
    pub render_us: u32,
    pub ui_vtx: u32,
    pub working_set_bytes: u64,
    pub private_bytes: u64,
    // 1: LmFrame lists the text laid out this frame (tests only)
    pub collect_text: u8,
}
