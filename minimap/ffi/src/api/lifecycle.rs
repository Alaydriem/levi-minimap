use std::path::PathBuf;

use minimap_runtime::services::Runtime;
use minimap_runtime::structs::RuntimePaths;
use minimap_store::structs::WorldIdentity;

use crate::services::{LmBoundary, LmConvert, LmHandle, LogBridge};
use crate::types::{LmLogFn, LmPaths, LmStats, LmStatus, LmWorldInfo};

// returns null when the paths are invalid or the runtime cannot start
#[unsafe(no_mangle)]
pub extern "C" fn lm_create(paths: *const LmPaths, log: LmLogFn) -> *mut LmHandle {
    LogBridge::install(log);
    LmBoundary::guard_value(std::ptr::null_mut(), || {
        // SAFETY: the shim passes a pointer to an LmPaths it owns for the call, or null
        let Some(paths) = (unsafe { LmBoundary::input(paths) }) else { return std::ptr::null_mut() };
        // SAFETY: each LmStr in LmPaths points to bytes the shim owns for the call
        let text = |s| unsafe { LmBoundary::text(s) };
        let (Some(config), Some(data)) = (text(&paths.config_file), text(&paths.data_dir)) else {
            return std::ptr::null_mut();
        };
        let runtime_paths = RuntimePaths { config_file: PathBuf::from(config), data_dir: PathBuf::from(data) };
        match Runtime::new(runtime_paths) {
            Ok(runtime) => Box::into_raw(Box::new(LmHandle::new(runtime))),
            Err(error) => {
                log::error!("levi-minimap could not start: {error}");
                std::ptr::null_mut()
            }
        }
    })
}

// flushes the store and frees the handle; the handle must not be used again
#[unsafe(no_mangle)]
pub extern "C" fn lm_destroy(handle: *mut LmHandle) {
    let _ = LmBoundary::guard(|| {
        if handle.is_null() {
            return LmStatus::InvalidArgument;
        }
        // SAFETY: a non-null handle came from lm_create and is destroyed exactly once
        let handle = unsafe { Box::from_raw(handle) };
        handle.runtime.shutdown();
        drop(handle);
        LmStatus::Ok
    });
    LogBridge::uninstall();
}

#[unsafe(no_mangle)]
pub extern "C" fn lm_enter_world(handle: *const LmHandle, info: *const LmWorldInfo) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create; info is owned by the shim for the call
        let (Some(handle), Some(info)) = (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::input(info) })
        else {
            return LmStatus::InvalidArgument;
        };
        // SAFETY: the LmStr fields point to bytes the shim owns for the call
        let (Some(local), Some(host)) =
            (unsafe { LmBoundary::text(&info.local_dir) }, unsafe { LmBoundary::text(&info.host) })
        else {
            return LmStatus::InvalidArgument;
        };
        let identity = if local.is_empty() {
            WorldIdentity::Remote { host, port: info.port, seed: info.seed }
        } else {
            WorldIdentity::Local { world_dir: PathBuf::from(local) }
        };
        match handle.runtime.enter_world(&identity, info.dim) {
            Ok(()) => LmStatus::Ok,
            Err(error) => {
                log::error!("world not entered: {error}");
                LmStatus::Failed
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn lm_leave_world(handle: *const LmHandle) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create
        let Some(handle) = (unsafe { LmBoundary::handle(handle) }) else { return LmStatus::InvalidArgument };
        handle.runtime.leave_world();
        LmStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn lm_stats(handle: *const LmHandle, out: *mut LmStats) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create; out is writable for the call
        let (Some(handle), Some(out)) = (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::output(out) })
        else {
            return LmStatus::InvalidArgument;
        };
        *out = LmConvert::stats(handle.runtime.stats());
        LmStatus::Ok
    })
}
