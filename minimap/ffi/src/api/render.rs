use crate::services::{LmBoundary, LmConvert, LmHandle};
use minimap_core::structs::mesh::{UiDraw, UiVertex};

use crate::types::{LmFrame, LmFrameInput, LmStatus, LmUiDraw, LmUiVertex};

// the C mirrors are cast from the core buffers without a copy, so their layouts must agree
const _: () = assert!(size_of::<LmUiVertex>() == size_of::<UiVertex>() && size_of::<LmUiDraw>() == size_of::<UiDraw>());

// the frame's pointers stay valid until the next lm_frame
#[unsafe(no_mangle)]
pub extern "C" fn lm_frame(handle: *const LmHandle, input: *const LmFrameInput, out: *mut LmFrame) -> LmStatus {
    LmBoundary::guard(|| {
        // SAFETY: the handle came from lm_create; input and out are owned by the shim for the call
        let (Some(handle), Some(input), Some(out)) =
            (unsafe { LmBoundary::handle(handle) }, unsafe { LmBoundary::input(input) }, unsafe {
                LmBoundary::output(out)
            })
        else {
            return LmStatus::InvalidArgument;
        };
        let mut buffers = handle.render.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let buffers = &mut *buffers;
        handle.runtime.frame(&LmConvert::frame_input(input), &mut buffers.output);
        LmConvert::textures(&buffers.output, &mut buffers.textures);
        LmConvert::ui_textures(&buffers.output.mesh, &mut buffers.ui_textures);
        LmConvert::ui_texts(&buffers.output.mesh, &mut buffers.texts);
        let mesh = &buffers.output.mesh;
        let vertices: &[LmUiVertex] = bytemuck::cast_slice(&mesh.vertices);
        let draws: &[LmUiDraw] = bytemuck::cast_slice(&mesh.draws);
        *out = LmFrame {
            textures: buffers.textures.as_ptr(),
            texture_count: buffers.textures.len() as u32,
            vertices: vertices.as_ptr(),
            vertex_count: vertices.len() as u32,
            indices: mesh.indices.as_ptr(),
            index_count: mesh.indices.len() as u32,
            draws: draws.as_ptr(),
            draw_count: draws.len() as u32,
            ui_textures: buffers.ui_textures.as_ptr(),
            ui_texture_count: buffers.ui_textures.len() as u32,
            ui_freed: mesh.freed.as_ptr(),
            ui_freed_count: mesh.freed.len() as u32,
            clipboard: mesh.clipboard.as_ptr(),
            clipboard_len: mesh.clipboard.len() as u32,
            texts: buffers.texts.as_ptr(),
            text_count: buffers.texts.len() as u32,
            text_bytes: mesh.texts.as_ptr(),
            text_bytes_len: mesh.texts.len() as u32,
            cursor_flags: buffers.output.cursor_flags,
        };
        LmStatus::Ok
    })
}

// the back buffer of a frame that set CAPTURE_VIEW: `height` rows `stride` bytes apart, each `width`
// pixels of 4 bytes in DXGI `format`. The pixels are copied before this returns
#[unsafe(no_mangle)]
pub extern "C" fn lm_view_captured(
    handle: *const LmHandle,
    pixels: *const u8,
    width: u32,
    height: u32,
    stride: u32,
    format: u32,
) -> LmStatus {
    LmBoundary::guard(|| {
        let len = (height as usize)
            .checked_sub(1)
            .and_then(|rows| rows.checked_mul(stride as usize))
            .and_then(|bytes| bytes.checked_add(width as usize * 4));
        // SAFETY: the handle came from lm_create
        let (Some(handle), Some(len)) = (unsafe { LmBoundary::handle(handle) }, len) else {
            return LmStatus::InvalidArgument;
        };
        // SAFETY: the shim maps the back buffer copy and keeps it mapped for the call; len bytes are readable
        let Some(pixels) = (unsafe { LmBoundary::bytes(pixels, len) }) else { return LmStatus::InvalidArgument };
        handle.runtime.view_captured(pixels, (width, height), stride, format);
        LmStatus::Ok
    })
}
