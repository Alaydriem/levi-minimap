#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInput {
    // Win32 virtual-key code
    pub code: i32,
    pub down: bool,
    pub ctrl: bool,
    // the game has captured the mouse: no chat box or menu is open
    pub mouse_grabbed: bool,
}

impl KeyInput {
    pub fn new(code: i32, down: bool, ctrl: bool, mouse_grabbed: bool) -> Self {
        Self { code, down, ctrl, mouse_grabbed }
    }
}
