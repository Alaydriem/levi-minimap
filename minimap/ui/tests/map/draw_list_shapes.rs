use minimap_core::structs::draw::{DrawCommand, DrawList, SamplerKind, TextAlign};
use minimap_core::structs::mesh::{UiDraw, UiMesh, UiVertex};
use minimap_ui::runtime::UiRuntime;
use minimap_ui::theme::FontSource;

use crate::support::Frames;

fn mesh_of(list: &DrawList) -> UiMesh {
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();
    Frames::run(&mut runtime, &Frames::input((0.0, 0.0), false, &[]), list, &mut out);
    out
}

fn map_draws(mesh: &UiMesh) -> Vec<UiDraw> {
    mesh.draws.iter().copied().filter(|d| d.kind == UiDraw::MAP).collect()
}

fn vertices_of(mesh: &UiMesh, draw: &UiDraw) -> Vec<UiVertex> {
    let range = draw.index_offset as usize..(draw.index_offset + draw.index_count) as usize;
    mesh.indices[range].iter().map(|&i| mesh.vertices[(draw.vertex_offset + i) as usize]).collect()
}

fn image(tex: u32) -> DrawCommand {
    DrawCommand::Image { tex, p0: (10.0, 10.0), p1: (266.0, 266.0), uv0: (0.0, 0.0), uv1: (1.0, 1.0) }
}

#[test]
fn a_map_image_draws_with_its_texture_and_the_last_sampler() {
    let mut list = DrawList::new();
    list.push(image(3));
    list.push(DrawCommand::Sampler(SamplerKind::Linear));
    list.push(image(7));

    let draws = map_draws(&mesh_of(&list));

    let samplers: Vec<(u32, u8)> = draws.iter().map(|d| (d.texture, d.sampler)).collect();
    assert_eq!(samplers, vec![(3, UiDraw::POINT), (7, UiDraw::LINEAR)]);
}

#[test]
fn an_image_disc_stays_inside_its_circle() {
    let mut list = DrawList::new();
    list.push(DrawCommand::ImageDisc {
        tex: 1,
        centre: (200.0, 200.0),
        radius: 100.0,
        uv_centre: (0.5, 0.5),
        uv_radius: 0.4,
        angle: 0.6,
    });

    let mesh = mesh_of(&list);
    let draws = map_draws(&mesh);

    assert_eq!(draws.len(), 1);
    let vertices = vertices_of(&mesh, &draws[0]);
    assert!(vertices.len() / 3 >= 32);
    assert!(vertices.iter().all(|v| ((v.x - 200.0).hypot(v.y - 200.0)) <= 100.5));
    assert!(vertices.iter().all(|v| ((v.u - 0.5).hypot(v.v - 0.5)) <= 0.4001));
}

#[test]
fn a_turned_image_keeps_its_corner_uvs_and_turns_its_corners() {
    let mut list = DrawList::new();
    list.push(DrawCommand::ImageRotated {
        tex: 4,
        centre: (300.0, 300.0),
        half: (100.0, 50.0),
        uv0: (0.0, 0.0),
        uv1: (1.0, 1.0),
        angle: std::f32::consts::FRAC_PI_2,
    });

    let mesh = mesh_of(&list);
    let draws = map_draws(&mesh);
    let vertices = vertices_of(&mesh, &draws[0]);

    let corner = vertices.iter().find(|v| v.u == 0.0 && v.v == 0.0).copied().unwrap_or_default();
    assert!((corner.x - 350.0).abs() < 0.01 && (corner.y - 200.0).abs() < 0.01, "{corner:?}");
    for uv in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
        assert!(vertices.iter().any(|v| (v.u, v.v) == uv));
    }
}

#[test]
fn text_commands_are_laid_out_with_their_string() {
    let mut list = DrawList::new();
    list.text((640.0, 20.0), 0xFFFF_FFFF, 18.0, TextAlign::Centre, "12, 64, -3");

    let mesh = mesh_of(&list);

    let rect = mesh.text_rect("12, 64, -3").unwrap_or_default();
    assert!(((rect[0] + rect[2]) * 0.5 - 640.0).abs() < 1.0 && rect[1] >= 19.0, "{rect:?}");
}

#[test]
fn map_shapes_divide_by_the_pixels_per_point_and_come_back_in_pixels() {
    let mut list = DrawList::new();
    list.push(image(9));
    let mut runtime = UiRuntime::new(FontSource::BuiltIn);
    let mut out = UiMesh::new();
    let input =
        minimap_core::structs::ui::UiFrameInput { pixels_per_point: 2.0, ..Frames::input((0.0, 0.0), false, &[]) };

    Frames::run(&mut runtime, &input, &list, &mut out);

    let draws = map_draws(&out);
    let xs: Vec<f32> = vertices_of(&out, &draws[0]).iter().map(|v| v.x).collect();
    assert!(xs.iter().all(|x| (x - 10.0).abs() < 0.01 || (x - 266.0).abs() < 0.01), "{xs:?}");
}
