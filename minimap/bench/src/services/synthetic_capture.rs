use std::path::Path;

use minimap_core::config::TerrainConfig;
use minimap_core::structs::input::{KeyInput, MouseInput};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_runtime::errors::CaptureError;
use minimap_runtime::services::CaptureWriter;
use minimap_runtime::structs::{CaptureEvent, CaptureHeader, CaptureReport, FrameInput, SubmitStatus, TickInput};

use crate::services::BenchTerrain;

// writes a capture of a scripted walk over BenchTerrain, the way the shim would have recorded it:
// ticks, probes of the chunks around the player, frames, a world map pan and a dimension change
pub struct SyntheticCapture;

impl SyntheticCapture {
    pub const TICKS: i32 = 600;
    const RADIUS: i32 = 4;
    const M: i32 = 0x4D;

    pub fn write(path: &Path) -> Result<CaptureReport, CaptureError> {
        let writer = CaptureWriter::start(path, &CaptureHeader::new(&TerrainConfig::default()))?;
        let sink = writer.sink();
        let record = |event: CaptureEvent| {
            while !sink.record(event.clone()) {
                std::thread::yield_now();
            }
        };
        record(CaptureEvent::Enter { dim: 0 });
        record(CaptureEvent::BiomeName { id: 1, name: "plains".to_string() });
        record(CaptureEvent::BiomeName { id: 4, name: "forest".to_string() });
        for tick in 0..Self::TICKS {
            let dim = if tick < Self::TICKS * 5 / 6 { 0 } else { 1 };
            let player = Self::player(tick, dim);
            record(CaptureEvent::Tick(TickInput { player, foreground: true, cursor_hidden: true, dead: false }));
            for pos in Self::probes(tick, &player) {
                let probe = Box::new(BenchTerrain::probe_with_biomes(pos));
                record(CaptureEvent::Submit { pos, status: SubmitStatus::Loaded, probe });
            }
            if tick == Self::TICKS / 2 || tick == Self::TICKS / 2 + 60 {
                record(CaptureEvent::Key(KeyInput::new(Self::M, true, false, true)));
            }
            if tick > Self::TICKS / 2 && tick < Self::TICKS / 2 + 60 && tick % 10 == 0 {
                record(CaptureEvent::Mouse(MouseInput::new(MouseInput::WHEEL, -120)));
            }
            record(CaptureEvent::Frame(Self::frame(tick)));
        }
        record(CaptureEvent::Leave);
        Ok(writer.finish())
    }

    // east along z = 8 at one block per tick, then into the nether
    fn player(tick: i32, dim: i32) -> PlayerSnapshot {
        PlayerSnapshot { present: true, x: 8.5 + tick as f32, y: 80.0, z: 8.5, yaw: -90.0, dim }
    }

    // a full ring of probes on the first tick of each new chunk, as a scan plan would ask
    fn probes(tick: i32, player: &PlayerSnapshot) -> Vec<ChunkPos> {
        let centre = ChunkPos::from_block(player.x.floor() as i32, player.z.floor() as i32, player.dim);
        let first = tick == 0 || player.x.floor() as i32 % 16 == 0 || tick == Self::TICKS * 5 / 6;
        if !first {
            return Vec::new();
        }
        let mut out = Vec::new();
        for dz in -Self::RADIUS..=Self::RADIUS {
            for dx in -Self::RADIUS..=Self::RADIUS {
                out.push(ChunkPos::new(centre.x + dx, centre.z + dz, centre.dim));
            }
        }
        out
    }

    fn frame(tick: i32) -> FrameInput {
        let open = (Self::TICKS / 2..Self::TICKS / 2 + 60).contains(&tick);
        FrameInput {
            display: (1920.0, 1080.0),
            dt: 1.0 / 60.0,
            cursor: (960.0 + (tick % 60) as f32, 540.0),
            left_down: open && tick % 20 < 10,
            foreground: true,
            ..FrameInput::default()
        }
    }
}
