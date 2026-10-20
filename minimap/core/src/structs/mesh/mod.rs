pub mod draw;
pub mod text_rect;
pub mod texture_delta;
pub mod vertex;

pub use draw::UiDraw;
pub use text_rect::UiTextRect;
pub use texture_delta::UiTextureDelta;
pub use vertex::UiVertex;

// one tessellated UI frame for the shim; every buffer keeps its capacity between frames
#[derive(Debug, Clone, Default)]
pub struct UiMesh {
    pub vertices: Vec<UiVertex>,
    pub indices: Vec<u32>,
    pub draws: Vec<UiDraw>,
    // uploads for the UI's own textures (the font atlas); their pixels are ranges of `pixels`
    pub textures: Vec<UiTextureDelta>,
    pub pixels: Vec<u8>,
    // UI textures to free after this frame is drawn
    pub freed: Vec<u32>,
    // the pointer is over a panel, and a text field has the keyboard
    pub wants_pointer: bool,
    pub wants_keyboard: bool,
    // text for the system clipboard; empty when there is none
    pub clipboard: String,
    // the text laid out this frame, only when the frame input asked for it
    pub texts: String,
    pub text_rects: Vec<UiTextRect>,
}

impl UiMesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.draws.clear();
        self.textures.clear();
        self.pixels.clear();
        self.freed.clear();
        self.wants_pointer = false;
        self.wants_keyboard = false;
        self.clipboard.clear();
        self.texts.clear();
        self.text_rects.clear();
    }

    pub fn text_of(&self, text: &UiTextRect) -> &str {
        self.texts.get(text.start as usize..(text.start + text.len) as usize).unwrap_or("")
    }

    pub fn draws_text(&self, text: &str) -> bool {
        self.text_rect(text).is_some()
    }

    pub fn text_starting(&self, prefix: &str) -> bool {
        self.text_rects.iter().any(|t| self.text_of(t).starts_with(prefix))
    }

    // the screen rect of the first text that is exactly `text`, as [x0, y0, x1, y1] in pixels
    pub fn text_rect(&self, text: &str) -> Option<[f32; 4]> {
        self.text_rects.iter().find(|t| self.text_of(t) == text).map(|t| t.rect)
    }
}
