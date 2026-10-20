use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::services::LmHandle;
use crate::types::{LmStatus, LmStr};

// the rules every entry point follows: no panic crosses into C++, every pointer is checked
pub struct LmBoundary;

impl LmBoundary {
    // runs the body; a panic becomes LmStatus::Panicked and a log line
    pub fn guard(body: impl FnOnce() -> LmStatus) -> LmStatus {
        match catch_unwind(AssertUnwindSafe(body)) {
            Ok(status) => status,
            Err(payload) => {
                let message = payload
                    .downcast_ref::<&str>()
                    .map(|s| (*s).to_string())
                    .or_else(|| payload.downcast_ref::<String>().cloned())
                    .unwrap_or_else(|| "unknown panic".to_string());
                log::error!("panic stopped at the C boundary: {message}");
                LmStatus::Panicked
            }
        }
    }

    // like guard, for entry points that return a value; `fallback` is returned on panic
    pub fn guard_value<T>(fallback: T, body: impl FnOnce() -> T) -> T {
        match catch_unwind(AssertUnwindSafe(body)) {
            Ok(value) => value,
            Err(_) => {
                log::error!("panic stopped at the C boundary");
                fallback
            }
        }
    }

    // SAFETY: the caller passes either null or a pointer returned by lm_create and not yet destroyed
    pub unsafe fn handle<'a>(handle: *const LmHandle) -> Option<&'a LmHandle> {
        // SAFETY: see the function contract
        unsafe { handle.as_ref() }
    }

    // SAFETY: the caller passes either null or a pointer to a live, aligned T for the call
    pub unsafe fn input<'a, T>(value: *const T) -> Option<&'a T> {
        // SAFETY: see the function contract
        unsafe { value.as_ref() }
    }

    // SAFETY: the caller passes either null or a pointer to a live, aligned, writable T for the call
    pub unsafe fn output<'a, T>(value: *mut T) -> Option<&'a mut T> {
        // SAFETY: see the function contract
        unsafe { value.as_mut() }
    }

    // SAFETY: the caller passes either null or a pointer to len readable bytes for the call
    pub unsafe fn bytes<'a>(ptr: *const u8, len: usize) -> Option<&'a [u8]> {
        if ptr.is_null() {
            return None;
        }
        // SAFETY: see the function contract
        Some(unsafe { std::slice::from_raw_parts(ptr, len) })
    }

    // SAFETY: ptr is null with len 0, or points to len readable bytes for the call
    pub unsafe fn text(value: &LmStr) -> Option<String> {
        if value.len == 0 {
            return Some(String::new());
        }
        if value.ptr.is_null() {
            return None;
        }
        // SAFETY: see the function contract
        let bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.len) };
        std::str::from_utf8(bytes).ok().map(str::to_string)
    }
}
