use crate::errors::WaypointError;
use crate::services::ui::DimensionName;
use crate::services::waypoints::WaypointStore;
use crate::structs::export::ExportKind;
use crate::structs::input::{KeyInput, MouseInput};
use crate::structs::ui::{
    DialogTarget, MapAction, MapMenu, MenuItem, MenuTarget, MessageKind, MessageLine, UiEvent, UiEventQueue,
    WaypointDialog,
};
use crate::structs::view::ViewState;
use crate::structs::waypoint::{Waypoint, WaypointEdit, WaypointSource};

// the world map's right-click menu, the waypoint dialog, the message line, the waypoint list switch
// and the last removal. The game thread gives it keys and typed text, which it queues for the UI;
// the render thread gives it the UI's actions
pub struct MapUi {
    scale: f32,
    menu: Option<MapMenu>,
    dialog: Option<WaypointDialog>,
    // counts dialogs opened, so the UI starts each from its own name
    dialog_serial: u64,
    message: Option<MessageLine>,
    // the last removal and where it was, while its message shows
    undo: Option<(usize, Waypoint)>,
    clipboard: Option<String>,
    waypoints_shown: bool,
    // an export chosen from the menu, until the runtime takes it
    export: Option<ExportKind>,
    // the first half of a UTF-16 surrogate pair, until the second arrives
    high_surrogate: Option<u32>,
    // the world map's waypoint list is shown
    list_open: bool,
    // keys, text and wheel turns for the UI's next frame
    events: UiEventQueue,
    // the UI's last frame had the pointer over a panel, and a text field with the keyboard
    pointer_captured: bool,
    keyboard_captured: bool,
}

impl MapUi {
    pub const MESSAGE_SECONDS: f32 = 5.0;
    pub const ESCAPE: i32 = 0x1B;
    pub const Z: i32 = 0x5A;

    pub fn new(scale: f32, event_capacity: usize) -> Self {
        Self {
            scale,
            menu: None,
            dialog: None,
            dialog_serial: 0,
            message: None,
            undo: None,
            clipboard: None,
            waypoints_shown: true,
            export: None,
            high_surrogate: None,
            list_open: false,
            events: UiEventQueue::new(event_capacity),
            pointer_captured: false,
            keyboard_captured: false,
        }
    }

    // the render thread reports where the UI's last frame had the pointer and the keyboard
    pub fn set_capture(&mut self, pointer: bool, keyboard: bool) {
        self.pointer_captured = pointer;
        self.keyboard_captured = keyboard;
    }

    pub fn pointer_captured(&self) -> bool {
        self.pointer_captured
    }

    pub fn drain_events(&mut self, out: &mut Vec<UiEvent>) {
        self.events.drain_into(out);
    }

    // events dropped because the UI did not take them in time
    pub fn dropped_events(&self) -> u64 {
        self.events.dropped()
    }

    pub fn list_open(&self) -> bool {
        self.list_open
    }

    // a wheel turn over a UI panel goes to the UI; true when the map must not zoom
    pub fn wheel(&mut self, input: MouseInput) -> bool {
        if input.button != MouseInput::WHEEL || input.data == 0 || !self.pointer_captured {
            return false;
        }
        self.events.push(UiEvent::Wheel(input.data));
        true
    }

    // a panel's request, after the UI frame that made it; `name` is the dialog's typed name and
    // `dim` the player's dimension
    pub fn apply(
        &mut self,
        action: MapAction,
        name: &str,
        dim: i32,
        view: &mut ViewState,
        store: &mut WaypointStore,
        now: u64,
    ) {
        match action {
            MapAction::ToggleGrid => view.show_chunk_grid = !view.show_chunk_grid,
            MapAction::ToggleSlime if dim == DimensionName::OVERWORLD => {
                view.show_slime_chunks = !view.show_slime_chunks
            }
            MapAction::ToggleSlime => {}
            MapAction::ToggleList => self.list_open = !self.list_open,
            MapAction::CentreOn { x, z } => view.anchor = Some((x, z)),
            MapAction::Choose(item) => {
                if let Some(menu) = self.menu.take() {
                    self.choose(item, menu.target, store);
                }
            }
            MapAction::CloseMenu => self.menu = None,
            MapAction::DialogColour(colour) => {
                if let Some(dialog) = self.dialog.as_mut() {
                    dialog.colour = colour;
                }
            }
            MapAction::DialogEdge => {
                if let Some(dialog) = self.dialog.as_mut() {
                    dialog.show_on_edge = !dialog.show_on_edge;
                }
            }
            MapAction::DialogDistance => {
                if let Some(dialog) = self.dialog.as_mut() {
                    dialog.show_distance = !dialog.show_distance;
                }
            }
            MapAction::SaveDialog => self.save(name, store, now),
            MapAction::CancelDialog => self.dialog = None,
        }
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    // the render thread sets it each frame: the widget scale times the screen's text scale
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn menu(&self) -> Option<&MapMenu> {
        self.menu.as_ref()
    }

    pub fn dialog(&self) -> Option<&WaypointDialog> {
        self.dialog.as_ref()
    }

    pub fn message(&self) -> Option<&MessageLine> {
        self.message.as_ref()
    }

    pub fn waypoints_shown(&self) -> bool {
        self.waypoints_shown
    }

    // the dialog or a UI text field has the keyboard: typed characters go to it
    pub fn wants_text(&self) -> bool {
        self.dialog.is_some() || self.keyboard_captured
    }

    // a menu or the dialog takes left clicks instead of the map
    pub fn is_open(&self) -> bool {
        self.menu.is_some() || self.dialog.is_some()
    }

    // a right click on the map; ignored while the dialog is open
    pub fn open_menu(&mut self, at: (f32, f32), target: MenuTarget) {
        if self.dialog.is_none() {
            self.menu = Some(MapMenu::new(at, target));
        }
    }

    // a key down; true when the game and the map must not see it. Ctrl+Z works while the undo
    // lasts; while the dialog is open every key is its own and goes to the UI in order with the
    // typed text
    pub fn key(&mut self, input: KeyInput, store: &mut WaypointStore) -> bool {
        if !input.down {
            return false;
        }
        if input.ctrl && input.code == Self::Z && self.undo.is_some() {
            self.undo(store);
            return true;
        }
        if self.menu.is_some() && input.code == Self::ESCAPE {
            self.menu = None;
            return true;
        }
        if self.dialog.is_some() || self.keyboard_captured {
            self.events.push(UiEvent::Key { code: input.code, down: true, ctrl: input.ctrl, shift: false });
            return true;
        }
        false
    }

    // one UTF-16 unit of typed text, as WM_CHAR gives it; pairs are joined
    pub fn text(&mut self, unit: u32) {
        if !self.wants_text() {
            return;
        }
        let code = match (unit, self.high_surrogate.take()) {
            (0xD800..=0xDBFF, _) => {
                self.high_surrogate = Some(unit);
                return;
            }
            (0xDC00..=0xDFFF, Some(high)) => 0x10000 + ((high - 0xD800) << 10) + (unit - 0xDC00),
            (0xDC00..=0xDFFF, None) => return,
            (unit, _) => unit,
        };
        if let Some(c) = char::from_u32(code) {
            self.events.push(UiEvent::Text(c));
        }
    }

    // a note, or a refusal or error
    pub fn show_message(&mut self, text: String) {
        self.message = Some(MessageLine { text, remaining: Self::MESSAGE_SECONDS, kind: MessageKind::Info });
    }

    // a note that something the player asked for was done
    pub fn show_done(&mut self, text: String) {
        self.message = Some(MessageLine { text, remaining: Self::MESSAGE_SECONDS, kind: MessageKind::Done });
    }

    // the message and the undo it offers run out together
    pub fn tick(&mut self, dt: f32) {
        if let Some(message) = self.message.as_mut() {
            message.remaining -= dt;
            if message.remaining <= 0.0 {
                self.message = None;
                self.undo = None;
            }
        }
    }

    pub fn take_export(&mut self) -> Option<ExportKind> {
        self.export.take()
    }

    pub fn take_clipboard(&mut self) -> Option<String> {
        self.clipboard.take()
    }

    // the world map closed
    pub fn close(&mut self) {
        self.menu = None;
        self.dialog = None;
        self.high_surrogate = None;
        self.events.clear();
        self.keyboard_captured = false;
    }

    // the world was left; whether waypoints are shown is kept
    pub fn reset(&mut self) {
        self.close();
        self.message = None;
        self.undo = None;
        self.clipboard = None;
        self.export = None;
        self.list_open = false;
    }

    fn open_dialog(&mut self, dialog: impl FnOnce(u64) -> WaypointDialog) {
        self.dialog_serial += 1;
        self.dialog = Some(dialog(self.dialog_serial));
    }

    fn choose(&mut self, item: MenuItem, target: MenuTarget, store: &mut WaypointStore) {
        match (item, target) {
            (MenuItem::AddWaypoint, MenuTarget::Ground { dim, x, y, z }) => {
                let colour = store.next_colour();
                self.open_dialog(|serial| WaypointDialog {
                    target: DialogTarget::New { dim, x, y, z },
                    initial_name: String::new(),
                    serial,
                    colour,
                    show_on_edge: true,
                    show_distance: true,
                    position: (x, y, z),
                });
            }
            (MenuItem::CopyCoordinates, MenuTarget::Ground { x, z, .. }) => self.copy(format!("{x}, ~, {z}")),
            (MenuItem::CopyCoordinates, MenuTarget::Marker(id)) => {
                if let Some(w) = store.get(id) {
                    self.copy(format!("{}, {}, {}", w.x, w.y, w.z));
                }
            }
            (MenuItem::ToggleWaypoints, _) => {
                self.waypoints_shown = !self.waypoints_shown;
                let state = if self.waypoints_shown { "shown" } else { "hidden" };
                self.show_done(format!("Waypoints {state}"));
            }
            (MenuItem::ExportView, _) => self.export = Some(ExportKind::View),
            (MenuItem::ExportArea, _) => self.export = Some(ExportKind::Area),
            (MenuItem::EditWaypoint, MenuTarget::Marker(id)) => {
                if let Some(w) = store.get(id).cloned() {
                    self.open_dialog(|serial| WaypointDialog {
                        target: DialogTarget::Edit(id),
                        initial_name: w.name,
                        serial,
                        colour: w.colour,
                        show_on_edge: w.show_on_edge,
                        show_distance: w.show_distance,
                        position: (w.x, w.y, w.z),
                    });
                }
            }
            (MenuItem::RemoveWaypoint, MenuTarget::Marker(id)) => {
                if let Some((index, removed)) = store.remove(id) {
                    self.show_message(format!("Removed \"{}\" - press Ctrl+Z to undo", removed.name));
                    self.undo = Some((index, removed));
                }
            }
            _ => {}
        }
    }

    fn copy(&mut self, text: String) {
        self.show_done(format!("Copied {text}"));
        self.clipboard = Some(text);
    }

    fn undo(&mut self, store: &mut WaypointStore) {
        let Some((index, waypoint)) = self.undo.take() else { return };
        let name = waypoint.name.clone();
        if store.restore(index, waypoint) {
            self.show_done(format!("Restored \"{name}\""));
        }
    }

    // a refused add keeps the dialog open behind the message
    fn save(&mut self, name: &str, store: &mut WaypointStore, now: u64) {
        let Some(dialog) = self.dialog.as_ref() else { return };
        let edit = WaypointEdit {
            name: name.chars().take(WaypointStore::MAX_NAME_CHARS).collect(),
            colour: dialog.colour,
            show_on_edge: dialog.show_on_edge,
            show_distance: dialog.show_distance,
        };
        match dialog.target {
            DialogTarget::New { dim, x, y, z } => {
                match store.add(&edit.name, dim, (x, y, z), WaypointSource::Manual, now) {
                    Ok(id) => {
                        store.update(id, &edit);
                        self.dialog = None;
                    }
                    Err(WaypointError::Full(cap)) => {
                        self.show_message(format!("This world already has {cap} waypoints"))
                    }
                }
            }
            DialogTarget::Edit(id) => {
                store.update(id, &edit);
                self.dialog = None;
            }
        }
    }
}
