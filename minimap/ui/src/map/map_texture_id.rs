use egui::TextureId;
use minimap_core::structs::draw::SamplerKind;

// a map texture in egui: TextureId::User with the map texture id in the low 32 bits and the
// sampler in bit 32
pub struct MapTextureId;

impl MapTextureId {
    const LINEAR_BIT: u64 = 1 << 32;

    pub fn encode(tex: u32, sampler: SamplerKind) -> TextureId {
        let bit = if sampler == SamplerKind::Linear { Self::LINEAR_BIT } else { 0 };
        TextureId::User(u64::from(tex) | bit)
    }

    // None for egui's own textures
    pub fn decode(id: TextureId) -> Option<(u32, SamplerKind)> {
        match id {
            TextureId::User(raw) => {
                let sampler = if raw & Self::LINEAR_BIT != 0 { SamplerKind::Linear } else { SamplerKind::Point };
                Some((raw as u32, sampler))
            }
            TextureId::Managed(_) => None,
        }
    }
}
