use minimap_core::structs::mesh::{UiDraw, UiMesh, UiTextRect, UiVertex};

fn with_texts(texts: &[(&str, [f32; 4])]) -> UiMesh {
    let mut mesh = UiMesh::new();
    for (text, rect) in texts {
        let start = mesh.texts.len() as u32;
        mesh.texts.push_str(text);
        mesh.text_rects.push(UiTextRect { start, len: text.len() as u32, rect: *rect });
    }
    mesh
}

#[test]
fn clear_empties_every_buffer_and_keeps_its_capacity() {
    let mut mesh = with_texts(&[("Base", [0.0; 4])]);
    mesh.vertices.resize(64, UiVertex::default());
    mesh.draws.resize(8, UiDraw::default());
    mesh.pixels.resize(1024, 0);
    mesh.clipboard.push_str("1, ~, 2");
    let capacities = (mesh.vertices.capacity(), mesh.draws.capacity(), mesh.pixels.capacity());

    mesh.clear();

    assert!(mesh.vertices.is_empty() && mesh.draws.is_empty() && mesh.pixels.is_empty());
    assert!(mesh.clipboard.is_empty() && mesh.text_rects.is_empty());
    assert_eq!((mesh.vertices.capacity(), mesh.draws.capacity(), mesh.pixels.capacity()), capacities);
}

#[test]
fn text_rect_finds_the_first_exact_match_and_prefixes_match_any() {
    let mesh = with_texts(&[("Base camp", [1.0, 1.0, 2.0, 2.0]), ("Base", [5.0, 5.0, 9.0, 9.0]), ("Base", [7.0; 4])]);

    assert_eq!(mesh.text_rect("Base"), Some([5.0, 5.0, 9.0, 9.0]));
    assert!(mesh.text_starting("Base c"));
    assert!(!mesh.draws_text("Base c"));
    assert_eq!(mesh.text_rect("Village"), None);
}
