use minimap_core::services::ui::{MapUi, WaypointFilter};
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::draw::DrawList;
use minimap_core::structs::mesh::UiMesh;
use minimap_core::structs::ui::{HintKeys, MapAction, UiEvent, UiFrame};
use minimap_core::structs::view::ViewState;
use minimap_core::structs::waypoint::{Waypoint, WaypointSource};
use minimap_ui::runtime::UiRuntime;
use minimap_ui::theme::FontSource;

use crate::support::Frames;

// a world map with the panels: seven named waypoints in the Overworld, the player at (214, 70, -88)
pub struct Scene {
    pub runtime: UiRuntime,
    pub ui: MapUi,
    pub view: ViewState,
    pub waypoints: Vec<Waypoint>,
    pub rows: Vec<usize>,
    pub dim: i32,
    pub mesh: UiMesh,
    pub actions: Vec<MapAction>,
}

impl Scene {
    pub const PLAYER: (i32, i32, i32) = (214, 70, -88);
    pub const NAMES: [(&'static str, i32, i32); 7] = [
        ("Base", 214, -88),
        ("Iron farm", 342, -31),
        ("Village", 64, 40),
        ("Nether portal", 290, -170),
        ("Slime farm", 120, -150),
        ("Shipwreck", 10, -120),
        ("Death point", 305, 70),
    ];

    pub fn new(list_open: bool) -> Self {
        let mut store = WaypointStore::new(16, 1);
        for (name, x, z) in Self::NAMES {
            let _ = store.add(name, 0, (x, 70, z), WaypointSource::Manual, 0);
        }
        let mut ui = MapUi::new(1.0, 64);
        let mut view = ViewState::new(5);
        view.zoom = 2.5;
        if list_open {
            ui.apply(MapAction::ToggleList, "", 0, &mut view, &mut store, 0);
        }
        let waypoints = store.waypoints().to_vec();
        let rows = (0..waypoints.len()).collect();
        Self {
            runtime: UiRuntime::new(FontSource::BuiltIn),
            ui,
            view,
            waypoints,
            rows,
            dim: 0,
            mesh: UiMesh::new(),
            actions: Vec::new(),
        }
    }

    // one frame with the cursor at `cursor`; the actions it returned are kept
    pub fn frame(&mut self, cursor: (f32, f32), left_down: bool, events: &[UiEvent]) {
        let mut filter = WaypointFilter::new();
        filter.set_query(self.runtime.search());
        filter.collect(&self.waypoints, self.dim, &mut self.rows);
        let frame = UiFrame {
            panels: true,
            ui: &self.ui,
            view: &self.view,
            waypoints: &self.waypoints,
            rows: &self.rows,
            in_dimension: WaypointFilter::count_in(&self.waypoints, self.dim),
            player_block: Self::PLAYER,
            dim: self.dim,
            hover: None,
            keys: HintKeys { toggle: 0x4D },
        };
        let input = Frames::input(cursor, left_down, events);
        let actions = self.runtime.run(&input, &DrawList::new(), Some(&frame), &mut self.mesh);
        self.actions.clear();
        self.actions.extend_from_slice(actions);
    }

    // two frames, so new panels are laid out and visible
    pub fn settle(&mut self) {
        self.frame((1200.0, 700.0), false, &[]);
        self.frame((1200.0, 700.0), false, &[]);
    }

    pub fn centre_of(&self, text: &str) -> (f32, f32) {
        let found = self.mesh.text_rects.iter().rev().find(|t| self.mesh.text_of(t) == text);
        let rect = found.map(|t| t.rect).unwrap_or_else(|| panic!("{text} not drawn"));
        ((rect[0] + rect[2]) * 0.5, (rect[1] + rect[3]) * 0.5)
    }

    // moves onto `text`, presses and releases; returns the actions of the release frame
    pub fn click(&mut self, text: &str) -> Vec<MapAction> {
        let at = self.centre_of(text);
        self.frame(at, false, &[]);
        self.frame(at, true, &[]);
        self.frame(at, false, &[]);
        self.actions.clone()
    }
}
