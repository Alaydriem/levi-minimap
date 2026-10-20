use minimap_ffi::services::LmBoundary;
use minimap_ffi::types::{LmStatus, LmStr};

#[test]
fn a_panic_inside_the_guard_becomes_a_status() {
    assert_eq!(LmBoundary::guard(|| panic!("boom")), LmStatus::Panicked);
    assert_eq!(LmBoundary::guard_value(7, || -> i32 { panic!("boom") }), 7);
    assert_eq!(LmBoundary::guard(|| LmStatus::Ok), LmStatus::Ok);
}

#[test]
fn text_rejects_null_with_length_and_invalid_utf8() {
    let null = LmStr { ptr: std::ptr::null(), len: 3 };
    // SAFETY: a null pointer with a non-zero length is the case under test and is never dereferenced
    assert_eq!(unsafe { LmBoundary::text(&null) }, None);
    let bad = [0xFFu8, 0xFE];
    let invalid = LmStr { ptr: bad.as_ptr(), len: 2 };
    // SAFETY: invalid points to two live bytes
    assert_eq!(unsafe { LmBoundary::text(&invalid) }, None);
    let empty = LmStr { ptr: std::ptr::null(), len: 0 };
    // SAFETY: a zero length is never dereferenced
    assert_eq!(unsafe { LmBoundary::text(&empty) }, Some(String::new()));
}
