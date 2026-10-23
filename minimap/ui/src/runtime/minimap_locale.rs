use std::sync::Arc;

use levi_i18n::localize::Localizer;
use levi_i18n::message::Message;
use levi_i18n::service::{LocaleSetting, LocalizationService};
use levi_ui::LeviUiMessage;
use levi_ui::text::LeviUiPacks;

use crate::message::MinimapMessage;
use crate::text::MinimapPacks;

/// The minimap's words in one language: its own packs and the levi-ui widgets' packs.
pub struct MinimapLocale;

impl MinimapLocale {
    /// `code` is the manager's language code: empty for English, `en_XA` for the pseudo-locale.
    pub fn localizer(code: &str) -> Arc<Localizer> {
        let service = LocalizationService::new(vec![
            (LeviUiMessage::DOMAIN, LeviUiPacks::SOURCE),
            (MinimapMessage::DOMAIN, MinimapPacks::SOURCE),
        ]);
        Arc::new(service.load(&LocaleSetting::from_code(code)))
    }
}
