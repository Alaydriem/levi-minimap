#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LmStatus {
    Ok = 0,
    InvalidArgument = 1,
    Panicked = 2,
    NotInWorld = 3,
    Failed = 4,
}
