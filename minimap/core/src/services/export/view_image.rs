use crate::errors::ViewCaptureError;

// a captured back buffer turned into tight RGBA rows with full alpha
pub struct ViewImage;

impl ViewImage {
    pub const R8G8B8A8_UNORM: u32 = 28;
    pub const R8G8B8A8_UNORM_SRGB: u32 = 29;
    pub const B8G8R8A8_UNORM: u32 = 87;
    pub const B8G8R8A8_UNORM_SRGB: u32 = 91;

    // format is the DXGI format of the back buffer; stride is the bytes between row starts
    pub fn from_capture(
        pixels: &[u8],
        width: usize,
        height: usize,
        stride: usize,
        format: u32,
    ) -> Result<Vec<u8>, ViewCaptureError> {
        let swap = match format {
            Self::R8G8B8A8_UNORM | Self::R8G8B8A8_UNORM_SRGB => false,
            Self::B8G8R8A8_UNORM | Self::B8G8R8A8_UNORM_SRGB => true,
            other => return Err(ViewCaptureError::UnsupportedFormat(other)),
        };
        let row = width * 4;
        if height == 0 || stride < row || pixels.len() < stride * (height - 1) + row {
            return Err(ViewCaptureError::TooShort);
        }
        let mut out = Vec::with_capacity(row * height);
        for y in 0..height {
            for px in pixels[y * stride..y * stride + row].as_chunks::<4>().0 {
                let (r, b) = if swap { (px[2], px[0]) } else { (px[0], px[2]) };
                out.extend_from_slice(&[r, px[1], b, 255]);
            }
        }
        Ok(out)
    }
}
