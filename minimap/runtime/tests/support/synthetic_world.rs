use std::path::{Path, PathBuf};
use std::time::Duration;

use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::{FrameInput, FrameOutput, RuntimePaths, ScanPlan, SubmitStatus, TickInput};
use minimap_store::structs::WorldIdentity;

// a deterministic hilly world that answers probes the way the shim does
pub struct SyntheticWorld {
    pub dir: tempfile::TempDir,
    pub plan: ScanPlan,
    pub frame: FrameOutput,
    pub player: PlayerSnapshot,
    // answer the plan's biome name requests the way the shim does
    pub answer_biomes: bool,
}

impl SyntheticWorld {
    pub fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
            plan: ScanPlan::default(),
            frame: FrameOutput::new(),
            player: PlayerSnapshot { present: true, x: 8.5, y: 80.0, z: 8.5, yaw: 0.0, dim: 0 },
            answer_biomes: true,
        }
    }

    pub fn paths(&self) -> RuntimePaths {
        RuntimePaths {
            config_file: self.dir.path().join("config").join("config.json"),
            data_dir: self.dir.path().join("data"),
        }
    }

    pub fn world_dir(&self) -> PathBuf {
        self.dir.path().join("world")
    }

    pub fn identity(&self) -> WorldIdentity {
        WorldIdentity::Local { world_dir: self.world_dir() }
    }

    pub fn write_config(&self, json: &str) {
        let path = self.paths().config_file;
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, json).unwrap();
    }

    pub fn runtime(&self) -> Runtime {
        let runtime = Runtime::new(self.paths()).unwrap();
        runtime.enter_world(&self.identity(), self.player.dim).unwrap();
        runtime
    }

    pub fn height(x: i32, z: i32) -> i16 {
        64 + ((x.rem_euclid(23) as i16 - 11).abs() + (z.rem_euclid(17) as i16 - 8).abs())
    }

    // game biome 1 (plains) where x mod 64 is below 32, else 4 (forest)
    pub fn biome_id(x: i32) -> u16 {
        if x.rem_euclid(64) < 32 { 1 } else { 4 }
    }

    pub fn probe(pos: ChunkPos) -> ChunkProbe {
        let mut surface = ChunkSurface::empty();
        let mut biomes = [0u16; 256];
        for lz in 0..16 {
            for lx in 0..16 {
                let (x, z) = (pos.min_block_x() + lx as i32, pos.min_block_z() + lz as i32);
                let h = Self::height(x, z);
                let shade = (h - 40).clamp(0, 255) as u8 * 4;
                let colour = Rgba::new(shade, 160, 60, 255).packed();
                let index = ChunkSurface::index(lx, lz);
                surface.write_column(index, ColumnSample::new(colour, 0, h, h, false, Self::biome_id(x)));
                biomes[index] = Self::biome_id(x);
            }
        }
        surface.skipped = [0; 4];
        ChunkProbe::new(surface, biomes)
    }

    pub fn biome_name(id: u16) -> &'static str {
        match id {
            1 => "plains",
            4 => "forest",
            _ => "",
        }
    }

    // one game tick: probe up to the plan's limit, report the rest as not probed
    pub fn tick(&mut self, runtime: &Runtime) {
        let input = TickInput { player: self.player, foreground: true, cursor_hidden: true, dead: false };
        runtime.tick(&input, &mut self.plan);
        if self.answer_biomes {
            for &id in &self.plan.biome_requests {
                runtime.biome_name(id, Self::biome_name(id));
            }
        }
        for (i, pos) in self.plan.candidates.clone().into_iter().enumerate() {
            if i < self.plan.limit {
                runtime.submit_chunk(pos, SubmitStatus::Loaded, &Self::probe(pos));
            } else {
                runtime.submit_chunk(pos, SubmitStatus::NotProbed, &ChunkProbe::new(ChunkSurface::empty(), [0; 256]));
            }
        }
    }

    pub fn draw(&mut self, runtime: &Runtime) {
        let input = FrameInput {
            display: (1280.0, 720.0),
            dt: 0.016,
            cursor: (640.0, 360.0),
            left_down: false,
            foreground: true,
            collect_text: true,
            ..FrameInput::default()
        };
        runtime.frame(&input, &mut self.frame);
    }

    // ticks and draws until `done` holds or the step limit is reached; true when it held
    pub fn run_until(
        &mut self,
        runtime: &Runtime,
        steps: usize,
        done: impl Fn(&Runtime, &FrameOutput) -> bool,
    ) -> bool {
        for _ in 0..steps {
            self.tick(runtime);
            self.draw(runtime);
            if done(runtime, &self.frame) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        false
    }

    pub fn exists(path: &Path) -> bool {
        path.exists()
    }
}
