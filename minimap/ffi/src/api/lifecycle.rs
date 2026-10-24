use std::path::PathBuf;

use minimap_store::structs::WorldIdentity;

use crate::services::{LmBoundary, LmConvert, LmCreate, LmHandle, LmLog};
use crate::types::{LmPaths, LmStats, LmStatus, LmStr, LmWorldInfo};

// registers with the mod manager and starts the runtime; null with `status` set when it cannot. Call it from the
// mod's enable(), never from load(): it starts threads
#[unsafe(no_mangle)]
pub extern "C" fn lm_create(paths: *const LmPaths, status: *mut LmStatus) -> *mut LmHandle {
    let mut result = LmStatus::Panicked;
    let handle = LmBoundary::guard_value(std::ptr::null_mut(), || {
        let (handle, outcome) = LmCreate::create(paths);
        result = outcome;
        handle
    });
    // SAFETY: the shim passes a writable LmStatus for the call, or null
    if let Some(status) = unsafe { LmBoundary::output(status) } {
        *status = result;
    }
    handle
}

// the shim unhooks first, so no tick runs; the session gives its leases back and unregisters, then the runtime
// stops; the handle must not be used again
#[unsafe(no_mangle)]
pub extern "C" fn lm_destroy(handle: *mut LmHandle) {
    let _ = LmBoundary::guard(|| {
        if handle.is_null() {
            return LmStatus::InvalidArgument;
        }
        // SAFETY: a non-null handle came from lm_create and is destroyed exactly once
        let handle = unsafe { Box::from_raw(handle) };
        let session = handle.session.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
        match session {
            Some(session) => session.shutdown(),
            None => handle.runtime.shutdown(),
        }
        LmStatus::Ok
    });
}

// one line from the shim into the mod's log; level 1 error, 2 warn, 3 info, else debug
#[unsafe(no_mangle)]
pub extern "C" fn lm_log(level: u8, message: LmStr) {
    let _ = LmBoundary::guard(|| {
        // SAFETY: the shim passes the bytes of a string it owns for the call
        let Some(message) = (unsafe { LmBoundary::text(&message) }) else { return LmStatus::InvalidArgument };
        LmLog::write(level, &message);
        LmStatus::Ok
    });
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
                curia::error!("world not entered", { error: error.to_string() });
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
