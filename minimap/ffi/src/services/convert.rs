use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_runtime::structs::{CursorAction, ProbeRequest, RuntimeStats, TickInput};

use crate::types::{
    LmBlockColor, LmChunkPos, LmChunkSurface, LmColumnSample, LmProbeRequest, LmStats, LmStr, LmTickInput,
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
            mouse_grabbed: input.mouse_grabbed != 0,
            working_set_bytes: input.working_set_bytes,
            private_bytes: input.private_bytes,
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

    // the name points into the runtime's config, which lives as long as the handle
    pub fn block_color((name, colour): &(String, u32)) -> LmBlockColor {
        LmBlockColor { name: LmStr { ptr: name.as_ptr(), len: name.len() }, colour: *colour }
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
