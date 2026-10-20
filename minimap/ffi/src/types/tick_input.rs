#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LmTickInput {
    pub present: u8,
    pub foreground: u8,
    pub cursor_hidden: u8,
    // the local player is dead
    pub dead: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub dim: i32,
}
