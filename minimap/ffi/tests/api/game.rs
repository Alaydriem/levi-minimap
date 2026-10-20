use minimap_ffi::api::game::{
    lm_biome_name, lm_block_changed, lm_key, lm_mouse, lm_staging, lm_submit_chunk, lm_submit_column, lm_text, lm_tick,
};
use minimap_ffi::api::lifecycle::lm_stats;
use minimap_ffi::types::{
    LmChunkPos, LmColumnSample, LmKeyResult, LmProbeRequest, LmScanPlan, LmStats, LmStatus, LmStr, LmTickInput,
};

use crate::support::FfiSession;

fn tick_input() -> LmTickInput {
    LmTickInput { present: 1, foreground: 1, cursor_hidden: 1, dead: 0, x: 8.5, y: 70.0, z: 8.5, yaw: 0.0, dim: 0 }
}

fn empty_plan() -> LmScanPlan {
    LmScanPlan {
        candidates: std::ptr::null(),
        count: 0,
        limit: 0,
        camera_height: 0,
        transparent_water: 0,
        cursor: 0,
        block_colors: std::ptr::null(),
        block_color_count: 0,
        biome_requests: std::ptr::null(),
        biome_request_count: 0,
    }
}

fn biome_requests(plan: &LmScanPlan) -> Vec<u16> {
    if plan.biome_request_count == 0 {
        return Vec::new();
    }
    // SAFETY: lm_tick filled biome_requests with biome_request_count IDs valid until the next lm_tick
    unsafe { std::slice::from_raw_parts(plan.biome_requests, plan.biome_request_count as usize) }.to_vec()
}

// probes every candidate as flat ground through the staging buffer
fn probe_all(session: &FfiSession, plan: &LmScanPlan) {
    // SAFETY: lm_tick filled candidates with count positions that stay valid until the next lm_tick
    let candidates = unsafe { std::slice::from_raw_parts(plan.candidates, plan.count as usize) };
    for (i, pos) in candidates.iter().enumerate() {
        if i >= plan.limit as usize {
            assert_eq!(lm_submit_chunk(session.handle, pos, 3), LmStatus::Ok);
            continue;
        }
        let staging = lm_staging(session.handle);
        // SAFETY: lm_staging returns a live, aligned, zeroed surface owned by the handle
        let surface = unsafe { &mut *staging };
        for c in 0..256 {
            surface.surface[c] = 0xFF40_8040;
            surface.height[c] = 64;
            surface.solid[c] = 64;
            surface.biome[c] = 1;
        }
        assert_eq!(lm_submit_chunk(session.handle, pos, 0), LmStatus::Ok);
    }
}

#[test]
fn a_tick_returns_a_plan_and_probes_reach_the_chunk_store() {
    let session = FfiSession::new();
    session.enter();
    let mut plan = empty_plan();
    assert_eq!(lm_tick(session.handle, &tick_input(), &mut plan), LmStatus::Ok);
    assert!(plan.count > 0);
    assert_eq!(plan.limit, 32);
    assert_eq!(plan.camera_height, 320);
    assert_eq!(plan.cursor, 1);
    probe_all(&session, &plan);
    let mut stats = LmStats::default();
    lm_stats(session.handle, &mut stats);
    assert_eq!(stats.chunks_submitted, u64::from(plan.count.min(plan.limit)));
    assert!(stats.surfaces_resident > 0);
}

#[test]
fn block_changes_ask_for_a_column_of_a_known_chunk() {
    let session = FfiSession::new();
    session.enter();
    let mut plan = empty_plan();
    lm_tick(session.handle, &tick_input(), &mut plan);
    probe_all(&session, &plan);
    let mut request = LmProbeRequest::default();
    assert_eq!(lm_block_changed(session.handle, 5, 6, 0, &mut request), LmStatus::Ok);
    assert_eq!(request, LmProbeRequest { kind: 1, reserved: [0; 3], x: 0, z: 0, dim: 0, index: 6 * 16 + 5 });
    let sample = LmColumnSample { surface: 1, water: 0, height: 90, solid: 90, retry: 0, reserved: 0, biome: 0 };
    let pos = LmChunkPos { x: 0, z: 0, dim: 0 };
    assert_eq!(lm_submit_column(session.handle, &pos, request.index, &sample), LmStatus::Ok);
    assert_eq!(lm_submit_column(session.handle, &pos, 256, &sample), LmStatus::InvalidArgument);
}

#[test]
fn a_bad_submit_status_is_rejected() {
    let session = FfiSession::new();
    session.enter();
    let pos = LmChunkPos { x: 0, z: 0, dim: 0 };
    assert_eq!(lm_submit_chunk(session.handle, &pos, 9), LmStatus::InvalidArgument);
}

#[test]
fn keys_and_the_wheel_go_through_the_input_rules() {
    let session = FfiSession::new();
    session.enter();
    let mut result = LmKeyResult::default();
    assert_eq!(lm_key(session.handle, 0x4D, 1, 0, 1, &mut result), LmStatus::Ok);
    assert_eq!(result, LmKeyResult { consume: 1, action: 1 });
    assert_eq!(lm_mouse(session.handle, 4, 1), 1);
    assert_eq!(lm_mouse(std::ptr::null(), 4, 1), 0);
}

#[test]
fn the_scan_plan_carries_the_block_colour_overrides_for_the_probe() {
    let session = FfiSession::new();
    session.enter();
    let mut plan = empty_plan();
    assert_eq!(lm_tick(session.handle, &tick_input(), &mut plan), LmStatus::Ok);
    assert_eq!(plan.block_color_count, 1);
    // SAFETY: lm_tick filled block_colors with block_color_count entries valid until the next lm_tick
    let colours = unsafe { std::slice::from_raw_parts(plan.block_colors, plan.block_color_count as usize) };
    // SAFETY: each name points to UTF-8 bytes owned by the handle
    let name = unsafe { std::slice::from_raw_parts(colours[0].name.ptr, colours[0].name.len) };
    assert_eq!(name, b"minecraft:lava");
    assert_eq!(colours[0].colour, u32::from_le_bytes([230, 110, 20, 255]));
}

#[test]
fn typed_text_needs_a_live_handle() {
    assert_eq!(lm_text(std::ptr::null(), u32::from('a')), LmStatus::InvalidArgument);
    let session = FfiSession::new();
    session.enter();
    assert_eq!(lm_text(session.handle, u32::from('a')), LmStatus::Ok);
}

#[test]
fn the_plan_lists_biome_ids_to_name_and_lm_biome_name_answers_them() {
    let session = FfiSession::new();
    session.enter();
    let mut plan = empty_plan();
    assert_eq!(lm_tick(session.handle, &tick_input(), &mut plan), LmStatus::Ok);
    probe_all(&session, &plan);
    assert_eq!(lm_tick(session.handle, &tick_input(), &mut plan), LmStatus::Ok);
    assert_eq!(biome_requests(&plan), vec![1]);
    assert_eq!(lm_biome_name(session.handle, 1, FfiSession::str("plains")), LmStatus::Ok);
    probe_all(&session, &plan);
    assert_eq!(lm_tick(session.handle, &tick_input(), &mut plan), LmStatus::Ok);
    assert!(biome_requests(&plan).is_empty());
}

#[test]
fn lm_biome_name_needs_a_live_handle_and_utf8() {
    let session = FfiSession::new();
    session.enter();
    assert_eq!(lm_biome_name(std::ptr::null(), 1, FfiSession::str("plains")), LmStatus::InvalidArgument);
    let bad = [0xFFu8, 0xFE];
    assert_eq!(lm_biome_name(session.handle, 1, LmStr { ptr: bad.as_ptr(), len: 2 }), LmStatus::InvalidArgument);
    assert_eq!(lm_biome_name(session.handle, 2, FfiSession::str("")), LmStatus::Ok);
}
