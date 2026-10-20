use bytemuck::Zeroable;
use minimap_ffi::services::LmConvert;
use minimap_ffi::types::{LmChunkSurface, LmColumnSample, LmFrameInput};

#[test]
fn the_shim_telemetry_in_a_frame_input_reaches_the_runtime() {
    let input = LmFrameInput {
        display_width: 1920.0,
        display_height: 1080.0,
        dt: 0.016,
        cursor_x: 3.0,
        cursor_y: 4.0,
        left_down: 1,
        foreground: 1,
        right_down: 0,
        render_us: 850,
        ui_vtx: 1200,
        working_set_bytes: 1 << 30,
        private_bytes: 1 << 29,
        collect_text: 1,
    };
    let frame = LmConvert::frame_input(&input);
    assert_eq!(frame.display, (1920.0, 1080.0));
    assert!(frame.left_down && frame.foreground);
    assert_eq!((frame.render_us, frame.ui_vtx), (850, 1200));
    assert!(frame.collect_text);
    assert_eq!((frame.working_set_bytes, frame.private_bytes), (1 << 30, 1 << 29));
}

#[test]
fn ui_texture_uploads_point_at_their_own_pixels() {
    use minimap_core::structs::mesh::{UiMesh, UiTextureDelta};
    let mut mesh = UiMesh::new();
    mesh.pixels.extend_from_slice(&[1; 16]);
    mesh.pixels.extend_from_slice(&[2; 8]);
    mesh.textures.push(UiTextureDelta {
        texture: 0,
        full: true,
        width: 2,
        height: 2,
        offset: 0,
        len: 16,
        ..UiTextureDelta::default()
    });
    mesh.textures.push(UiTextureDelta {
        texture: 0,
        full: false,
        x: 1,
        width: 2,
        height: 1,
        offset: 16,
        len: 8,
        ..UiTextureDelta::default()
    });
    let mut out = Vec::new();

    LmConvert::ui_textures(&mesh, &mut out);

    assert_eq!((out[0].full, out[1].full, out[1].x, out[1].len), (1, 0, 1, 8));
    // SAFETY: the pointers point into mesh.pixels, which outlives this read
    let second = unsafe { std::slice::from_raw_parts(out[1].pixels, out[1].len) };
    assert_eq!(second, &[2; 8]);
}

#[test]
fn the_probe_buffer_carries_the_raw_biome_ids_beside_the_surface() {
    let mut raw = LmChunkSurface::zeroed();
    raw.height[5] = 70;
    raw.biome[5] = 21;
    let probe = LmConvert::probe(&raw);
    assert_eq!(probe.surface.height[5], 70);
    assert_eq!(probe.biome_ids[5], 21);
    assert!(probe.surface.biome.key(5).is_none());
}

#[test]
fn a_column_sample_carries_its_biome_id() {
    let sample = LmColumnSample { biome: 21, ..LmColumnSample::default() };
    assert_eq!(LmConvert::column(&sample).biome_id, 21);
    assert_eq!(std::mem::size_of::<LmColumnSample>(), 16);
}
