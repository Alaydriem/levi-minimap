use minimap_core::structs::pos::ChunkPos;

// what the shim should probe after a block change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeRequest {
    None,
    Column { pos: ChunkPos, index: usize },
}
