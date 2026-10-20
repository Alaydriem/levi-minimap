#pragma once

#include <d3d11.h>
#include <cstdint>
#include <windows.h>

namespace levi_minimap {

// one overlay frame: asks Rust for it, applies texture work, draws the UI mesh, sets the cursor
class FrameRenderer {
public:
    static void render(ID3D11Device* device, ID3D11DeviceContext* context, HWND hwnd);

    // the world map is open, so the system cursor must be shown
    static bool mapOpen();

    // a dialog has the keyboard, so typed characters go to Rust instead of the game
    static bool textInput();

    // the frame just drawn must be copied for a view export; read on the render thread after drawing
    static bool captureView();

    // telemetry for the next frame: the whole overlay's time
    static void reportFrame(uint32_t renderUs);
};

} // namespace levi_minimap
