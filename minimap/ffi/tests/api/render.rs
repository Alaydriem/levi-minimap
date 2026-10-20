use minimap_ffi::api::game::lm_tick;
use minimap_ffi::api::render::{lm_frame, lm_view_captured};
use minimap_ffi::types::{LmFrame, LmFrameInput, LmScanPlan, LmStatus, LmTickInput};

use crate::support::FfiSession;

fn frame_input() -> LmFrameInput {
    LmFrameInput {
        display_width: 1280.0,
        display_height: 720.0,
        dt: 0.016,
        cursor_x: 0.0,
        cursor_y: 0.0,
        left_down: 0,
        foreground: 1,
        ..LmFrameInput::default()
    }
}

fn empty_frame() -> LmFrame {
    LmFrame::empty()
}

fn in_world(session: &FfiSession) {
    session.enter();
    let input =
        LmTickInput { present: 1, foreground: 1, cursor_hidden: 1, dead: 0, x: 1.5, y: 70.0, z: 2.5, yaw: 0.0, dim: 0 };
    let mut plan = LmScanPlan {
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
    };
    lm_tick(session.handle, &input, &mut plan);
}

// the texts of a frame that asked for them
fn texts(frame: &LmFrame) -> Vec<String> {
    if frame.text_count == 0 {
        return Vec::new();
    }
    // SAFETY: lm_frame filled texts and text_bytes, valid until the next lm_frame
    let (texts, bytes) = unsafe {
        (
            std::slice::from_raw_parts(frame.texts, frame.text_count as usize),
            std::slice::from_raw_parts(frame.text_bytes, frame.text_bytes_len as usize),
        )
    };
    texts
        .iter()
        .map(|t| String::from_utf8_lossy(&bytes[t.start as usize..(t.start + t.len) as usize]).into_owned())
        .collect()
}

#[test]
fn a_frame_before_any_player_draws_nothing_and_null_arguments_are_refused() {
    let session = FfiSession::new();
    let mut frame = empty_frame();
    assert_eq!(lm_frame(session.handle, &frame_input(), &mut frame), LmStatus::Ok);
    assert_eq!((frame.vertex_count, frame.draw_count), (0, 0));
    assert_eq!(lm_frame(std::ptr::null(), &frame_input(), &mut frame), LmStatus::InvalidArgument);
    assert_eq!(lm_frame(session.handle, std::ptr::null(), &mut frame), LmStatus::InvalidArgument);
    assert_eq!(lm_frame(session.handle, &frame_input(), std::ptr::null_mut()), LmStatus::InvalidArgument);
}

#[test]
fn a_frame_with_a_player_draws_the_minimap_texture_and_its_coordinates() {
    let session = FfiSession::new();
    in_world(&session);
    let mut frame = empty_frame();
    let input = LmFrameInput { collect_text: 1, ..frame_input() };

    assert_eq!(lm_frame(session.handle, &input, &mut frame), LmStatus::Ok);

    assert!(frame.vertex_count > 0 && frame.texture_count >= 1 && frame.ui_texture_count >= 1);
    // SAFETY: lm_frame filled the arrays, valid until the next lm_frame
    let (draws, indices, deltas) = unsafe {
        (
            std::slice::from_raw_parts(frame.draws, frame.draw_count as usize),
            std::slice::from_raw_parts(frame.indices, frame.index_count as usize),
            std::slice::from_raw_parts(frame.ui_textures, frame.ui_texture_count as usize),
        )
    };
    assert!(draws.iter().any(|d| d.kind == 1 && d.texture == 1));
    assert!(draws.iter().all(|d| {
        let range = d.index_offset as usize..(d.index_offset + d.index_count) as usize;
        indices[range].iter().all(|&i| d.vertex_offset + i < frame.vertex_count)
    }));
    assert!(deltas[0].full == 1 && deltas[0].len == (deltas[0].width * deltas[0].height * 4) as usize);
    let labels = texts(&frame);
    assert!(labels.iter().any(|t| t == "1, 70, 2"), "{labels:?}");
    assert!(labels.iter().any(|t| t == "Nether 0, 0"), "{labels:?}");
}

#[test]
fn texts_are_listed_only_when_the_frame_asks_for_them() {
    let session = FfiSession::new();
    in_world(&session);
    let mut frame = empty_frame();

    assert_eq!(lm_frame(session.handle, &frame_input(), &mut frame), LmStatus::Ok);

    assert_eq!((frame.text_count, frame.text_bytes_len), (0, 0));
    assert!(frame.vertex_count > 0);
}

#[test]
fn a_view_capture_needs_a_handle_pixels_and_a_size_that_fits() {
    let pixels = [0u8; 16];
    assert_eq!(lm_view_captured(std::ptr::null(), pixels.as_ptr(), 2, 2, 8, 87), LmStatus::InvalidArgument);
    let session = FfiSession::new();
    session.enter();
    assert_eq!(lm_view_captured(session.handle, std::ptr::null(), 2, 2, 8, 87), LmStatus::InvalidArgument);
    assert_eq!(lm_view_captured(session.handle, pixels.as_ptr(), 2, 0, 8, 87), LmStatus::InvalidArgument);
    assert_eq!(lm_view_captured(session.handle, pixels.as_ptr(), 2, 2, 8, 87), LmStatus::Ok);
}
