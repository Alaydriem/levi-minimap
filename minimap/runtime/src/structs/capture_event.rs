use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::input::{KeyInput, MouseInput};
use minimap_core::structs::pos::ChunkPos;

use crate::structs::{FrameInput, SubmitStatus, TickInput};

// one call from the shim into the runtime, in the order the runtime received it
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureEvent {
    Enter { dim: i32 },
    Leave,
    Tick(TickInput),
    Submit { pos: ChunkPos, status: SubmitStatus, probe: Box<ChunkProbe> },
    // the game's answer for a biome ID's internal name; empty when it has none
    BiomeName { id: u16, name: String },
    Column { pos: ChunkPos, index: u32, sample: ColumnSample },
    Block { x: i32, z: i32, dim: i32 },
    Key(KeyInput),
    Mouse(MouseInput),
    Frame(FrameInput),
}
