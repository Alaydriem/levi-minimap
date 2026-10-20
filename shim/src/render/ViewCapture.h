#pragma once

#include <d3d11.h>

namespace levi_minimap {

// copies the frame just drawn into a CPU-readable texture and hands its rows to Rust once
class ViewCapture {
public:
    static void capture(ID3D11Device* device, ID3D11DeviceContext* context, ID3D11RenderTargetView* target);
};

} // namespace levi_minimap
