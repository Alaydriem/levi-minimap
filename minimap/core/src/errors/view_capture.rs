#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ViewCaptureError {
    #[error("the screen format {0} cannot be exported")]
    UnsupportedFormat(u32),
    #[error("the captured image is smaller than its size says")]
    TooShort,
}
