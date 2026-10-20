use std::path::PathBuf;

use minimap_core::services::export::AreaLayout;
use minimap_core::structs::pos::RegionPos;

// one export for the export thread; `epoch` is the world visit it belongs to
#[derive(Debug)]
pub enum ExportJob {
    // tight RGBA rows of the captured screen
    View { path: PathBuf, rgba: Vec<u8>, width: u32, height: u32, epoch: u64 },
    // `regions` are drawn, from the tile store or the region files under `cache`; the rest is clear
    Area { path: PathBuf, layout: AreaLayout, regions: Vec<RegionPos>, cache: Option<PathBuf>, dim: i32, epoch: u64 },
}
