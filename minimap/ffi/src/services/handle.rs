use std::sync::{Arc, Mutex};

use amm_client::link::ManagerLink;
use minimap_link::services::{MinimapSession, SystemClipboard};
use minimap_runtime::services::Runtime;

use crate::services::GameBuffers;

// the opaque handle C++ holds: the manager session the tick runs, and the runtime the game calls reach
pub struct LmHandle {
    pub session: Mutex<Option<MinimapSession<ManagerLink, SystemClipboard>>>,
    pub runtime: Arc<Runtime>,
    pub game: Mutex<GameBuffers>,
}

impl LmHandle {
    pub fn new(session: MinimapSession<ManagerLink, SystemClipboard>) -> Self {
        let runtime = Arc::clone(session.runtime());
        Self { session: Mutex::new(Some(session)), runtime, game: Mutex::new(GameBuffers::new()) }
    }
}
