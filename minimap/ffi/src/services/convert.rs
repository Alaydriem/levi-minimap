use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::draw::TextureOp;
use minimap_core::structs::input::{KeyResult, MouseAction};
use minimap_core::structs::mesh::UiMesh;
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_runtime::structs::{CursorAction, FrameInput, FrameOutput, ProbeRequest, RuntimeStats, TickInput};

use crate::types::{
    LmBlockColor, LmChunkPos, LmChunkSurface, LmColumnSample, LmFrameInput, LmKeyResult, LmProbeRequest, LmStats,
    LmStr, LmTextureOp, LmTickInput, LmUiText, LmUiTextureDelta,
};

// conversions between the C types and the core types, each in one place
pub struct LmConvert;

impl LmConvert {
    pub fn chunk_pos(pos: &LmChunkPos) -> ChunkPos {
        ChunkPos::new(pos.x, pos.z, pos.dim)
    }

    pub fn lm_chunk_pos(pos: ChunkPos) -> LmChunkPos {
        LmChunkPos { x: pos.x, z: pos.z, dim: pos.dim }
    }

    pub fn probe(surface: &LmChunkSurface) -> ChunkProbe {
        bytemuck::cast(*surface)
    }

    pub fn column(sample: &LmColumnSample) -> ColumnSample {
        ColumnSample::new(sample.surface, sample.water, sample.height, sample.solid, sample.retry != 0, sample.biome)
    }

    pub fn tick_input(input: &LmTickInput) -> TickInput {
        TickInput {
            player: PlayerSnapshot {
                present: input.present != 0,
                x: input.x,
                y: input.y,
                z: input.z,
                yaw: input.yaw,
                dim: input.dim,
            },
            foreground: input.foreground != 0,
            cursor_hidden: input.cursor_hidden != 0,
            dead: input.dead != 0,
        }
    }

    pub fn cursor(action: CursorAction) -> u8 {
        match action {
            CursorAction::None => 0,
            CursorAction::ClipToCentre => 1,
            CursorAction::Release => 2,
        }
    }

    pub fn probe_request(request: ProbeRequest) -> LmProbeRequest {
        match request {
            ProbeRequest::None => LmProbeRequest::default(),
            ProbeRequest::Column { pos, index } => {
                LmProbeRequest { kind: 1, reserved: [0; 3], x: pos.x, z: pos.z, dim: pos.dim, index: index as u32 }
            }
        }
    }

    pub fn key_result(result: KeyResult) -> LmKeyResult {
        let action = match result.action {
            MouseAction::None => 0,
            MouseAction::ReleaseMouse => 1,
            MouseAction::GrabMouse => 2,
        };
        LmKeyResult { consume: u8::from(result.consume), action }
    }

    // the name points into the runtime's config, which lives as long as the handle
    pub fn block_color((name, colour): &(String, u32)) -> LmBlockColor {
        LmBlockColor { name: LmStr { ptr: name.as_ptr(), len: name.len() }, colour: *colour }
    }

    pub fn frame_input(input: &LmFrameInput) -> FrameInput {
        FrameInput {
            display: (input.display_width, input.display_height),
            dt: input.dt,
            cursor: (input.cursor_x, input.cursor_y),
            left_down: input.left_down != 0,
            right_down: input.right_down != 0,
            foreground: input.foreground != 0,
            render_us: input.render_us,
            ui_vtx: input.ui_vtx,
            working_set_bytes: input.working_set_bytes,
            private_bytes: input.private_bytes,
            collect_text: input.collect_text != 0,
        }
    }

    pub fn textures(output: &FrameOutput, out: &mut Vec<LmTextureOp>) {
        out.clear();
        for op in &output.textures {
            out.push(match *op {
                TextureOp::Upload { tex, width, height, buffer } => {
                    let pixels = &output.buffers[buffer as usize];
                    LmTextureOp { kind: 1, tex, width, height, pixels: pixels.as_ptr(), len: pixels.len() }
                }
                TextureOp::Release { tex } => {
                    LmTextureOp { kind: 2, tex, width: 0, height: 0, pixels: std::ptr::null(), len: 0 }
                }
            });
        }
    }

    // the UI texture uploads, pointing into the mesh's pixel buffer
    pub fn ui_textures(mesh: &UiMesh, out: &mut Vec<LmUiTextureDelta>) {
        out.clear();
        out.extend(mesh.textures.iter().map(|t| LmUiTextureDelta {
            texture: t.texture,
            full: u8::from(t.full),
            reserved: [0; 3],
            x: t.x,
            y: t.y,
            width: t.width,
            height: t.height,
            pixels: mesh.pixels.get(t.offset..).map_or(std::ptr::null(), <[u8]>::as_ptr),
            len: t.len,
        }));
    }

    pub fn ui_texts(mesh: &UiMesh, out: &mut Vec<LmUiText>) {
        out.clear();
        out.extend(mesh.text_rects.iter().map(|t| LmUiText { start: t.start, len: t.len, rect: t.rect }));
    }

    pub fn stats(stats: RuntimeStats) -> LmStats {
        LmStats {
            in_world: u8::from(stats.in_world),
            ticks: stats.ticks,
            frames: stats.frames,
            chunks_submitted: stats.chunks_submitted,
            bakes_completed: stats.bakes_completed,
            tiles_loaded: stats.tiles_loaded,
            surfaces_resident: stats.surfaces_resident as u64,
            surface_capacity: stats.surface_capacity as u64,
            tiles_resident: stats.tiles_resident as u64,
            tile_capacity: stats.tile_capacity as u64,
            tile_overrun: stats.tile_overrun as u64,
            gpu_textures: stats.gpu_textures as u64,
            bake_queue: stats.bake_queue as u64,
            outbox: stats.outbox as u64,
            deferred: stats.deferred as u64,
        }
    }
}
