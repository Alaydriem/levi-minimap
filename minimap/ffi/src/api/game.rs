use bytemuck::Zeroable;
use minimap_runtime::structs::SubmitStatus;

use crate::services::{LmBoundary, LmConvert, LmHandle};
use crate::types::{
    LmChunkPos, LmChunkSurface, LmColumnSample, LmProbeRequest, LmScanPlan, LmStatus, LmStr, LmTickInput,
};

#[unsafe(no_mangle)]
pub extern "C" fn lm_tick(handle: *const LmHandle, input: *const LmTickInput, plan: *mut LmScanPlan) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create; input and plan are owned by the shim for the call
        let (Some(handle), Some(input), Some(plan)) =
            (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::input(input) }, unsafe {
                LmBoundary::output(plan)
            })
        else {
            return LmStatus::InvalidArgument;
        };
        let mut buffers = handle.game.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let buffers = &mut *buffers;
        let tick = LmConvert::tick_input(input);
        match handle.session.lock().unwrap_or_else(std::sync::PoisonError::into_inner).as_mut() {
            Some(session) => session.tick(&tick, &mut buffers.plan),
            None => return LmStatus::Failed,
        }
        buffers.candidates.clear();
        buffers.candidates.extend(buffers.plan.candidates.iter().map(|p| LmConvert::lm_chunk_pos(*p)));
        buffers.block_colors.clear();
        buffers.block_colors.extend(handle.runtime.block_colors().iter().map(LmConvert::block_color));
        *plan = LmScanPlan {
            candidates: buffers.candidates.as_ptr(),
            count: buffers.candidates.len() as u32,
            limit: buffers.plan.limit as u32,
            camera_height: buffers.plan.camera_height,
            transparent_water: u8::from(buffers.plan.transparent_water),
            cursor: LmConvert::cursor(buffers.plan.cursor),
            block_colors: buffers.block_colors.as_ptr(),
            block_color_count: buffers.block_colors.len() as u32,
            biome_requests: buffers.plan.biome_requests.as_ptr(),
            biome_request_count: buffers.plan.biome_requests.len() as u32,
        };
        LmStatus::Ok
    })
}

// a zeroed surface for the probe to fill; valid until the next lm_staging or lm_submit_chunk
#[unsafe(no_mangle)]
pub extern "C" fn lm_staging(handle: *const LmHandle) -> *mut LmChunkSurface {
    LmBoundary::guard_value(std::ptr::null_mut(), || {
        // SAFETY: the handle came from lm_create
        let Some(handle) = (unsafe { LmBoundary::handle(handle) }) else { return std::ptr::null_mut() };
        let mut buffers = handle.game.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        *buffers.staging = LmChunkSurface::zeroed();
        &mut *buffers.staging as *mut LmChunkSurface
    })
}

// status: 0 loaded (the staging surface holds the probe), 1 not loaded, 2 top sub-chunk placeholder, 3 not probed
#[unsafe(no_mangle)]
pub extern "C" fn lm_submit_chunk(handle: *const LmHandle, pos: *const LmChunkPos, status: u8) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create; pos is owned by the shim for the call
        let (Some(handle), Some(pos)) = (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::input(pos) })
        else {
            return LmStatus::InvalidArgument;
        };
        let Some(status) = SubmitStatus::from_code(status) else { return LmStatus::InvalidArgument };
        let probe = {
            let buffers = handle.game.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            LmConvert::probe(&buffers.staging)
        };
        handle.runtime.submit_chunk(LmConvert::chunk_pos(pos), status, &probe);
        LmStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn lm_block_changed(
    handle: *const LmHandle,
    x: i32,
    z: i32,
    dim: i32,
    out: *mut LmProbeRequest,
) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create; out is writable for the call
        let (Some(handle), Some(out)) = (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::output(out) })
        else {
            return LmStatus::InvalidArgument;
        };
        *out = LmConvert::probe_request(handle.runtime.block_changed(x, z, dim));
        LmStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn lm_submit_column(
    handle: *const LmHandle,
    pos: *const LmChunkPos,
    index: u32,
    sample: *const LmColumnSample,
) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create; pos and sample are owned by the shim for the call
        let (Some(handle), Some(pos), Some(sample)) =
            (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::input(pos) }, unsafe {
                LmBoundary::input(sample)
            })
        else {
            return LmStatus::InvalidArgument;
        };
        if index >= 256 {
            return LmStatus::InvalidArgument;
        }
        handle.runtime.submit_column(LmConvert::chunk_pos(pos), index as usize, LmConvert::column(sample));
        LmStatus::Ok
    })
}

// returns `fallback` when the handle or name is invalid
#[unsafe(no_mangle)]
pub extern "C" fn lm_resolve_void_color(handle: *const LmHandle, biome: LmStr, fallback: u32) -> u32 {
    LmBoundary::guard_value(fallback, || {
        // SAFETY: the handle came from lm_create; biome points to bytes the shim owns for the call
        let (Some(handle), Some(name)) = (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::text(&biome) })
        else {
            return fallback;
        };
        handle.runtime.resolve_void_color(&name, fallback)
    })
}

// the game's internal name for a biome ID the plan asked about; an empty name means the game has none
#[unsafe(no_mangle)]
pub extern "C" fn lm_biome_name(handle: *const LmHandle, id: u16, name: LmStr) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create; name points to bytes the shim owns for the call
        let (Some(handle), Some(name)) = (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::text(&name) })
        else {
            return LmStatus::InvalidArgument;
        };
        handle.runtime.biome_name(id, &name);
        LmStatus::Ok
    })
}
