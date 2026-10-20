use egui::{Area, Context, Id, pos2};
use minimap_core::structs::draw::DrawList;
use minimap_core::structs::mesh::UiMesh;
use minimap_core::structs::ui::UiFrameInput;
use minimap_ui::runtime::UiRuntime;
use minimap_ui::theme::FontSource;

use crate::support::Frames;

fn label(ctx: &Context) {
    Area::new(Id::unique("label")).fixed_pos(pos2(10.0, 10.0)).show(ctx, |ui| {
        ui.label("Plains");
    });
}

// two passes: a new area is laid out invisibly in its first
fn run_label(runtime: &mut UiRuntime, input: &UiFrameInput<'_>, out: &mut UiMesh) {
    let layers = DrawList::new();
    runtime.run_with(input, &layers, label, out);
    runtime.run_with(input, &layers, label, out);
}

#[test]
fn first_frame_uploads_the_font_texture_in_full() {
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();

    runtime.run_with(&Frames::input((0.0, 0.0), false, &[]), &DrawList::new(), label, &mut out);

    let first = out.textures.first().copied().unwrap_or_default();
    assert!(first.full);
    assert_eq!(first.len, (first.width * first.height * 4) as usize);
    assert!(first.offset + first.len <= out.pixels.len());
}

#[test]
fn an_unchanged_frame_has_no_texture_delta() {
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();
    let input = Frames::input((0.0, 0.0), false, &[]);
    run_label(&mut runtime, &input, &mut out);

    runtime.run_with(&input, &DrawList::new(), label, &mut out);

    assert!(out.textures.is_empty());
    assert!(out.pixels.is_empty());
}

#[test]
fn every_index_is_inside_its_vertex_range_and_every_clip_inside_the_display() {
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();

    run_label(&mut runtime, &Frames::input((0.0, 0.0), false, &[]), &mut out);

    assert!(!out.draws.is_empty());
    let total: u32 = out.draws.iter().map(|d| d.index_count).sum();
    assert_eq!(total as usize, out.indices.len());
    for draw in &out.draws {
        let range = draw.index_offset as usize..(draw.index_offset + draw.index_count) as usize;
        let max = out.indices[range].iter().copied().max().unwrap_or(0);
        assert!(((draw.vertex_offset + max) as usize) < out.vertices.len());
        let [x0, y0, x1, y1] = draw.clip;
        assert!(x0 >= 0.0 && y0 >= 0.0 && x1 <= Frames::DISPLAY.0 && y1 <= Frames::DISPLAY.1);
    }
}

#[test]
fn pixels_per_point_scales_panel_vertices() {
    let mut out = UiMesh::new();
    let mut max_x = |ppp: f32| {
        let mut runtime = UiRuntime::new(FontSource::BuiltIn);
        let input = UiFrameInput { pixels_per_point: ppp, ..Frames::input((0.0, 0.0), false, &[]) };
        run_label(&mut runtime, &input, &mut out);
        out.vertices.iter().map(|v| v.x).fold(0.0, f32::max)
    };

    let (one, one_and_half) = (max_x(1.0), max_x(1.5));

    assert!((one_and_half / one - 1.5).abs() < 0.03, "{one} -> {one_and_half}");
}

#[test]
fn a_display_and_scale_change_sends_new_glyphs() {
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();
    run_label(&mut runtime, &Frames::input((0.0, 0.0), false, &[]), &mut out);

    let bigger =
        UiFrameInput { display: (2560.0, 1440.0), pixels_per_point: 2.0, ..Frames::input((0.0, 0.0), false, &[]) };
    run_label(&mut runtime, &bigger, &mut out);
    runtime.run_with(&bigger, &DrawList::new(), label, &mut out);
    let mut first = UiMesh::new();
    let mut fresh = UiRuntime::new(FontSource::BuiltIn);
    fresh.run_with(&bigger, &DrawList::new(), label, &mut first);

    assert!(first.textures.iter().any(|t| t.full));
    assert!(out.draws.iter().all(|d| d.clip[2] <= 2560.0 && d.clip[3] <= 1440.0));
    assert!(out.draws_text("Plains"));
}

#[test]
fn text_is_collected_only_when_asked() {
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();
    let quiet = UiFrameInput { collect_text: false, ..Frames::input((0.0, 0.0), false, &[]) };

    run_label(&mut runtime, &quiet, &mut out);
    let silent = out.texts.is_empty() && out.text_rects.is_empty();
    run_label(&mut runtime, &Frames::input((0.0, 0.0), false, &[]), &mut out);

    assert!(silent);
    assert!(out.draws_text("Plains"));
}

#[test]
fn the_cursor_is_drawn_last_at_the_pointer() {
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();
    let input = UiFrameInput { draw_cursor: true, ..Frames::input((300.0, 200.0), false, &[]) };

    run_label(&mut runtime, &input, &mut out);

    let last = out.draws.last().copied().unwrap_or_default();
    let range = last.index_offset as usize..(last.index_offset + last.index_count) as usize;
    let near_tip = out.indices[range].iter().any(|&i| {
        let v = out.vertices[(last.vertex_offset + i) as usize];
        (v.x - 300.0).abs() < 1.5 && (v.y - 200.0).abs() < 1.5
    });
    assert!(near_tip);
}

#[test]
fn left_press_and_release_click_a_button() {
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();
    let layers = DrawList::new();
    let clicked = std::cell::Cell::new(false);
    let at = std::cell::Cell::new((0.0, 0.0));
    let mut show = |ctx: &Context| {
        Area::new(Id::unique("button")).fixed_pos(pos2(100.0, 100.0)).show(ctx, |ui| {
            let response = ui.button("Save");
            at.set((response.rect.center().x, response.rect.center().y));
            clicked.set(clicked.get() || response.clicked());
        });
    };

    runtime.run_with(&Frames::input((0.0, 0.0), false, &[]), &layers, &mut show, &mut out);
    runtime.run_with(&Frames::input((0.0, 0.0), false, &[]), &layers, &mut show, &mut out);
    let target = at.get();
    runtime.run_with(&Frames::input(target, false, &[]), &layers, &mut show, &mut out);
    let hovering = out.wants_pointer;
    runtime.run_with(&Frames::input(target, true, &[]), &layers, &mut show, &mut out);
    runtime.run_with(&Frames::input(target, false, &[]), &layers, &mut show, &mut out);

    assert!(hovering);
    assert!(clicked.get());
}
