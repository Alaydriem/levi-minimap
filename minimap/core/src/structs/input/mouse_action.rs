// what the shim must ask the game to do with the mouse
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseAction {
    #[default]
    None,
    ReleaseMouse,
    GrabMouse,
}
