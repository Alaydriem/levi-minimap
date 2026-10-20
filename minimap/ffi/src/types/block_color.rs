use crate::types::LmStr;

// a block name and the packed RGBA colour the probe draws for it; colour is r | g << 8 | b << 16 | a << 24
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LmBlockColor {
    pub name: LmStr,
    pub colour: u32,
}
