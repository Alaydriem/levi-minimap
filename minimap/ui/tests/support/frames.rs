use minimap_core::structs::draw::DrawList;
use minimap_core::structs::mesh::UiMesh;
use minimap_core::structs::ui::{UiEvent, UiFrameInput};
use minimap_ui::runtime::UiRuntime;

// builds UiFrameInputs and runs UiRuntime frames for the tests
pub struct Frames;

impl Frames {
    pub const DISPLAY: (f32, f32) = (1280.0, 720.0);

    pub fn input(cursor: (f32, f32), left_down: bool, events: &[UiEvent]) -> UiFrameInput<'_> {
        UiFrameInput {
            display: Self::DISPLAY,
            pixels_per_point: 1.0,
            time_s: 0.0,
            cursor,
            left_down,
            right_down: false,
            foreground: true,
            events,
            draw_cursor: false,
            collect_text: true,
        }
    }

    pub fn run(runtime: &mut UiRuntime, input: &UiFrameInput<'_>, layers: &DrawList, out: &mut UiMesh) {
        runtime.run_with(input, layers, |_| {}, out);
    }
}
