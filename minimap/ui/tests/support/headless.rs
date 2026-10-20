use egui::{Context, Event, FullOutput, Modifiers, PointerButton, Pos2, RawInput, Rect, vec2};
use minimap_ui::theme::{FontSource, StudioTheme};

// runs egui passes without a window, at one physical pixel per point
pub struct Headless;

impl Headless {
    pub fn context() -> Context {
        let ctx = Context::default();
        StudioTheme::install(&ctx, FontSource::BuiltIn);
        ctx
    }

    pub fn frame(ctx: &Context, size: [f32; 2], events: Vec<Event>, ui: impl FnMut(&Context)) -> FullOutput {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(size[0], size[1]))),
            events,
            ..RawInput::default()
        };
        let mut output = ctx.run_pass(input, ui);
        output.textures_delta.clear();
        output
    }

    // the events of a left press, or of its release, at `at`
    pub fn button(at: Pos2, pressed: bool) -> Vec<Event> {
        vec![
            Event::PointerMoved(at),
            Event::PointerButton { pos: at, button: PointerButton::Primary, pressed, modifiers: Modifiers::default() },
        ]
    }

    // a left click at `at` over two passes, after two passes that lay the UI out: a new area is
    // invisible in its first pass and takes input from its second
    pub fn click(ctx: &Context, size: [f32; 2], at: Pos2, mut ui: impl FnMut(&Context)) {
        let _ = Self::frame(ctx, size, vec![Event::PointerMoved(at)], &mut ui);
        let _ = Self::frame(ctx, size, vec![Event::PointerMoved(at)], &mut ui);
        let _ = Self::frame(ctx, size, Self::button(at, true), &mut ui);
        let _ = Self::frame(ctx, size, Self::button(at, false), &mut ui);
    }
}
