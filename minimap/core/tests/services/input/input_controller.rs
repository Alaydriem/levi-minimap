use minimap_core::config::Config;
use minimap_core::services::input::InputController;
use minimap_core::structs::input::{KeyInput, MouseAction, MouseInput};
use minimap_core::structs::view::ViewState;

const M: i32 = 0x4D;
const EQUALS: i32 = 0xBB;
const MINUS: i32 = 0xBD;

fn controller() -> InputController {
    InputController::new(&Config::default())
}

#[test]
fn the_toggle_key_opens_the_world_map_only_while_the_game_holds_the_mouse() {
    let mut view = ViewState::new(5);
    let typing = controller().on_key(&mut view, KeyInput::new(M, true, false, false));
    assert!(!typing.consume);
    assert!(!view.world_map_open);
    view.offset_x = 40.0;
    let open = controller().on_key(&mut view, KeyInput::new(M, true, false, true));
    assert!(open.consume);
    assert_eq!(open.action, MouseAction::ReleaseMouse);
    assert!(view.world_map_open);
    assert_eq!(view.offset_x, 0.0);
}

#[test]
fn ctrl_and_the_toggle_key_hide_the_minimap_in_either_state() {
    let mut view = ViewState::new(5);
    controller().on_key(&mut view, KeyInput::new(M, true, true, true));
    assert!(!view.show_minimap);
    view.world_map_open = true;
    let result = controller().on_key(&mut view, KeyInput::new(M, true, true, false));
    assert!(view.show_minimap);
    assert!(view.world_map_open);
    assert_eq!(result.action, MouseAction::None);
}

#[test]
fn the_open_map_takes_every_key_down_and_closes_on_toggle_or_escape() {
    let mut view = ViewState { world_map_open: true, ..ViewState::new(5) };
    let other = controller().on_key(&mut view, KeyInput::new(0x57, true, false, false));
    assert!(other.consume);
    assert!(view.world_map_open);
    let release = controller().on_key(&mut view, KeyInput::new(0x57, false, false, false));
    assert!(!release.consume);
    let escape = controller().on_key(&mut view, KeyInput::new(InputController::ESCAPE, true, false, false));
    assert_eq!(escape.action, MouseAction::GrabMouse);
    assert!(!view.world_map_open);
}

#[test]
fn zoom_keys_change_the_radius_within_its_limits() {
    let mut view = ViewState::new(3);
    let zoom_in = controller().on_key(&mut view, KeyInput::new(EQUALS, true, false, true));
    assert!(zoom_in.consume && zoom_in.radius_changed);
    assert_eq!(view.minimap_radius, 2);
    let at_limit = controller().on_key(&mut view, KeyInput::new(EQUALS, true, false, true));
    assert!(at_limit.consume && !at_limit.radius_changed);
    assert_eq!(view.minimap_radius, 2);
    view.minimap_radius = 16;
    controller().on_key(&mut view, KeyInput::new(MINUS, true, false, true));
    assert_eq!(view.minimap_radius, 16);
}

#[test]
fn the_wheel_zooms_the_open_map_within_its_limits_and_is_consumed() {
    let mut view = ViewState { world_map_open: true, ..ViewState::new(5) };
    assert!(controller().on_mouse(&mut view, MouseInput::new(MouseInput::WHEEL, 1)));
    assert!((view.zoom - 1.15).abs() < 1e-6);
    for _ in 0..200 {
        controller().on_mouse(&mut view, MouseInput::new(MouseInput::WHEEL, -120));
    }
    assert!((view.zoom - 0.2).abs() < 1e-6);
    view.world_map_open = false;
    assert!(!controller().on_mouse(&mut view, MouseInput::new(MouseInput::WHEEL, 1)));
}

#[test]
fn dragging_moves_the_map_against_the_cursor_scaled_by_zoom() {
    let mut view = ViewState { world_map_open: true, zoom: 2.0, ..ViewState::new(5) };
    controller().on_drag(&mut view, 10.0, -4.0, 0.0);
    assert_eq!((view.offset_x, view.offset_z), (-5.0, 2.0));
}

#[test]
fn on_a_turned_map_the_drag_still_moves_the_map_under_the_cursor() {
    // a quarter turn puts world east at the screen bottom, so screen right is world north;
    // dragging right pulls the map right, so the centre moves south
    let mut view = ViewState { world_map_open: true, zoom: 2.0, ..ViewState::new(5) };
    controller().on_drag(&mut view, 10.0, 0.0, std::f32::consts::FRAC_PI_2);
    assert!(view.offset_x.abs() < 1e-4, "{}", view.offset_x);
    assert!((view.offset_z - 5.0).abs() < 1e-4, "{}", view.offset_z);
}

const N: i32 = 0x4E;

#[test]
fn the_waypoint_key_asks_for_a_waypoint_only_on_the_open_game_view() {
    let mut view = ViewState::new(5);
    let added = controller().on_key(&mut view, KeyInput::new(N, true, false, true));
    assert!(added.consume && added.add_waypoint);
    let typing = controller().on_key(&mut view, KeyInput::new(N, true, false, false));
    assert!(!typing.consume && !typing.add_waypoint);
    let with_ctrl = controller().on_key(&mut view, KeyInput::new(N, true, true, true));
    assert!(!with_ctrl.add_waypoint);
    let released = controller().on_key(&mut view, KeyInput::new(N, false, false, true));
    assert!(!released.add_waypoint);
    view.world_map_open = true;
    let on_the_map = controller().on_key(&mut view, KeyInput::new(N, true, false, false));
    assert!(on_the_map.consume && !on_the_map.add_waypoint);
}

#[test]
fn with_waypoints_turned_off_the_waypoint_key_is_left_to_the_game() {
    let mut config = Config::default();
    config.waypoints.enabled = false;
    let mut view = ViewState::new(5);
    let result = InputController::new(&config).on_key(&mut view, KeyInput::new(N, true, false, true));
    assert!(!result.consume && !result.add_waypoint);
}

const K: i32 = 0x4B;

#[test]
fn the_cave_key_toggles_on_the_game_view_and_on_the_open_map() {
    let mut view = ViewState::new(5);
    let game = controller().on_key(&mut view, KeyInput::new(K, true, false, true));
    assert!(game.consume && game.toggle_cave);
    let typing = controller().on_key(&mut view, KeyInput::new(K, true, false, false));
    assert!(!typing.toggle_cave);
    view.world_map_open = true;
    let map = controller().on_key(&mut view, KeyInput::new(K, true, false, false));
    assert!(map.consume && map.toggle_cave && view.world_map_open);
}

#[test]
fn ctrl_k_shows_and_hides_the_coordinates_on_either_view_and_plain_k_stays_the_cave_key() {
    let mut view = ViewState::new(5);
    let hide = controller().on_key(&mut view, KeyInput::new(K, true, true, true));
    assert!(hide.consume && hide.coordinates_changed && !hide.toggle_cave);
    assert!(!view.show_coordinates);
    view.world_map_open = true;
    let show = controller().on_key(&mut view, KeyInput::new(K, true, true, false));
    assert!(show.coordinates_changed && view.show_coordinates && view.world_map_open);
    let cave = controller().on_key(&mut view, KeyInput::new(K, true, false, false));
    assert!(cave.toggle_cave && !cave.coordinates_changed);
}

const ESCAPE: i32 = 0x1B;

#[test]
fn a_drag_moves_the_anchor_and_not_the_offset() {
    let mut view = ViewState { world_map_open: true, anchor: Some((0.0, 0.0)), zoom: 2.0, ..ViewState::new(5) };
    controller().on_drag(&mut view, 10.0, -4.0, 0.0);
    assert_eq!(view.anchor, Some((-5.0, 2.0)));
    assert_eq!((view.offset_x, view.offset_z), (0.0, 0.0));
}

#[test]
fn opening_and_closing_the_map_clears_the_anchor() {
    let mut view = ViewState { anchor: Some((5.0, 5.0)), ..ViewState::new(5) };
    controller().on_key(&mut view, KeyInput::new(M, true, false, true));
    assert!(view.world_map_open);
    assert_eq!(view.anchor, None);
    view.anchor = Some((7.0, 7.0));
    controller().on_key(&mut view, KeyInput::new(ESCAPE, true, false, false));
    assert!(!view.world_map_open);
    assert_eq!(view.anchor, None);
}
