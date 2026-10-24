#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LmTickInput {
    pub present: u8,
    pub foreground: u8,
    pub cursor_hidden: u8,
    // the local player is dead
    pub dead: u8,
    // the game holds the mouse: no chat, menu or other screen is open
    pub mouse_grabbed: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub dim: i32,
    // process memory
    pub working_set_bytes: u64,
    pub private_bytes: u64,
}
