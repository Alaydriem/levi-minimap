#pragma once

#include "levi_minimap.h"

#include <d3d11.h>

#include <cstdint>

namespace levi_minimap {

// draws the UI mesh Rust returns from lm_frame with D3D11; render thread only. It owns the UI's
// own textures (the font atlas); map textures come from TextureCache
class EguiRenderer {
public:
    // compiles the shaders and creates the fixed pipeline state once; false when the device refused
    static bool init(ID3D11Device* device);

    // creates, replaces or patches the UI textures the frame names
    static void applyTextures(ID3D11Device* device, ID3D11DeviceContext* context, const LmFrame& frame);

    // the frame's draw calls over the bound render target, `width` x `height` pixels
    static void draw(ID3D11Device* device, ID3D11DeviceContext* context, const LmFrame& frame, float width, float height);

    // UI textures the frame no longer uses, after it was drawn
    static void freeTextures(const LmFrame& frame);

    // every D3D11 object it holds; init runs again on the next frame
    static void release();
};

} // namespace levi_minimap
