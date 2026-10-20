// action: 0 none, 1 release the mouse, 2 grab the mouse
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LmKeyResult {
    pub consume: u8,
    pub action: u8,
}
