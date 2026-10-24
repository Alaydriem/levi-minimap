use minimap_ffi::api::lifecycle::{lm_create, lm_destroy, lm_enter_world, lm_leave_world, lm_stats};
use minimap_ffi::types::{LmPaths, LmStats, LmStatus, LmStr};

use crate::support::FfiSession;

#[test]
fn null_arguments_are_rejected_without_a_crash() {
    let mut status = LmStatus::Ok;
    assert!(lm_create(std::ptr::null(), &mut status).is_null());
    assert_eq!(status, LmStatus::InvalidArgument);
    assert_eq!(lm_enter_world(std::ptr::null(), std::ptr::null()), LmStatus::InvalidArgument);
    assert_eq!(lm_leave_world(std::ptr::null()), LmStatus::InvalidArgument);
    assert_eq!(lm_stats(std::ptr::null(), std::ptr::null_mut()), LmStatus::InvalidArgument);
    lm_destroy(std::ptr::null_mut());
}

#[test]
fn paths_that_are_not_utf8_are_rejected() {
    let bad = [0xFFu8];
    let paths = LmPaths { config_dir: LmStr { ptr: bad.as_ptr(), len: 1 }, data_dir: FfiSession::str("x") };
    let mut status = LmStatus::Ok;
    assert!(lm_create(&paths, &mut status).is_null());
    assert_eq!(status, LmStatus::InvalidArgument);
}

#[test]
fn a_relative_config_dir_is_rejected() {
    let paths = LmPaths { config_dir: FfiSession::str("config"), data_dir: FfiSession::str("data") };
    let mut status = LmStatus::Ok;

    assert!(lm_create(&paths, &mut status).is_null());

    assert_eq!(status, LmStatus::InvalidArgument);
}

#[test]
fn lm_create_without_a_manager_returns_manager_missing_and_null() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config").display().to_string();
    let data = dir.path().join("data").display().to_string();
    let paths = LmPaths { config_dir: FfiSession::str(&config), data_dir: FfiSession::str(&data) };
    let mut status = LmStatus::Ok;

    let handle = lm_create(&paths, &mut status);

    assert!(handle.is_null());
    assert_eq!(status, LmStatus::ManagerMissing);
}

#[test]
fn entering_and_leaving_a_world_shows_in_the_stats() {
    let session = FfiSession::new();
    session.enter();
    let mut stats = LmStats::default();
    assert_eq!(lm_stats(session.handle, &mut stats), LmStatus::Ok);
    assert_eq!(stats.in_world, 1);
    assert!(stats.surface_capacity > 0);
    assert_eq!(lm_leave_world(session.handle), LmStatus::Ok);
    lm_stats(session.handle, &mut stats);
    assert_eq!(stats.in_world, 0);
}
