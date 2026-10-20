use std::fs;
use std::io::Write;
use std::path::Path;

use crate::errors::StoreError;

pub struct ConfigFile;

impl ConfigFile {
    pub fn read(path: &Path) -> Result<Option<String>, StoreError> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    // keep_invalid_copy saves the unreadable file as config.json.invalid before it is replaced
    pub fn write(path: &Path, json: &str, keep_invalid_copy: bool) -> Result<(), StoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if keep_invalid_copy && path.exists() {
            fs::copy(path, path.with_extension("json.invalid"))?;
        }
        let tmp = path.with_extension("json.tmp");
        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }
}
