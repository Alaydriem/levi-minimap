use crate::types::LmStr;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LmPaths {
    // the mod's LeviLamina config folder, an absolute path; the manager keeps config.hcl there
    pub config_dir: LmStr,
    pub data_dir: LmStr,
}
