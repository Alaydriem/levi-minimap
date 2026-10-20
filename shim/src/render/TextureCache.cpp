#include "render/TextureCache.h"

#include <unordered_map>

namespace levi_minimap {

namespace {

struct Texture {
    ID3D11Texture2D*          texture = nullptr;
    ID3D11ShaderResourceView* view    = nullptr;
    uint32_t                  width   = 0;
    uint32_t                  height  = 0;
};

std::unordered_map<uint32_t, Texture> g_textures;
ID3D11SamplerState*                   g_point  = nullptr;
ID3D11SamplerState*                   g_linear = nullptr;

void release(Texture& texture) {
    if (texture.view) texture.view->Release();
    if (texture.texture) texture.texture->Release();
    texture = Texture{};
}

void upload(ID3D11Device* device, ID3D11DeviceContext* context, const LmTextureOp& op) {
    if (!op.pixels || op.len < static_cast<uintptr_t>(op.width) * op.height * 4) return;
    auto& texture = g_textures[op.tex];
    if (texture.texture && texture.width == op.width && texture.height == op.height) {
        context->UpdateSubresource(texture.texture, 0, nullptr, op.pixels, op.width * 4, 0);
        return;
    }
    release(texture);
    D3D11_TEXTURE2D_DESC desc = {};
    desc.Width                = op.width;
    desc.Height               = op.height;
    desc.MipLevels            = 1;
    desc.ArraySize            = 1;
    desc.Format               = DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.SampleDesc.Count     = 1;
    desc.Usage                = D3D11_USAGE_DEFAULT;
    desc.BindFlags            = D3D11_BIND_SHADER_RESOURCE;
    D3D11_SUBRESOURCE_DATA data = {};
    data.pSysMem                = op.pixels;
    data.SysMemPitch            = op.width * 4;
    if (FAILED(device->CreateTexture2D(&desc, &data, &texture.texture))) {
        g_textures.erase(op.tex);
        return;
    }
    if (FAILED(device->CreateShaderResourceView(texture.texture, nullptr, &texture.view))) {
        release(texture);
        g_textures.erase(op.tex);
        return;
    }
    texture.width  = op.width;
    texture.height = op.height;
}

} // namespace

void TextureCache::apply(ID3D11Device* device, ID3D11DeviceContext* context, const LmTextureOp* ops, uint32_t count) {
    if (!device || !context) return;
    for (uint32_t i = 0; i < count; ++i) {
        const LmTextureOp& op = ops[i];
        if (op.kind == 1) {
            upload(device, context, op);
        } else if (op.kind == 2) {
            auto found = g_textures.find(op.tex);
            if (found != g_textures.end()) {
                release(found->second);
                g_textures.erase(found);
            }
        }
    }
}

ID3D11ShaderResourceView* TextureCache::view(uint32_t tex) {
    auto found = g_textures.find(tex);
    return found == g_textures.end() ? nullptr : found->second.view;
}

// point sampling keeps blocks sharp when magnified; linear wraps
ID3D11SamplerState* TextureCache::sampler(ID3D11Device* device, uint8_t kind) {
    if (!device) return nullptr;
    if (!g_point || !g_linear) {
        D3D11_SAMPLER_DESC desc = {};
        desc.AddressU           = D3D11_TEXTURE_ADDRESS_CLAMP;
        desc.AddressV           = D3D11_TEXTURE_ADDRESS_CLAMP;
        desc.AddressW           = D3D11_TEXTURE_ADDRESS_CLAMP;
        desc.Filter             = D3D11_FILTER_MIN_MAG_MIP_POINT;
        if (!g_point) device->CreateSamplerState(&desc, &g_point);
        desc.Filter   = D3D11_FILTER_MIN_MAG_MIP_LINEAR;
        desc.AddressU = D3D11_TEXTURE_ADDRESS_WRAP;
        desc.AddressV = D3D11_TEXTURE_ADDRESS_WRAP;
        desc.AddressW = D3D11_TEXTURE_ADDRESS_WRAP;
        if (!g_linear) device->CreateSamplerState(&desc, &g_linear);
    }
    return kind == 0 ? g_point : g_linear;
}

void TextureCache::releaseAll() {
    for (auto& [tex, texture] : g_textures) release(texture);
    g_textures.clear();
    if (g_point) {
        g_point->Release();
        g_point = nullptr;
    }
    if (g_linear) {
        g_linear->Release();
        g_linear = nullptr;
    }
}

} // namespace levi_minimap
