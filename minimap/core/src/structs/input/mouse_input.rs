// a LeviLamina mouse event: action button id 4 is the wheel; data is its signed step
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseInput {
    pub button: i32,
    pub data: i32,
}

impl MouseInput {
    pub const WHEEL: i32 = 4;

    pub fn new(button: i32, data: i32) -> Self {
        Self { button, data }
    }
}
