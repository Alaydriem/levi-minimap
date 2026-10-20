#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteBack {
    // the file is valid; nothing to write
    None,
    // the file is missing; write the defaults
    Create,
    // the file could not be parsed; keep a copy of it, then write the defaults
    ReplaceInvalid,
}
