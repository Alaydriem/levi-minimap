use bytemuck::Zeroable;
use minimap_ffi::services::LmConvert;
use minimap_ffi::types::{LmChunkSurface, LmColumnSample};

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
