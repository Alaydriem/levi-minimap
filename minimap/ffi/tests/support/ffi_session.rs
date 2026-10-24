use std::sync::Arc;

use amm_client::test_support::FakeTable;
use minimap_core::config::Config;
use minimap_core::structs::draw::TextureGrant;
use minimap_ffi::api::lifecycle::{lm_destroy, lm_enter_world};
use minimap_ffi::services::LmHandle;
use minimap_ffi::types::{LmStatus, LmStr, LmWorldInfo};
use minimap_link::services::{MinimapSession, SystemClipboard};
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::RuntimePaths;

// a handle over a temporary folder, registered with amm-client's fake manager table
pub struct FfiSession {
    pub dir: tempfile::TempDir,
    pub handle: *mut LmHandle,
    world: String,
}

impl FfiSession {
    pub fn str(text: &str) -> LmStr {
        LmStr { ptr: text.as_ptr(), len: text.len() }
    }

    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let paths = RuntimePaths { data_dir: dir.path().join("data") };
        let runtime = Arc::new(Runtime::new(Config::default(), paths, TextureGrant::OLD_MANAGER).unwrap());
        let session = MinimapSession::with_port(FakeTable::link_without_frame(), runtime, SystemClipboard);
        let handle = Box::into_raw(Box::new(LmHandle::new(session)));
        let world = dir.path().join("world").display().to_string();
        Self { dir, handle, world }
    }

    pub fn enter(&self) {
        let info = LmWorldInfo { local_dir: Self::str(&self.world), host: Self::str(""), port: 0, seed: 0, dim: 0 };
        assert_eq!(lm_enter_world(self.handle, &info), LmStatus::Ok);
    }
}

impl Drop for FfiSession {
    fn drop(&mut self) {
        lm_destroy(self.handle);
    }
}
