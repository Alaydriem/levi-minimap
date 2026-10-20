#pragma once

#include "levi_minimap.h"

#include <d3d11.h>

#include <cstdint>

namespace levi_minimap {

// the GPU textures Rust asked for, by Rust's texture id; Rust sends a release for every id it retires
class TextureCache {
public:
    static void apply(ID3D11Device* device, ID3D11DeviceContext* context, const LmTextureOp* ops, uint32_t count);

    // null when the id has no texture
    static ID3D11ShaderResourceView* view(uint32_t tex);

    static ID3D11SamplerState* sampler(ID3D11Device* device, uint8_t kind);

    static void releaseAll();
};

} // namespace levi_minimap
