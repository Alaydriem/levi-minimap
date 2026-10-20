use egui::Shape;
use egui::epaint::ClippedShape;
use minimap_core::structs::mesh::{UiMesh, UiTextRect};

// lists the text a frame laid out, with its screen rect, so tests can find and click it
pub struct TextCollector;

impl TextCollector {
    pub fn collect(shapes: &[ClippedShape], pixels_per_point: f32, out: &mut UiMesh) {
        for clipped in shapes {
            Self::shape(&clipped.shape, pixels_per_point, out);
        }
    }

    fn shape(shape: &Shape, pixels_per_point: f32, out: &mut UiMesh) {
        match shape {
            Shape::Vec(shapes) => {
                for shape in shapes {
                    Self::shape(shape, pixels_per_point, out);
                }
            }
            Shape::Text(text) => {
                let rect = text.visual_bounding_rect();
                let start = out.texts.len() as u32;
                out.texts.push_str(text.galley.text());
                out.text_rects.push(UiTextRect {
                    start,
                    len: out.texts.len() as u32 - start,
                    rect: [
                        rect.min.x * pixels_per_point,
                        rect.min.y * pixels_per_point,
                        rect.max.x * pixels_per_point,
                        rect.max.y * pixels_per_point,
                    ],
                });
            }
            _ => {}
        }
    }
}
