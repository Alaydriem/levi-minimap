use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use minimap_core::config::Config;
use minimap_core::services::camera::SmoothCamera;
use minimap_core::services::cave::{LayerIds, LayerSelector};
use minimap_core::services::draw::{
    CaveBadgePainter, ChunkGridPainter, ExploredMaskCache, GpuResidency, MapRotation, MinimapComposer, MinimapPainter,
    SlimeOverlayPainter, TextScale, WaypointPainter, WorldMapLayout, WorldMapPainter,
};
use minimap_core::services::input::InputController;
use minimap_core::services::ui::WaypointFilter;
use minimap_core::structs::draw::{
    DrawList, ExploredChunks, MinimapFrame, ResidencyDecision, TextureOp, WorldMapFrame,
};
use minimap_core::structs::export::ExportKind;
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::tile::Tile;
use minimap_core::structs::ui::{HintKeys, HoverInfo, MapAction, MenuTarget, UiEvent, UiFrame, UiFrameInput};
use minimap_core::structs::view::{CameraPose, PlayerSnapshot, ViewState};
use minimap_core::structs::waypoint::Waypoint;
use minimap_ui::runtime::UiRuntime;
use minimap_ui::theme::FontSource;

use crate::services::{ExportService, HoverBiomeReader, WaypointSync};
use crate::structs::{FrameInput, FrameOutput, Shared};

// everything the render thread calls; it try-locks the shared map data and briefly locks the
// player, the view and the UI state
pub struct RenderSide {
    shared: Arc<Shared>,
    camera: SmoothCamera,
    composer: MinimapComposer,
    minimap: MinimapPainter,
    waypoint_painter: WaypointPainter,
    // the render thread's copy of the waypoints, refreshed when the store's revision moves
    waypoints: Vec<Waypoint>,
    waypoint_revision: Option<u64>,
    prev_right: bool,
    // the left button went down on the menu or dialog, so it does not pan the map until released
    left_to_ui: bool,
    exports: Arc<ExportService>,
    // the next world map frame is drawn without the menu or messages and captured
    capture_next: bool,
    // gives a cave layer's band its block range for the badge
    layers: LayerSelector,
    residency: GpuResidency,
    // which visible chunks have map data, for the overlays; keeps its capacity
    explored: ExploredChunks,
    explored_masks: ExploredMaskCache,
    input: InputController,
    regions: Vec<(RegionPos, Option<u32>)>,
    frame: u64,
    epoch: u64,
    minimap_uploaded: bool,
    prev_cursor: (f32, f32),
    prev_left: bool,
    font_size: f32,
    widget_scale: f32,
    // fontSize times the screen's text scale, for this frame
    text_size: f32,
    show_world_nether: bool,
    rotate_minimap: bool,
    rotate_world_map: bool,
    max_updates: usize,
    grid: ChunkGridPainter,
    slime: SlimeOverlayPainter,
    hover: HoverBiomeReader,
    filter: WaypointFilter,
    // indices into `waypoints` that the list shows this frame; keeps its capacity
    list_rows: Vec<usize>,
    ui: UiRuntime,
    // seconds since start, for the UI's animations
    ui_time: f64,
    // physical pixels per UI point this frame: the widget scale times the screen's text scale
    pixels_per_point: f32,
    // text for the clipboard from this frame's menu choice; keeps its capacity
    clipboard: String,
    // the queued input this frame hands the UI; keeps its capacity
    ui_events: Vec<UiEvent>,
    // this frame's world map panels: the player's dimension and the shown layer; None when the
    // panels are not drawn
    panel_dims: Option<(i32, i32)>,
    // the block under the cursor on the world map, when the cursor is on the map
    hovered: Option<(i32, i32)>,
    player_block: (i32, i32, i32),
    // the player's dimension this frame
    player_dim: i32,
    keys: HintKeys,
    // the panels' requests from this frame, applied after it; keeps its capacity
    actions: Vec<MapAction>,
}

impl RenderSide {
    const IDLE_CHECK_FRAMES: u64 = 300;
    // the cave badge's top, below the top of the minimap disc or the screen
    const BADGE_INSET: f32 = 8.0;
    // the height given to a waypoint added on a column that has not been explored
    pub const DEFAULT_Y: i32 = 64;

    pub fn new(shared: Arc<Shared>, config: &Config, exports: Arc<ExportService>) -> Self {
        let world = &config.world_map;
        Self {
            shared,
            camera: SmoothCamera::new(&config.smooth_camera),
            composer: MinimapComposer::new(),
            minimap: MinimapPainter::new(&config.mini_map),
            waypoint_painter: WaypointPainter::new(config),
            waypoints: Vec::new(),
            waypoint_revision: None,
            prev_right: false,
            left_to_ui: false,
            exports,
            capture_next: false,
            layers: LayerSelector::new(config),
            explored: ExploredChunks::new(),
            explored_masks: ExploredMaskCache::new(config.memory.gpu_texture_budget.max(1) as usize),
            residency: GpuResidency::new(
                config.memory.gpu_texture_budget.max(1) as usize,
                world.texture_idle_frames.max(1) as u64,
                world.max_new_textures_per_frame.max(1) as usize,
            ),
            input: InputController::new(config),
            regions: Vec::new(),
            frame: 0,
            epoch: 0,
            minimap_uploaded: false,
            prev_cursor: (0.0, 0.0),
            prev_left: false,
            font_size: config.font_size,
            widget_scale: config.mini_map.widget_scale,
            text_size: config.font_size,
            show_world_nether: world.show_nether_coordinates,
            rotate_minimap: config.mini_map.rotate_with_player,
            rotate_world_map: world.rotate_with_player,
            max_updates: world.max_new_textures_per_frame.max(1) as usize * 2,
            grid: ChunkGridPainter::new(&config.mini_map),
            slime: SlimeOverlayPainter::new(world.slime_cache_regions.max(1) as usize),
            hover: HoverBiomeReader::new(),
            filter: WaypointFilter::new(),
            list_rows: Vec::new(),
            ui: UiRuntime::new(FontSource::System),
            ui_time: 0.0,
            pixels_per_point: 1.0,
            clipboard: String::new(),
            ui_events: Vec::new(),
            panel_dims: None,
            hovered: None,
            player_block: (0, 0, 0),
            player_dim: 0,
            keys: HintKeys { toggle: world.toggle_key },
            actions: Vec::new(),
        }
    }

    pub fn gpu_textures(&self) -> usize {
        self.residency.len() + usize::from(self.minimap_uploaded)
    }

    pub fn frames(&self) -> u64 {
        self.frame
    }

    pub fn frame(&mut self, input: &FrameInput, out: &mut FrameOutput) {
        out.clear();
        self.frame += 1;
        self.panel_dims = None;
        self.hovered = None;
        self.ui_time += f64::from(input.dt);
        self.drop_textures_on_world_change(out);
        let player = *Shared::lock(&self.shared.player);
        if !player.present {
            self.prev_left = false;
            return;
        }
        self.player_dim = player.dim;
        let pose = self.camera.update(&player, input.dt);
        let text_scale = TextScale::for_display(input.display.1);
        self.text_size = self.font_size * text_scale;
        self.pixels_per_point = self.widget_scale * text_scale;
        Shared::lock(&self.shared.ui).set_scale(self.pixels_per_point);
        let presses = (
            input.foreground && input.left_down && !self.prev_left,
            input.foreground && input.right_down && !self.prev_right,
        );
        if !input.left_down {
            self.left_to_ui = false;
        }
        let view = {
            let mut view = Shared::lock(&self.shared.view);
            if view.world_map_open && input.foreground && input.left_down && self.prev_left && !self.left_to_ui {
                let (dx, dy) = (input.cursor.0 - self.prev_cursor.0, input.cursor.1 - self.prev_cursor.1);
                self.input.on_drag(&mut view, dx, dy, Self::angle(self.rotate_world_map, pose));
            }
            *view
        };
        self.prev_cursor = input.cursor;
        self.prev_left = input.left_down && input.foreground;
        self.prev_right = input.right_down && input.foreground;
        self.refresh_waypoints();
        if view.world_map_open {
            self.world_map(input, &player, pose, &view, presses, out);
        } else if view.show_minimap {
            self.minimap(input, &player, pose, &view, out);
        }
        self.paint_ui(input, view.world_map_open, out);
        self.run_ui(input, out);
    }

    // the map layers and the panels through the UI into one mesh, then the panels' requests are
    // applied; a capture frame has no cursor and no panels
    fn run_ui(&mut self, input: &FrameInput, out: &mut FrameOutput) {
        let started = Instant::now();
        let capture = out.cursor_flags & FrameOutput::CAPTURE_VIEW != 0;
        let ui_input = UiFrameInput {
            display: input.display,
            pixels_per_point: self.pixels_per_point,
            time_s: self.ui_time,
            cursor: input.cursor,
            left_down: input.left_down,
            right_down: input.right_down,
            foreground: input.foreground,
            events: &self.ui_events,
            draw_cursor: out.cursor_flags & FrameOutput::DRAW_CURSOR != 0 && !capture,
            collect_text: input.collect_text,
        };
        let panels = self.panel_dims.filter(|_| !capture);
        let dim = panels.map_or(self.player_dim, |(dim, _)| dim);
        {
            let ui = Shared::lock(&self.shared.ui);
            let view = *Shared::lock(&self.shared.view);
            let biome = match (panels, self.hovered) {
                (Some((_, layer)), Some(block)) => self.hover.read(&self.shared, block, layer),
                _ => None,
            };
            let hover = self.hovered.map(|block| HoverInfo {
                block,
                chunk: (block.0.div_euclid(16), block.1.div_euclid(16)),
                biome,
            });
            let frame = (!capture).then(|| UiFrame {
                panels: panels.is_some(),
                ui: &ui,
                view: &view,
                waypoints: &self.waypoints,
                rows: &self.list_rows,
                in_dimension: WaypointFilter::count_in(&self.waypoints, dim),
                player_block: self.player_block,
                dim,
                hover,
                keys: self.keys,
            });
            let actions = self.ui.run(&ui_input, &out.draw, frame.as_ref(), &mut out.mesh);
            self.actions.clear();
            self.actions.extend_from_slice(actions);
        }
        self.apply_actions(dim);
        {
            let mut ui = Shared::lock(&self.shared.ui);
            ui.set_capture(out.mesh.wants_pointer, out.mesh.wants_keyboard);
            if let Some(text) = ui.take_clipboard() {
                self.clipboard = text;
            }
        }
        if !self.clipboard.is_empty() {
            out.mesh.clipboard.clone_from(&self.clipboard);
            self.clipboard.clear();
        }
        out.ui_build_us = u32::try_from(started.elapsed().as_micros()).unwrap_or(u32::MAX);
    }

    // the message line over either map; the menu and the dialog only while the world map is open
    fn paint_ui(&mut self, input: &FrameInput, map_open: bool, out: &mut FrameOutput) {
        let mut ui = Shared::lock(&self.shared.ui);
        ui.tick(input.dt);
        self.ui_events.clear();
        ui.drain_events(&mut self.ui_events);
        while let Some(message) = self.exports.take_result() {
            if message.starts_with(ExportService::EXPORTED) {
                ui.show_done(message);
            } else {
                ui.show_message(message);
            }
        }
        if std::mem::take(&mut self.capture_next) && map_open {
            out.cursor_flags |= FrameOutput::CAPTURE_VIEW;
            return;
        }
        match ui.take_export() {
            Some(ExportKind::View) if self.exports.is_busy() => {
                ui.show_message("An export is already running".to_string())
            }
            Some(ExportKind::View) => self.capture_next = true,
            Some(ExportKind::Area) => self.shared.area_export.store(true, Ordering::SeqCst),
            None => {}
        }
        if !map_open && ui.is_open() {
            ui.close();
        }
        if map_open && ui.wants_text() {
            out.cursor_flags |= FrameOutput::TEXT_INPUT;
        }
    }

    fn apply_actions(&mut self, dim: i32) {
        if self.actions.is_empty() {
            return;
        }
        let mut ui = Shared::lock(&self.shared.ui);
        let mut view = Shared::lock(&self.shared.view);
        let mut store = Shared::lock(&self.shared.waypoints);
        let name = &self.ui.drafts().name;
        for action in &self.actions {
            ui.apply(*action, name, dim, &mut view, &mut store, WaypointSync::now());
        }
    }

    // the column under a world map screen point; y is its stored height, or DEFAULT_Y when not explored
    // `layer` is the map layer whose stored height is read; the target keeps the real dimension
    fn ground(shared: &Shared, frame: &WorldMapFrame<'_>, layer: i32, at: (f32, f32)) -> MenuTarget {
        let (x, z) = WorldMapLayout::block_at(at, frame.centre, frame.zoom, frame.display, frame.angle);
        let height = shared.chunks.try_read().ok().and_then(|chunks| chunks.column_height(x, z, layer));
        MenuTarget::Ground { dim: frame.dim, x, y: height.map_or(Self::DEFAULT_Y, i32::from), z }
    }

    // the badge of the shown cave band, if one is shown
    fn cave_badge(&self, top_centre: (f32, f32), draw: &mut DrawList) {
        let layer = self.shared.layer.load(Ordering::Relaxed);
        Self::badge_for(&self.layers, layer, top_centre, self.text_size, draw);
    }

    fn badge_for(layers: &LayerSelector, layer: i32, top_centre: (f32, f32), size: f32, draw: &mut DrawList) {
        if let Some(band) = LayerIds::band(layer) {
            CaveBadgePainter::paint(draw, top_centre, size, (layers.band_bottom(band), layers.band_top(band)));
        }
    }

    // the map turn for a map with `rotate` set: the player's facing at the top
    fn angle(rotate: bool, pose: CameraPose) -> f32 {
        if rotate { MapRotation::player_up(pose.render_yaw_rad()) } else { 0.0 }
    }

    // copies the list only after a change; a busy lock keeps last frame's copy
    fn refresh_waypoints(&mut self) {
        let Ok(store) = self.shared.waypoints.try_lock() else { return };
        if self.waypoint_revision != Some(store.revision()) {
            self.waypoint_revision = Some(store.revision());
            self.waypoints.clear();
            self.waypoints.extend_from_slice(store.waypoints());
        }
    }

    fn drop_textures_on_world_change(&mut self, out: &mut FrameOutput) {
        let epoch = self.shared.world_epoch.load(Ordering::SeqCst);
        if epoch == self.epoch {
            return;
        }
        self.epoch = epoch;
        self.residency.release_all(&mut out.textures);
        if self.minimap_uploaded {
            out.textures.push(TextureOp::Release { tex: GpuResidency::MINIMAP_TEXTURE });
            self.minimap_uploaded = false;
        }
        self.composer.invalidate();
        self.explored_masks.clear();
        self.camera.reset();
        self.hover.reset();
        self.ui.reset_drafts();
    }

    fn minimap(
        &mut self,
        input: &FrameInput,
        player: &PlayerSnapshot,
        pose: CameraPose,
        view: &ViewState,
        out: &mut FrameOutput,
    ) {
        let centre = (pose.x.floor() as i32, pose.z.floor() as i32);
        if let Ok(tiles) = self.shared.tiles.try_lock() {
            let chunks = self.shared.chunks.try_read().ok();
            let layer = self.shared.layer.load(Ordering::Relaxed);
            if self.composer.update(centre, layer, view.minimap_radius, &tiles, chunks.as_deref()) {
                let side = self.composer.side() as u32;
                let buffer = out.stage(self.composer.image());
                out.textures.push(TextureOp::Upload {
                    tex: GpuResidency::MINIMAP_TEXTURE,
                    width: side,
                    height: side,
                    buffer,
                });
                self.minimap_uploaded = true;
            }
        }
        let frame = MinimapFrame {
            display_width: input.display.0,
            camera: pose,
            player_block: player.block(),
            radius_chunks: view.minimap_radius,
            font_size: self.text_size,
            texture: self.minimap_uploaded.then_some(GpuResidency::MINIMAP_TEXTURE),
            image_origin: self.composer.origin(),
            image_side: self.composer.side(),
            dim: player.dim,
            show_coordinates: view.show_coordinates,
            angle: Self::angle(self.rotate_minimap, pose),
        };
        self.minimap.paint(&frame, &mut out.draw);
        let disc = self.minimap.disc(input.display.0);
        self.cave_badge((disc.0.0, disc.0.1 - disc.1 + Self::BADGE_INSET), &mut out.draw);
        let shown = Shared::lock(&self.shared.ui).waypoints_shown();
        let waypoints: &[Waypoint] = if shown { &self.waypoints } else { &[] };
        self.waypoint_painter.paint_minimap(&frame, disc, waypoints, &mut out.draw);
    }

    fn world_map(
        &mut self,
        input: &FrameInput,
        player: &PlayerSnapshot,
        pose: CameraPose,
        view: &ViewState,
        presses: (bool, bool),
        out: &mut FrameOutput,
    ) {
        out.cursor_flags = FrameOutput::DRAW_CURSOR | FrameOutput::RELEASE_CLIP;
        let centre = view.centre((pose.x, pose.z));
        let angle = Self::angle(self.rotate_world_map, pose);
        let layer = self.shared.layer.load(Ordering::Relaxed);
        let (rx0, rx1, rz0, rz1) = if angle == 0.0 {
            WorldMapLayout::visible(centre, view.zoom, input.display)
        } else {
            WorldMapLayout::visible_turned(centre, view.zoom, input.display)
        };
        self.residency.begin_frame(self.frame);
        self.explored.begin((rx0, rz0), rx1 - rx0 + 1);
        self.regions.clear();
        let mut updates = 0;
        {
            let tiles = self.shared.tiles.try_lock().ok();
            let known_empty = self.shared.known_empty.try_lock().ok();
            for rz in rz0..=rz1 {
                for rx in rx0..=rx1 {
                    let region = RegionPos::new(rx, rz, layer);
                    let tile = tiles.as_ref().and_then(|t| t.peek(region));
                    self.explored.push(self.explored_masks.mask(region, tile));
                    let decision = self.residency.decide(region, tile.map(Tile::version), &mut out.textures);
                    let tex = match (decision, tile) {
                        (ResidencyDecision::Draw(tex), _) => Some(tex),
                        (ResidencyDecision::Upload { tex, created }, Some(tile))
                            if created || updates < self.max_updates =>
                        {
                            let buffer = out.stage(tile.pixels());
                            out.textures.push(TextureOp::Upload { tex, width: 256, height: 256, buffer });
                            self.residency.uploaded(region, tile.version());
                            updates += 1;
                            Some(tex)
                        }
                        (ResidencyDecision::Upload { tex, .. }, _) => Some(tex),
                        (ResidencyDecision::Placeholder, None) => {
                            if known_empty.as_ref().is_some_and(|k| !k.contains(region)) {
                                let _ = self.shared.tile_requests.0.try_send(region);
                            }
                            None
                        }
                        (ResidencyDecision::Placeholder, Some(_)) => None,
                    };
                    self.regions.push((region, tex));
                }
            }
        }
        if self.frame.is_multiple_of(Self::IDLE_CHECK_FRAMES) {
            self.residency.release_idle(&mut out.textures);
        }
        let frame = WorldMapFrame {
            display: input.display,
            centre,
            zoom: view.zoom,
            camera: pose,
            player_block: player.block(),
            font_size: self.text_size,
            show_coordinates: view.show_coordinates,
            show_nether_coordinates: self.show_world_nether,
            dim: player.dim,
            angle,
            regions: &self.regions,
        };
        WorldMapPainter::paint(&frame, &mut out.draw);
        let over_panel = self.ui.covers(input.cursor, self.pixels_per_point);
        // a captured view keeps the overlays but not the panels or the hover
        let tools_shown = input.foreground && !self.capture_next;
        let hovered = (tools_shown && !over_panel)
            .then(|| WorldMapLayout::block_at(input.cursor, centre, view.zoom, input.display, angle));
        if view.show_slime_chunks {
            self.slime.paint(&frame, &self.explored, &mut out.draw);
        }
        if view.show_chunk_grid {
            let chunk = hovered.map(|(x, z)| (x.div_euclid(16), z.div_euclid(16)));
            self.grid.paint(&frame, chunk, &self.explored, &mut out.draw);
        }
        let shown = Shared::lock(&self.shared.ui).waypoints_shown();
        let waypoints: &[Waypoint] = if shown { &self.waypoints } else { &[] };
        self.waypoint_painter.paint_world_map(&frame, waypoints, &mut out.draw);
        Self::badge_for(&self.layers, layer, (input.display.0 * 0.5, Self::BADGE_INSET), self.text_size, &mut out.draw);
        let (left_pressed, right_pressed) = presses;
        if right_pressed && !over_panel {
            let target = match self.waypoint_painter.hit(&frame, waypoints, input.cursor) {
                Some(id) => MenuTarget::Marker(id),
                None => Self::ground(&self.shared, &frame, layer, input.cursor),
            };
            Shared::lock(&self.shared.ui).open_menu(input.cursor, target);
        }
        if tools_shown {
            self.filter.set_query(self.ui.search());
            self.filter.collect(&self.waypoints, player.dim, &mut self.list_rows);
            self.panel_dims = Some((player.dim, layer));
            self.hovered = hovered;
            self.player_block = player.block();
        }
        if left_pressed {
            // an open menu or dialog takes every press, as do the panels
            self.left_to_ui = over_panel || Shared::lock(&self.shared.ui).is_open();
        }
    }
}
