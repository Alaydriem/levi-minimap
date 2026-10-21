// the shim's log lines, through Curia to the manager's log for this mod
pub struct LmLog;

impl LmLog {
    const ERROR: u8 = 1;
    const WARN: u8 = 2;
    const INFO: u8 = 3;

    pub fn write(level: u8, message: &str) {
        match level {
            Self::ERROR => curia::error!("shim", { message: message }),
            Self::WARN => curia::warn!("shim", { message: message }),
            Self::INFO => curia::info!("shim", { message: message }),
            _ => curia::debug!("shim", { message: message }),
        }
    }
}
