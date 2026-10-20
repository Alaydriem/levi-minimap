use crate::types::LmStr;

// a local world sets local_dir; a server leaves it empty and sets host, port and seed
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LmWorldInfo {
    pub local_dir: LmStr,
    pub host: LmStr,
    pub port: u16,
    pub seed: u32,
    pub dim: i32,
}
