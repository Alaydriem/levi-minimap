use minimap_core::services::ui::KeyName;

#[test]
fn key_names_cover_letters_digits_function_keys_and_the_zoom_keys() {
    let names: Vec<&str> = [0x4D, 0x41, 0x5A, 0x30, 0x39, 0x70, 0x7B, 0x1B, 0xBB, 0xBD, 0x07].map(KeyName::of).to_vec();

    assert_eq!(names, vec!["M", "A", "Z", "0", "9", "F1", "F12", "Esc", "+", "-", "?"]);
}
