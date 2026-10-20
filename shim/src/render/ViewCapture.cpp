#include "render/ViewCapture.h"

#include "bridge/RustBridge.h"

namespace levi_minimap {

void ViewCapture::capture(ID3D11Device* device, ID3D11DeviceContext* context, ID3D11RenderTargetView* target) {
    const LmHandle* handle = RustBridge::handle();
    if (!handle) return;
    ID3D11Resource* resource = nullptr;
    target->GetResource(&resource);
    if (!resource) return;
    ID3D11Texture2D* back = nullptr;
    HRESULT          isTexture = resource->QueryInterface(__uuidof(ID3D11Texture2D), reinterpret_cast<void**>(&back));
    resource->Release();
    if (FAILED(isTexture)) return;
    D3D11_TEXTURE2D_DESC desc{};
    back->GetDesc(&desc);
    desc.Usage          = D3D11_USAGE_STAGING;
    desc.BindFlags      = 0;
    desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
    desc.MiscFlags      = 0;
    desc.MipLevels      = 1;
    desc.ArraySize      = 1;
    desc.SampleDesc     = {1, 0};
    ID3D11Texture2D* staging = nullptr;
    if (SUCCEEDED(device->CreateTexture2D(&desc, nullptr, &staging))) {
        context->CopyResource(staging, back);
        D3D11_MAPPED_SUBRESOURCE mapped{};
        if (SUCCEEDED(context->Map(staging, 0, D3D11_MAP_READ, 0, &mapped))) {
            lm_view_captured(
                handle,
                static_cast<const uint8_t*>(mapped.pData),
                desc.Width,
                desc.Height,
                mapped.RowPitch,
                static_cast<uint32_t>(desc.Format)
            );
            context->Unmap(staging, 0);
        }
        staging->Release();
    }
    back->Release();
}

} // namespace levi_minimap
