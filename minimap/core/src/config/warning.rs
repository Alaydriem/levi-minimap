#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigWarning {
    pub key: &'static str,
    pub message: String,
}

impl ConfigWarning {
    pub fn new(key: &'static str, message: String) -> Self {
        Self { key, message }
    }
}
