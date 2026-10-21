#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LmStatus {
    Ok = 0,
    InvalidArgument = 1,
    Panicked = 2,
    NotInWorld = 3,
    Failed = 4,
    // alaydriem-mod-manager is not loaded
    ManagerMissing = 5,
    // the manager refused the registration
    RegisterFailed = 6,
}
