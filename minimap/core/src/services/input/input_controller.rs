use crate::config::Config;
use crate::services::draw::MapRotation;
use crate::structs::input::{KeyInput, KeyResult, MouseAction, MouseInput};
use crate::structs::view::ViewState;

// key and mouse decisions: world map toggle, minimap toggle and zoom, world map wheel zoom
pub struct InputController {
    toggle_key: i32,
    zoom_in_key: i32,
    zoom_out_key: i32,
    // None while waypoints are turned off
    add_waypoint_key: Option<i32>,
    cave_key: i32,
    coordinates_key: i32,
    min_radius: i32,
    max_radius: i32,
    zoom_step: f32,
    min_zoom: f32,
    max_zoom: f32,
}

impl InputController {
    pub const ESCAPE: i32 = 0x1B;

    pub fn new(config: &Config) -> Self {
        Self {
            toggle_key: config.world_map.toggle_key,
            zoom_in_key: config.mini_map.zoom_in_key,
            zoom_out_key: config.mini_map.zoom_out_key,
            add_waypoint_key: config.waypoints.enabled.then_some(config.waypoints.add_key),
            cave_key: config.cave_mode.toggle_key,
            coordinates_key: config.mini_map.coordinates_key,
            min_radius: config.mini_map.min_radius_chunks,
            max_radius: config.mini_map.max_radius_chunks,
            zoom_step: config.world_map.zoom_step,
            min_zoom: config.world_map.min_zoom,
            max_zoom: config.world_map.max_zoom,
        }
    }

    pub fn on_key(&self, view: &mut ViewState, input: KeyInput) -> KeyResult {
        if view.world_map_open {
            return self.on_key_with_map_open(view, input);
        }
        let mut result = KeyResult::default();
        if !input.down || !input.mouse_grabbed {
            return result;
        }
        if input.code == self.toggle_key {
            result.consume = true;
            if input.ctrl {
                view.show_minimap = !view.show_minimap;
            } else {
                view.world_map_open = true;
                view.offset_x = 0.0;
                view.offset_z = 0.0;
                view.anchor = None;
                result.action = MouseAction::ReleaseMouse;
            }
        } else if input.code == self.zoom_in_key || input.code == self.zoom_out_key {
            result.consume = true;
            let step = if input.code == self.zoom_in_key { -1 } else { 1 };
            let radius = (view.minimap_radius + step).clamp(self.min_radius, self.max_radius);
            result.radius_changed = radius != view.minimap_radius;
            view.minimap_radius = radius;
        } else if Some(input.code) == self.add_waypoint_key && !input.ctrl {
            result.consume = true;
            result.add_waypoint = true;
        } else if input.code == self.coordinates_key && input.ctrl {
            result.consume = true;
            result.coordinates_changed = Self::toggle_coordinates(view);
        } else if input.code == self.cave_key && !input.ctrl {
            result.consume = true;
            result.toggle_cave = true;
        }
        result
    }

    fn toggle_coordinates(view: &mut ViewState) -> bool {
        view.show_coordinates = !view.show_coordinates;
        true
    }

    // the open world map takes every key-down; the toggle key or Escape closes it
    fn on_key_with_map_open(&self, view: &mut ViewState, input: KeyInput) -> KeyResult {
        let mut result = KeyResult { consume: input.down, ..KeyResult::default() };
        if !input.down {
            return result;
        }
        if input.code == self.toggle_key && input.ctrl {
            view.show_minimap = !view.show_minimap;
        } else if input.code == self.toggle_key || input.code == Self::ESCAPE {
            view.world_map_open = false;
            view.anchor = None;
            result.action = MouseAction::GrabMouse;
        } else if input.code == self.coordinates_key && input.ctrl {
            result.coordinates_changed = Self::toggle_coordinates(view);
        } else if input.code == self.cave_key && !input.ctrl {
            result.toggle_cave = true;
        }
        result
    }

    // returns true when the game must not see the mouse event
    pub fn on_mouse(&self, view: &mut ViewState, input: MouseInput) -> bool {
        if !view.world_map_open {
            return false;
        }
        if input.button == MouseInput::WHEEL && input.data != 0 {
            let factor = if input.data > 0 { self.zoom_step } else { 1.0 / self.zoom_step };
            view.zoom = (view.zoom * factor).clamp(self.min_zoom, self.max_zoom);
        }
        input.data == 1 || input.button == MouseInput::WHEEL || input.button == 0
    }

    // left-drag pan: the cursor moved by (dx, dy) screen pixels on a map turned by `angle` radians,
    // so the map moves under the cursor whatever the turn
    pub fn on_drag(&self, view: &mut ViewState, dx: f32, dy: f32, angle: f32) {
        let zoom = view.zoom.clamp(self.min_zoom, self.max_zoom);
        let (mx, mz) = MapRotation::new(angle, (0.0, 0.0)).invert_vector((dx, dy));
        if let Some((x, z)) = view.anchor {
            view.anchor = Some((x - f64::from(mx / zoom), z - f64::from(mz / zoom)));
            return;
        }
        view.offset_x -= mx / zoom;
        view.offset_z -= mz / zoom;
    }
}
