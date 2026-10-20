use std::fs;

use minimap_store::services::ConfigFile;

#[test]
fn a_missing_config_reads_as_none() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(ConfigFile::read(&dir.path().join("config.json")).unwrap(), None);
}

#[test]
fn replacing_an_invalid_config_keeps_a_copy_of_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    fs::write(&path, "{ broken").unwrap();
    ConfigFile::write(&path, "{}", true).unwrap();
    assert_eq!(ConfigFile::read(&path).unwrap().as_deref(), Some("{}"));
    assert_eq!(fs::read_to_string(dir.path().join("config.json.invalid")).unwrap(), "{ broken");
}

#[test]
fn a_normal_write_keeps_no_copy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("config.json");
    ConfigFile::write(&path, "{\"a\":1}", false).unwrap();
    ConfigFile::write(&path, "{\"a\":2}", false).unwrap();
    assert_eq!(ConfigFile::read(&path).unwrap().as_deref(), Some("{\"a\":2}"));
    assert!(!path.with_extension("json.invalid").exists());
}
