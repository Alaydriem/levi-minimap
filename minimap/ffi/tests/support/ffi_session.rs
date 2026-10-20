use minimap_ffi::api::lifecycle::{lm_create, lm_destroy, lm_enter_world};
use minimap_ffi::services::LmHandle;
use minimap_ffi::types::{LmPaths, LmStatus, LmStr, LmWorldInfo};

// a handle created through the C ABI over a temporary folder
pub struct FfiSession {
    pub dir: tempfile::TempDir,
    pub handle: *mut LmHandle,
    strings: Vec<String>,
}

impl FfiSession {
    pub fn str(text: &str) -> LmStr {
        LmStr { ptr: text.as_ptr(), len: text.len() }
    }

    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let strings = vec![
            dir.path().join("config.json").display().to_string(),
            dir.path().join("data").display().to_string(),
            dir.path().join("world").display().to_string(),
        ];
        let paths = LmPaths { config_file: Self::str(&strings[0]), data_dir: Self::str(&strings[1]) };
        let handle = lm_create(&paths, None);
        assert!(!handle.is_null());
        Self { dir, handle, strings }
    }

    pub fn enter(&self) {
        let info =
            LmWorldInfo { local_dir: Self::str(&self.strings[2]), host: Self::str(""), port: 0, seed: 0, dim: 0 };
        assert_eq!(lm_enter_world(self.handle, &info), LmStatus::Ok);
    }
}

impl Drop for FfiSession {
    fn drop(&mut self) {
        lm_destroy(self.handle);
    }
}
