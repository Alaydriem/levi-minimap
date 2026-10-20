#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FrameInput {
    pub display: (f32, f32),
    pub dt: f32,
    // client-area cursor position in pixels
    pub cursor: (f32, f32),
    pub left_down: bool,
    pub right_down: bool,
    pub foreground: bool,
    // telemetry from the shim: its own work in the previous frame, the UI's vertex count, process memory
    pub render_us: u32,
    pub ui_vtx: u32,
    pub working_set_bytes: u64,
    pub private_bytes: u64,
    // the frame lists the text it laid out, for tests
    pub collect_text: bool,
}
