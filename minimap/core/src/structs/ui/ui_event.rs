// one input event for the UI, as the game thread receives it; `code` is a Windows virtual key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEvent {
    Text(char),
    Key { code: i32, down: bool, ctrl: bool, shift: bool },
    // the wheel's turn as Windows reports it: 120 per notch, positive away from the user
    Wheel(i32),
}
