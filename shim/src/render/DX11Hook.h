#pragma once

#include <windows.h>

// these names clash with LeviLamina's own definitions
#define D3D12_FEATURE_DATA_D3D12_OPTIONS D3D12_FEATURE_DATA_D3D12_OPTIONS_LEGACY
#define D3D12_FEATURE_DATA_ARCHITECTURE  D3D12_FEATURE_DATA_ARCHITECTURE_LEGACY
#define D3D12_RAYTRACING_GEOMETRY_DESC   D3D12_RAYTRACING_GEOMETRY_DESC_LEGACY

#include <d3d11.h>
#include <d3d11on12.h>
#include <d3d12.h>
#include <dxgi1_4.h>

#undef D3D12_FEATURE_DATA_D3D12_OPTIONS
#undef D3D12_FEATURE_DATA_ARCHITECTURE
#undef D3D12_RAYTRACING_GEOMETRY_DESC

namespace levi_minimap {

// hooks the swap chain's Present to draw the overlay; works in D3D11 and D3D12 (through D3D11On12)
class DX11Hook {
public:
    static bool init();
    static void shutdown();

    static ID3D11Device*        getDevice();
    static ID3D11DeviceContext* getContext();
    static HWND                 getHwnd();
};

} // namespace levi_minimap
