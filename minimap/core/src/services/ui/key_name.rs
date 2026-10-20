// the label of a Windows virtual key code, as hints show it
pub struct KeyName;

impl KeyName {
    const LETTERS: [&'static str; 26] = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V",
        "W", "X", "Y", "Z",
    ];
    const DIGITS: [&'static str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    const FUNCTIONS: [&'static str; 12] = ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12"];

    pub fn of(vk: i32) -> &'static str {
        match vk {
            0x41..=0x5A => Self::LETTERS[(vk - 0x41) as usize],
            0x30..=0x39 => Self::DIGITS[(vk - 0x30) as usize],
            0x70..=0x7B => Self::FUNCTIONS[(vk - 0x70) as usize],
            0x1B => "Esc",
            0x0D => "Enter",
            0x20 => "Space",
            0x09 => "Tab",
            0xBB => "+",
            0xBD => "-",
            _ => "?",
        }
    }
}
