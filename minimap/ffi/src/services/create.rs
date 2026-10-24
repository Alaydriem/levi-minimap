use std::path::{Path, PathBuf};

use amm_core::errors::LinkError;
use minimap_link::errors::SessionError;
use minimap_link::services::MinimapSession;
use minimap_runtime::structs::RuntimePaths;

use crate::services::{LmBoundary, LmHandle};
use crate::types::{LmPaths, LmStatus};

// reads lm_create's paths and connects the session; the status says why a null handle came back
pub struct LmCreate;

impl LmCreate {
    pub fn create(paths: *const LmPaths) -> (*mut LmHandle, LmStatus) {
        // SAFETY: the shim passes a pointer to an LmPaths it owns for the call, or null
        let Some(paths) = (unsafe { LmBoundary::input(paths) }) else {
            return (std::ptr::null_mut(), LmStatus::InvalidArgument);
        };
        // SAFETY: each LmStr in LmPaths points to bytes the shim owns for the call
        let text = |s| unsafe { LmBoundary::text(s) };
        let (Some(config), Some(data)) = (text(&paths.config_dir), text(&paths.data_dir)) else {
            return (std::ptr::null_mut(), LmStatus::InvalidArgument);
        };
        if !Path::new(&config).is_absolute() {
            return (std::ptr::null_mut(), LmStatus::InvalidArgument);
        }
        let runtime_paths = RuntimePaths { data_dir: PathBuf::from(data) };
        match MinimapSession::connect(Path::new(&config), runtime_paths) {
            Ok(session) => (Box::into_raw(Box::new(LmHandle::new(session))), LmStatus::Ok),
            Err(error) => (std::ptr::null_mut(), Self::status(&error)),
        }
    }

    fn status(error: &SessionError) -> LmStatus {
        match error {
            SessionError::Link(LinkError::ManagerMissing) => LmStatus::ManagerMissing,
            SessionError::Link(_) => {
                curia::error!("the mod manager refused levi-minimap", { error: error.to_string() });
                LmStatus::RegisterFailed
            }
            SessionError::Runtime(_) | SessionError::Folder(_) => {
                curia::error!("levi-minimap could not start", { error: error.to_string() });
                LmStatus::Failed
            }
        }
    }
}
