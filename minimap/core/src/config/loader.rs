use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::config::{Config, ConfigClamp, ConfigWarning, LoadedConfig, WriteBack};
use crate::errors::ConfigError;

pub struct ConfigLoader;

impl ConfigLoader {
    // text is the content of config.json, or None when the file does not exist
    pub fn load(text: Option<&str>) -> LoadedConfig {
        let Some(text) = text else {
            let warning = ConfigWarning::new("config", "config.json missing; defaults written".to_string());
            return LoadedConfig::new(Config::default(), vec![warning], WriteBack::Create);
        };
        let map = match serde_json::from_str::<Value>(text) {
            Ok(Value::Object(map)) => map,
            Ok(_) => return Self::invalid("the top level is not an object".to_string()),
            Err(error) => return Self::invalid(error.to_string()),
        };
        let (config, mut warnings) = Self::from_sections(&map);
        let (config, clamp_warnings) = ConfigClamp::new().apply(config);
        warnings.extend(clamp_warnings);
        LoadedConfig::new(config, warnings, WriteBack::None)
    }

    pub fn to_json(config: &Config) -> Result<String, ConfigError> {
        Ok(serde_json::to_string_pretty(config)?)
    }

    // a section that fails to parse falls back to its defaults; the other sections are kept
    fn from_sections(map: &Map<String, Value>) -> (Config, Vec<ConfigWarning>) {
        let defaults = Config::default();
        let mut warnings = Vec::new();
        let config = Config {
            version: Self::section(map, "version", defaults.version, &mut warnings),
            font_size: Self::section(map, "fontSize", defaults.font_size, &mut warnings),
            mini_map: Self::section(map, "miniMap", defaults.mini_map, &mut warnings),
            smooth_camera: Self::section(map, "smoothCamera", defaults.smooth_camera, &mut warnings),
            terrain: Self::section(map, "terrain", defaults.terrain, &mut warnings),
            world_map: Self::section(map, "worldMap", defaults.world_map, &mut warnings),
            memory: Self::section(map, "memory", defaults.memory, &mut warnings),
            bake: Self::section(map, "bake", defaults.bake, &mut warnings),
            store: Self::section(map, "store", defaults.store, &mut warnings),
            debug: Self::section(map, "debug", defaults.debug, &mut warnings),
            waypoints: Self::section(map, "waypoints", defaults.waypoints, &mut warnings),
            export: Self::section(map, "export", defaults.export, &mut warnings),
            cave_mode: Self::section(map, "caveMode", defaults.cave_mode, &mut warnings),
        };
        (config, warnings)
    }

    fn section<T: DeserializeOwned>(
        map: &Map<String, Value>,
        key: &'static str,
        default: T,
        warnings: &mut Vec<ConfigWarning>,
    ) -> T {
        let Some(value) = map.get(key) else { return default };
        match serde_json::from_value(value.clone()) {
            Ok(parsed) => parsed,
            Err(error) => {
                warnings.push(ConfigWarning::new(key, format!("{error}; this section uses its defaults")));
                default
            }
        }
    }

    fn invalid(reason: String) -> LoadedConfig {
        let warning = ConfigWarning::new("config", format!("config.json unreadable ({reason}); defaults written"));
        LoadedConfig::new(Config::default(), vec![warning], WriteBack::ReplaceInvalid)
    }
}
