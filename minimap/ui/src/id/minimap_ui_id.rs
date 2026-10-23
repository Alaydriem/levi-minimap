use egui::Id;

/// The egui ids of the minimap's world map views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinimapUiId {
    /// The map layer under the cards.
    Map,
    /// The cards and their layers.
    Tools,
    List,
    Hint,
    Menu,
    Dialog,
    Dim,
    Toast,
    /// The text fields and the scroll area (egui keeps their cursor, focus and offset).
    Search,
    Rows,
    DialogName,
    /// One colour swatch in the dialog, with its index.
    Swatch,
}

impl MinimapUiId {
    /// The id, salted with the minimap's name so it never equals another crate's id from an enum of its own.
    pub fn id(self) -> Id {
        Id::unique(("minimap", self))
    }
}
