use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePaths {
    pub config_file: PathBuf,
    // per-mod data folder; server caches live under it
    pub data_dir: PathBuf,
}
