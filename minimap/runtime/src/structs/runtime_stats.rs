// counters for tests and telemetry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RuntimeStats {
    pub in_world: bool,
    pub ticks: u64,
    pub frames: u64,
    pub chunks_submitted: u64,
    pub bakes_completed: u64,
    pub tiles_loaded: u64,
    pub surfaces_resident: usize,
    pub surface_capacity: usize,
    pub tiles_resident: usize,
    pub tile_capacity: usize,
    pub tile_overrun: usize,
    pub gpu_textures: usize,
    pub bake_queue: usize,
    pub outbox: usize,
    pub deferred: usize,
    pub disk_written_bytes: u64,
}
