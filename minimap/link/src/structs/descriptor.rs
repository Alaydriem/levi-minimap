// the descriptor the mod registers with the manager: id, claims, keybinds and the settings schema
pub struct Descriptor;

impl Descriptor {
    pub const TEXT: &'static str = include_str!("../../descriptor.hcl");
}
