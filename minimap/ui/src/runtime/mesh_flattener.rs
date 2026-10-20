use egui::epaint::textures::TexturesDelta;
use egui::epaint::{ImageData, Primitive};
use egui::{ClippedPrimitive, TextureId};
use minimap_core::structs::draw::SamplerKind;
use minimap_core::structs::mesh::{UiDraw, UiMesh, UiTextureDelta, UiVertex};

use crate::map::MapTextureId;

// appends egui's tessellated meshes and texture changes to one UiMesh, in physical pixels
pub struct MeshFlattener;

impl MeshFlattener {
    pub fn textures(delta: &TexturesDelta, out: &mut UiMesh) {
        for (id, images) in &delta.set {
            let TextureId::Managed(texture) = *id else { continue };
            for image in images {
                let ImageData::Color(colour) = &image.image;
                let offset = out.pixels.len();
                out.pixels.reserve(colour.pixels.len() * 4);
                for pixel in &colour.pixels {
                    out.pixels.extend_from_slice(&pixel.to_array());
                }
                let (x, y) = image.pos.map_or((0, 0), |[x, y]| (x as u32, y as u32));
                out.textures.push(UiTextureDelta {
                    texture: texture as u32,
                    full: image.pos.is_none(),
                    x,
                    y,
                    width: colour.size[0] as u32,
                    height: colour.size[1] as u32,
                    offset,
                    len: out.pixels.len() - offset,
                });
            }
        }
        out.freed.extend(delta.free.iter().filter_map(|id| match *id {
            TextureId::Managed(texture) => Some(texture as u32),
            TextureId::User(_) => None,
        }));
    }

    pub fn meshes(primitives: &[ClippedPrimitive], pixels_per_point: f32, display: (f32, f32), out: &mut UiMesh) {
        for primitive in primitives {
            let Primitive::Mesh(mesh) = &primitive.primitive else { continue };
            if mesh.indices.is_empty() {
                continue;
            }
            let clip = primitive.clip_rect;
            let clip = [
                (clip.min.x * pixels_per_point).clamp(0.0, display.0),
                (clip.min.y * pixels_per_point).clamp(0.0, display.1),
                (clip.max.x * pixels_per_point).clamp(0.0, display.0),
                (clip.max.y * pixels_per_point).clamp(0.0, display.1),
            ];
            if clip[2] <= clip[0] || clip[3] <= clip[1] {
                continue;
            }
            let (texture, kind, sampler) = match MapTextureId::decode(mesh.texture_id) {
                Some((tex, SamplerKind::Point)) => (tex, UiDraw::MAP, UiDraw::POINT),
                Some((tex, SamplerKind::Linear)) => (tex, UiDraw::MAP, UiDraw::LINEAR),
                None => match mesh.texture_id {
                    TextureId::Managed(id) => (id as u32, UiDraw::EGUI, UiDraw::LINEAR),
                    TextureId::User(_) => continue,
                },
            };
            out.draws.push(UiDraw {
                clip,
                texture,
                kind,
                sampler,
                reserved: [0; 2],
                index_offset: out.indices.len() as u32,
                index_count: mesh.indices.len() as u32,
                vertex_offset: out.vertices.len() as u32,
            });
            out.indices.extend_from_slice(&mesh.indices);
            out.vertices.extend(mesh.vertices.iter().map(|v| UiVertex {
                x: v.pos.x * pixels_per_point,
                y: v.pos.y * pixels_per_point,
                u: v.uv.x,
                v: v.uv.y,
                rgba: u32::from_le_bytes(v.color.to_array()),
            }));
        }
    }
}
