use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldIdentity {
    // a world saved on this machine
    Local { world_dir: PathBuf },
    // a server; the cache is kept per address and seed
    Remote { host: String, port: u16, seed: u32 },
}

impl WorldIdentity {
    pub const LEVEL_NAME_FILE: &'static str = "levelname.txt";

    // a short name for export files: the world's name from levelname.txt, else its folder's name;
    // or the server's address and port
    pub fn label(&self) -> String {
        match self {
            Self::Local { world_dir } => {
                let world = Self::world_folder(world_dir);
                std::fs::read_to_string(world.join(Self::LEVEL_NAME_FILE))
                    .ok()
                    .map(|name| name.trim().to_string())
                    .filter(|name| !name.is_empty())
                    .or_else(|| world.file_name().map(|n| n.to_string_lossy().into_owned()))
                    .unwrap_or_else(|| "world".to_string())
            }
            Self::Remote { host, port, .. } => format!("{host}_{port}"),
        }
    }

    // the mod's data for a world is <world>/ll_data/<mod name>; the world folder is two levels up
    fn world_folder(world_dir: &Path) -> &Path {
        match world_dir.parent() {
            Some(ll_data) if ll_data.file_name().is_some_and(|n| n == "ll_data") => {
                ll_data.parent().unwrap_or(world_dir)
            }
            _ => world_dir,
        }
    }
}
