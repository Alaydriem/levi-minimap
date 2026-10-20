#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LmStats {
    pub in_world: u8,
    pub ticks: u64,
    pub frames: u64,
    pub chunks_submitted: u64,
    pub bakes_completed: u64,
    pub tiles_loaded: u64,
    pub surfaces_resident: u64,
    pub surface_capacity: u64,
    pub tiles_resident: u64,
    pub tile_capacity: u64,
    pub tile_overrun: u64,
    pub gpu_textures: u64,
    pub bake_queue: u64,
    pub outbox: u64,
    pub deferred: u64,
}
