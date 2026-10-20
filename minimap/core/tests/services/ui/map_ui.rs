use minimap_core::services::ui::MapUi;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::export::ExportKind;
use minimap_core::structs::input::{KeyInput, MouseInput};
use minimap_core::structs::ui::{MapAction, MenuItem, MenuTarget, MessageKind, UiEvent};
use minimap_core::structs::view::ViewState;
use minimap_core::structs::waypoint::WaypointSource;

const AT: (f32, f32) = (300.0, 200.0);
const GROUND: MenuTarget = MenuTarget::Ground { dim: 0, x: 38, y: 71, z: -26 };
const NOW: u64 = 1_792_138_360;

fn key(code: i32) -> KeyInput {
    KeyInput::new(code, true, false, false)
}

fn ctrl(code: i32) -> KeyInput {
    KeyInput::new(code, true, true, false)
}

// a panel's action, with `name` as the dialog's typed name
fn act(ui: &mut MapUi, store: &mut WaypointStore, action: MapAction, name: &str) {
    ui.apply(action, name, 0, &mut ViewState::new(5), store, NOW);
}

// the menu row holding `item`, as the menu view reports a click on it
fn choose(ui: &mut MapUi, store: &mut WaypointStore, item: MenuItem) {
    assert!(ui.menu().expect("an open menu").items.contains(&item));
    act(ui, store, MapAction::Choose(item), "");
}

fn type_text(ui: &mut MapUi, text: &str) {
    for unit in text.encode_utf16() {
        ui.text(u32::from(unit));
    }
}

fn drained(ui: &mut MapUi) -> Vec<UiEvent> {
    let mut out = Vec::new();
    ui.drain_events(&mut out);
    out
}

fn with_home() -> (MapUi, WaypointStore, minimap_core::structs::waypoint::WaypointId) {
    let mut store = WaypointStore::new(8, 1);
    store.add("Other", 0, (0, 64, 0), WaypointSource::Manual, NOW).unwrap();
    let home = store.add("Home", 0, (5, 70, 6), WaypointSource::Manual, NOW).unwrap();
    store.add("Third", 0, (9, 64, 9), WaypointSource::Manual, NOW).unwrap();
    (MapUi::new(1.0, 64), store, home)
}

#[test]
fn add_here_opens_the_dialog_at_the_column_and_save_keeps_the_typed_name_and_picks() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::AddWaypoint);
    let dialog = ui.dialog().expect("the dialog");
    assert_eq!(dialog.position, (38, 71, -26));
    assert_eq!(dialog.colour, store.next_colour());
    assert!(dialog.initial_name.is_empty());
    assert!(ui.wants_text() && ui.menu().is_none());
    act(&mut ui, &mut store, MapAction::DialogColour(WaypointStore::PALETTE[3]), "");
    act(&mut ui, &mut store, MapAction::DialogEdge, "");
    act(&mut ui, &mut store, MapAction::SaveDialog, "Base");
    assert!(ui.dialog().is_none());
    let saved = &store.waypoints()[0];
    assert_eq!((saved.name.as_str(), saved.x, saved.y, saved.z), ("Base", 38, 71, -26));
    assert_eq!(saved.colour, WaypointStore::PALETTE[3]);
    assert!(!saved.show_on_edge && saved.show_distance);
    assert_eq!(saved.source, WaypointSource::Manual);
}

#[test]
fn a_blank_name_saves_as_the_next_waypoint_number() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::AddWaypoint);
    act(&mut ui, &mut store, MapAction::SaveDialog, "");
    assert_eq!(store.waypoints()[0].name, "Waypoint 1");
}

#[test]
fn while_the_dialog_is_open_every_key_is_its_own_and_goes_to_the_ui_in_order() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::AddWaypoint);
    assert!(ui.key(key(0x4D), &mut store));
    type_text(&mut ui, "b");
    assert!(ui.key(key(0x0D), &mut store));
    ui.open_menu(AT, GROUND);
    assert!(ui.menu().is_none());
    let enter = UiEvent::Key { code: 0x0D, down: true, ctrl: false, shift: false };
    let m = UiEvent::Key { code: 0x4D, down: true, ctrl: false, shift: false };
    assert_eq!(drained(&mut ui), vec![m, UiEvent::Text('b'), enter]);
    assert!(ui.dialog().is_some(), "Enter reaches the dialog through the UI, not at once");
    act(&mut ui, &mut store, MapAction::CancelDialog, "");
    assert!(ui.dialog().is_none() && store.is_empty());
    assert!(!ui.key(key(0x4D), &mut store));
}

#[test]
fn each_dialog_opened_gets_a_new_serial() {
    let (mut ui, mut store, home) = with_home();
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::AddWaypoint);
    let first = ui.dialog().unwrap().serial;
    act(&mut ui, &mut store, MapAction::CancelDialog, "");
    ui.open_menu(AT, MenuTarget::Marker(home));
    choose(&mut ui, &mut store, MenuItem::EditWaypoint);
    assert!(ui.dialog().unwrap().serial > first);
}

#[test]
fn escape_or_a_press_outside_closes_the_menu_without_choosing() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    ui.open_menu(AT, GROUND);
    assert!(ui.key(key(MapUi::ESCAPE), &mut store));
    assert!(!ui.is_open());
    ui.open_menu(AT, GROUND);
    act(&mut ui, &mut store, MapAction::CloseMenu, "");
    assert!(!ui.is_open() && ui.dialog().is_none());
}

#[test]
fn copy_coordinates_puts_x_tilde_z_for_ground_and_x_y_z_for_a_marker() {
    let (mut ui, mut store, home) = with_home();
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::CopyCoordinates);
    assert_eq!(ui.take_clipboard().as_deref(), Some("38, ~, -26"));
    assert_eq!(ui.message().unwrap().text, "Copied 38, ~, -26");
    assert_eq!(ui.message().unwrap().kind, MessageKind::Done);
    assert!(ui.take_clipboard().is_none());
    ui.open_menu(AT, MenuTarget::Marker(home));
    choose(&mut ui, &mut store, MenuItem::CopyCoordinates);
    assert_eq!(ui.take_clipboard().as_deref(), Some("5, 70, 6"));
}

#[test]
fn remove_shows_how_to_undo_and_ctrl_z_puts_it_back_in_its_place() {
    let (mut ui, mut store, home) = with_home();
    ui.open_menu(AT, MenuTarget::Marker(home));
    choose(&mut ui, &mut store, MenuItem::RemoveWaypoint);
    assert_eq!(store.len(), 2);
    assert_eq!(ui.message().unwrap().text, "Removed \"Home\" - press Ctrl+Z to undo");
    assert_eq!(ui.message().unwrap().kind, MessageKind::Info);
    assert!(ui.key(ctrl(MapUi::Z), &mut store));
    assert_eq!(store.waypoints()[1].id, home);
    assert_eq!(ui.message().unwrap().text, "Restored \"Home\"");
    assert!(!ui.key(ctrl(MapUi::Z), &mut store));
}

#[test]
fn the_undo_runs_out_with_its_message() {
    let (mut ui, mut store, home) = with_home();
    ui.open_menu(AT, MenuTarget::Marker(home));
    choose(&mut ui, &mut store, MenuItem::RemoveWaypoint);
    ui.tick(MapUi::MESSAGE_SECONDS - 0.1);
    assert!(ui.message().is_some());
    ui.tick(0.2);
    assert!(ui.message().is_none());
    assert!(!ui.key(ctrl(MapUi::Z), &mut store));
    assert_eq!(store.len(), 2);
}

#[test]
fn edit_changes_the_name_colour_and_flags_but_not_the_place() {
    let (mut ui, mut store, home) = with_home();
    ui.open_menu(AT, MenuTarget::Marker(home));
    choose(&mut ui, &mut store, MenuItem::EditWaypoint);
    assert_eq!(ui.dialog().unwrap().initial_name, "Home");
    act(&mut ui, &mut store, MapAction::DialogColour(WaypointStore::PALETTE[5]), "");
    act(&mut ui, &mut store, MapAction::DialogDistance, "");
    act(&mut ui, &mut store, MapAction::SaveDialog, "Homy");
    let edited = store.get(home).unwrap();
    assert_eq!((edited.name.as_str(), edited.x, edited.y, edited.z), ("Homy", 5, 70, 6));
    assert_eq!(edited.colour, WaypointStore::PALETTE[5]);
    assert!(!edited.show_distance && edited.show_on_edge);
    assert_eq!(store.len(), 3);
}

#[test]
fn hiding_waypoints_flips_the_menu_label_and_says_so() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    assert_eq!(MenuItem::ToggleWaypoints.label(ui.waypoints_shown()), "Hide waypoints");
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::ToggleWaypoints);
    assert!(!ui.waypoints_shown());
    assert_eq!(ui.message().unwrap().text, "Waypoints hidden");
    assert_eq!(MenuItem::ToggleWaypoints.label(ui.waypoints_shown()), "Show waypoints");
    ui.reset();
    assert!(!ui.waypoints_shown());
}

#[test]
fn typed_surrogate_pairs_join_and_lone_halves_are_dropped() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    ui.text(u32::from('a'));
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::AddWaypoint);
    type_text(&mut ui, "A😀");
    ui.text(0xDC00);
    ui.text(0xD83D);
    ui.text(u32::from('b'));
    assert_eq!(drained(&mut ui), vec![UiEvent::Text('A'), UiEvent::Text('😀'), UiEvent::Text('b')]);
}

#[test]
fn a_save_on_a_full_world_keeps_the_dialog_and_says_why() {
    let mut store = WaypointStore::new(1, 1);
    store.add("Only", 0, (0, 64, 0), WaypointSource::Manual, NOW).unwrap();
    let mut ui = MapUi::new(1.0, 64);
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::AddWaypoint);
    act(&mut ui, &mut store, MapAction::SaveDialog, "Two");
    assert!(ui.dialog().is_some());
    assert_eq!(ui.message().unwrap().text, "This world already has 1 waypoints");
    assert_eq!(store.len(), 1);
}

#[test]
fn closing_the_map_drops_the_menu_and_dialog_but_keeps_the_message() {
    let (mut ui, mut store, home) = with_home();
    ui.open_menu(AT, MenuTarget::Marker(home));
    choose(&mut ui, &mut store, MenuItem::RemoveWaypoint);
    ui.open_menu(AT, GROUND);
    ui.close();
    assert!(!ui.is_open());
    assert!(ui.message().is_some());
    ui.reset();
    assert!(ui.message().is_none());
}

#[test]
fn the_export_items_hand_their_kind_to_the_runtime_once() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::ExportArea);
    assert!(!ui.is_open());
    assert_eq!(ui.take_export(), Some(ExportKind::Area));
    assert_eq!(ui.take_export(), None);
    ui.open_menu(AT, GROUND);
    choose(&mut ui, &mut store, MenuItem::ExportView);
    assert_eq!(ui.take_export(), Some(ExportKind::View));
}

fn apply(ui: &mut MapUi, view: &mut minimap_core::structs::view::ViewState, action: MapAction, dim: i32) {
    let mut store = WaypointStore::new(8, 1);
    ui.apply(action, "", dim, view, &mut store, NOW);
}

#[test]
fn slime_chunks_switch_only_in_the_overworld_and_the_grid_switches_anywhere() {
    let (mut ui, mut view) = (MapUi::new(1.0, 64), minimap_core::structs::view::ViewState::new(5));

    apply(&mut ui, &mut view, MapAction::ToggleSlime, 1);
    let in_nether = view.show_slime_chunks;
    apply(&mut ui, &mut view, MapAction::ToggleSlime, 0);
    apply(&mut ui, &mut view, MapAction::ToggleGrid, 2);

    assert!(!in_nether);
    assert!(view.show_slime_chunks && view.show_chunk_grid);
}

#[test]
fn centre_on_holds_the_map_on_the_point() {
    let (mut ui, mut view) = (MapUi::new(1.0, 64), minimap_core::structs::view::ViewState::new(5));

    apply(&mut ui, &mut view, MapAction::CentreOn { x: 40.5, z: 8.5 }, 0);

    assert_eq!(view.anchor, Some((40.5, 8.5)));
}

#[test]
fn the_list_stays_open_when_the_map_closes_and_closes_when_the_world_is_left() {
    let (mut ui, mut view) = (MapUi::new(1.0, 64), minimap_core::structs::view::ViewState::new(5));
    apply(&mut ui, &mut view, MapAction::ToggleList, 0);

    ui.close();
    let after_close = ui.list_open();
    ui.reset();

    assert!(after_close);
    assert!(!ui.list_open());
}

#[test]
fn keys_and_text_go_to_the_ui_queue_while_a_ui_field_has_the_keyboard() {
    use minimap_core::structs::ui::UiEvent;
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    ui.set_capture(false, true);

    let consumed = ui.key(key(0x57), &mut store);
    type_text(&mut ui, "w");

    assert!(consumed && ui.wants_text());
    assert_eq!(
        drained(&mut ui),
        vec![UiEvent::Key { code: 0x57, down: true, ctrl: false, shift: false }, UiEvent::Text('w')]
    );
}

#[test]
fn keys_and_text_reach_the_game_when_nothing_has_the_keyboard() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);

    let consumed = ui.key(key(0x57), &mut store);
    type_text(&mut ui, "w");

    assert!(!consumed && !ui.wants_text());
    assert!(drained(&mut ui).is_empty());
}

#[test]
fn a_wheel_turn_over_a_ui_panel_goes_to_the_ui_and_not_the_map() {
    use minimap_core::structs::ui::UiEvent;
    let mut ui = MapUi::new(1.0, 64);
    let wheel = MouseInput::new(MouseInput::WHEEL, -120);

    let off_panel = ui.wheel(wheel);
    ui.set_capture(true, false);
    let on_panel = ui.wheel(wheel);

    assert!(!off_panel && on_panel);
    assert_eq!(drained(&mut ui), vec![UiEvent::Wheel(-120)]);
}

#[test]
fn ctrl_z_undoes_at_once_even_while_a_ui_field_has_the_keyboard() {
    let (mut ui, mut store, home) = with_home();
    ui.open_menu(AT, MenuTarget::Marker(home));
    choose(&mut ui, &mut store, MenuItem::RemoveWaypoint);
    ui.set_capture(false, true);

    assert!(ui.key(ctrl(0x5A), &mut store));

    assert!(store.get(home).is_some());
    assert!(drained(&mut ui).is_empty());
}

#[test]
fn closing_the_map_drops_queued_ui_input_and_the_keyboard() {
    let mut store = WaypointStore::new(8, 1);
    let mut ui = MapUi::new(1.0, 64);
    ui.set_capture(false, true);
    ui.key(key(0x41), &mut store);

    ui.close();

    assert!(drained(&mut ui).is_empty());
    assert!(!ui.wants_text());
}
