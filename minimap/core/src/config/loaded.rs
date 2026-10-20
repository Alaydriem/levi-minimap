use crate::config::{Config, ConfigWarning, WriteBack};

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedConfig {
    pub config: Config,
    pub warnings: Vec<ConfigWarning>,
    pub write_back: WriteBack,
}

impl LoadedConfig {
    pub fn new(config: Config, warnings: Vec<ConfigWarning>, write_back: WriteBack) -> Self {
        Self { config, warnings, write_back }
    }
}
