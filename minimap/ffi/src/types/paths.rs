use crate::types::LmStr;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LmPaths {
    pub config_file: LmStr,
    pub data_dir: LmStr,
}
