use egui::{
    Event, Key, Modifiers, MouseWheelSource, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect, TouchPhase,
    ViewportId, ViewportInfo, pos2, vec2,
};
use minimap_core::structs::ui::{UiEvent, UiFrameInput};

// turns one frame's polled pointer and queued keys, text and wheel into egui's RawInput; pointer
// presses are the edges of the polled button state
pub struct RawInputBuilder {
    prev_left: bool,
    prev_right: bool,
    prev_cursor: Option<Pos2>,
    modifiers: Modifiers,
}

impl RawInputBuilder {
    // lines scrolled per wheel notch
    const WHEEL_LINES: f32 = 3.0;

    pub fn new() -> Self {
        Self { prev_left: false, prev_right: false, prev_cursor: None, modifiers: Modifiers::default() }
    }

    pub fn build(&mut self, input: &UiFrameInput<'_>) -> RawInput {
        let ppp = input.pixels_per_point.max(f32::EPSILON);
        let mut raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(input.display.0 / ppp, input.display.1 / ppp))),
            time: Some(input.time_s),
            focused: input.foreground,
            ..RawInput::default()
        };
        raw.viewports.insert(
            ViewportId::ROOT,
            ViewportInfo {
                native_pixels_per_point: Some(ppp),
                focused: Some(input.foreground),
                ..ViewportInfo::default()
            },
        );
        self.pointer(input, ppp, &mut raw.events);
        for event in input.events {
            self.event(*event, &mut raw.events);
        }
        raw
    }

    fn pointer(&mut self, input: &UiFrameInput<'_>, ppp: f32, events: &mut Vec<Event>) {
        if !input.foreground {
            if self.prev_cursor.take().is_some() {
                events.push(Event::PointerGone);
            }
            self.prev_left = false;
            self.prev_right = false;
            return;
        }
        let at = pos2(input.cursor.0 / ppp, input.cursor.1 / ppp);
        if self.prev_cursor != Some(at) {
            events.push(Event::PointerMoved(at));
            self.prev_cursor = Some(at);
        }
        for (down, prev, button) in [
            (input.left_down, &mut self.prev_left, PointerButton::Primary),
            (input.right_down, &mut self.prev_right, PointerButton::Secondary),
        ] {
            if down != *prev {
                events.push(Event::PointerButton { pos: at, button, pressed: down, modifiers: self.modifiers });
                *prev = down;
            }
        }
    }

    fn event(&mut self, event: UiEvent, events: &mut Vec<Event>) {
        match event {
            UiEvent::Text(c) if !c.is_control() => events.push(Event::Text(c.to_string())),
            UiEvent::Text(_) => {}
            UiEvent::Wheel(data) if data != 0 => events.push(Event::MouseWheel {
                unit: MouseWheelUnit::Line,
                delta: vec2(0.0, (data.signum() as f32) * Self::WHEEL_LINES),
                phase: TouchPhase::Move,
                source: MouseWheelSource::Wheel,
                modifiers: self.modifiers,
            }),
            UiEvent::Wheel(_) => {}
            UiEvent::Key { code, down, ctrl, shift } => {
                self.modifiers = Modifiers { ctrl, shift, command: ctrl, ..Modifiers::default() };
                match (Self::key(code), ctrl && down) {
                    (Some(Key::C), true) => events.push(Event::Copy),
                    (Some(Key::X), true) => events.push(Event::Cut),
                    (Some(key), _) => events.push(Event::Key {
                        key,
                        physical_key: None,
                        pressed: down,
                        repeat: false,
                        modifiers: self.modifiers,
                    }),
                    (None, _) => {}
                }
            }
        }
    }

    // the egui key for a Windows virtual key code; the keys text fields and dialogs use
    pub fn key(code: i32) -> Option<Key> {
        const DIGITS: [Key; 10] = [
            Key::Num0,
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Num8,
            Key::Num9,
        ];
        const LETTERS: [Key; 26] = [
            Key::A,
            Key::B,
            Key::C,
            Key::D,
            Key::E,
            Key::F,
            Key::G,
            Key::H,
            Key::I,
            Key::J,
            Key::K,
            Key::L,
            Key::M,
            Key::N,
            Key::O,
            Key::P,
            Key::Q,
            Key::R,
            Key::S,
            Key::T,
            Key::U,
            Key::V,
            Key::W,
            Key::X,
            Key::Y,
            Key::Z,
        ];
        match code {
            0x08 => Some(Key::Backspace),
            0x09 => Some(Key::Tab),
            0x0D => Some(Key::Enter),
            0x1B => Some(Key::Escape),
            0x20 => Some(Key::Space),
            0x23 => Some(Key::End),
            0x24 => Some(Key::Home),
            0x25 => Some(Key::ArrowLeft),
            0x26 => Some(Key::ArrowUp),
            0x27 => Some(Key::ArrowRight),
            0x28 => Some(Key::ArrowDown),
            0x2E => Some(Key::Delete),
            0x30..=0x39 => DIGITS.get((code - 0x30) as usize).copied(),
            0x41..=0x5A => LETTERS.get((code - 0x41) as usize).copied(),
            _ => None,
        }
    }
}

impl Default for RawInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}
