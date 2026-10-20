// what an export writes: the screen as shown, or every explored region at a fixed scale
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    View,
    Area,
}

impl ExportKind {
    // the last part of the file name
    pub fn suffix(&self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Area => "area",
        }
    }
}
