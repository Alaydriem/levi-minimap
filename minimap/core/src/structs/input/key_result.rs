use crate::structs::input::MouseAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyResult {
    // the game must not see this key event
    pub consume: bool,
    pub action: MouseAction,
    // the minimap radius changed and should be saved to config
    pub radius_changed: bool,
    // add a waypoint at the player
    pub add_waypoint: bool,
    // switch between the surface and the cave view
    pub toggle_cave: bool,
    // the coordinates were shown or hidden and should be saved to config
    pub coordinates_changed: bool,
}
