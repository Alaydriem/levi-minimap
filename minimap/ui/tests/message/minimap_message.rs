use std::path::PathBuf;

use levi_i18n::message::Message;
use levi_i18n_tool::pot::PotWriter;
use minimap_ui::message::MinimapMessage;

#[test]
fn every_message_has_a_dense_index() {
    for (position, message) in MinimapMessage::ALL.iter().enumerate() {
        assert_eq!(message.index(), position, "{message:?}");
    }
}

#[test]
fn pot_is_current() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("i18n").join(format!("{}.pot", MinimapMessage::DOMAIN.name()));
    let stale = PotWriter::write::<MinimapMessage>(&path, true).unwrap_or_else(|error| panic!("{error}"));
    assert!(!stale, "{} is out of date: run `mise run i18n:extract`", path.display());
}
