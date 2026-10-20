// radii, sizes and the spacing grid of the Studio Dark look, in points
pub struct Metrics;

impl Metrics {
    pub const RADIUS_SMALL: u8 = 4;
    pub const RADIUS_CONTROL: u8 = 6;
    pub const RADIUS_PANEL: u8 = 10;
    pub const RADIUS_PILL: u8 = 17;

    pub const GAP_S: f32 = 8.0;
    pub const GAP_M: f32 = 12.0;
    pub const GAP_L: f32 = 16.0;

    pub const ROW: f32 = 28.0;
    pub const FIELD: f32 = 34.0;
    pub const BUTTON: f32 = 36.0;

    pub const TOOLS_WIDTH: f32 = 300.0;
    pub const LIST_WIDTH: f32 = 360.0;
    pub const LIST_HEIGHT: f32 = 452.0;
    pub const MENU_WIDTH: f32 = 250.0;
    pub const DIALOG_WIDTH: f32 = 480.0;

    // hover and press transitions, ease-out
    pub const MOTION_SECONDS: f32 = 0.12;
}
