use minimap_core::errors::ViewCaptureError;
use minimap_core::services::export::ViewImage;

// 2 x 2 pixels, each row padded to 12 bytes
const PADDED: [u8; 24] = [1, 2, 3, 0, 4, 5, 6, 0, 99, 99, 99, 99, 7, 8, 9, 0, 10, 11, 12, 0, 99, 99, 99, 99];

#[test]
fn a_bgra_capture_becomes_rgba_with_full_alpha_and_no_row_padding() {
    let rgba = ViewImage::from_capture(&PADDED, 2, 2, 12, ViewImage::B8G8R8A8_UNORM).unwrap();
    assert_eq!(rgba, vec![3, 2, 1, 255, 6, 5, 4, 255, 9, 8, 7, 255, 12, 11, 10, 255]);
}

#[test]
fn an_rgba_capture_keeps_its_channel_order() {
    let rgba = ViewImage::from_capture(&PADDED, 2, 2, 12, ViewImage::R8G8B8A8_UNORM_SRGB).unwrap();
    assert_eq!(&rgba[..4], &[1, 2, 3, 255]);
}

#[test]
fn other_formats_and_short_buffers_are_refused() {
    assert_eq!(ViewImage::from_capture(&PADDED, 2, 2, 12, 24), Err(ViewCaptureError::UnsupportedFormat(24)));
    assert_eq!(ViewImage::from_capture(&PADDED[..19], 2, 2, 12, 87), Err(ViewCaptureError::TooShort));
    assert_eq!(ViewImage::from_capture(&PADDED, 2, 2, 4, 87), Err(ViewCaptureError::TooShort));
    assert_eq!(ViewImage::from_capture(&PADDED, 2, 0, 12, 87), Err(ViewCaptureError::TooShort));
}
