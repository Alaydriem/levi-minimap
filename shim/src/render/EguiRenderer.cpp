#include "render/EguiRenderer.h"

#include "render/TextureCache.h"

#include <d3dcompiler.h>

#include <algorithm>
#include <cstring>
#include <unordered_map>

namespace levi_minimap {

namespace {

constexpr uint8_t kMapTexture = 1;

// positions arrive in physical pixels and become clip space from the screen size; the map pixel
// shader premultiplies the straight-alpha map textures, the UI one samples premultiplied textures
constexpr const char* kShader = R"(
cbuffer Screen : register(b0) { float2 screen; float2 unused; };
struct VsIn { float2 pos : POSITION; float2 uv : TEXCOORD0; float4 colour : COLOR0; };
struct VsOut { float4 pos : SV_POSITION; float2 uv : TEXCOORD0; float4 colour : COLOR0; };
Texture2D tex : register(t0);
SamplerState samp : register(s0);
VsOut vs_main(VsIn i) {
    VsOut o;
    o.pos = float4(i.pos.x / screen.x * 2.0 - 1.0, 1.0 - i.pos.y / screen.y * 2.0, 0.0, 1.0);
    o.uv = i.uv;
    o.colour = i.colour;
    return o;
}
float4 ps_ui(VsOut i) : SV_Target { return i.colour * tex.Sample(samp, i.uv); }
float4 ps_map(VsOut i) : SV_Target {
    float4 s = tex.Sample(samp, i.uv);
    return i.colour * float4(s.rgb * s.a, s.a);
}
)";

struct UiTexture {
    ID3D11Texture2D*          texture = nullptr;
    ID3D11ShaderResourceView* view    = nullptr;
    uint32_t                  width   = 0;
    uint32_t                  height  = 0;
};

struct Pipeline {
    ID3D11VertexShader*      vs        = nullptr;
    ID3D11PixelShader*       psUi      = nullptr;
    ID3D11PixelShader*       psMap     = nullptr;
    ID3D11InputLayout*       layout    = nullptr;
    ID3D11Buffer*            screen    = nullptr;
    ID3D11BlendState*        blend     = nullptr;
    ID3D11RasterizerState*   raster    = nullptr;
    ID3D11DepthStencilState* depth     = nullptr;
    ID3D11SamplerState*      point     = nullptr;
    ID3D11SamplerState*      linear    = nullptr;
    ID3D11Buffer*            vertices  = nullptr;
    ID3D11Buffer*            indices   = nullptr;
    uint32_t                 vertexCap = 0;
    uint32_t                 indexCap  = 0;
};

Pipeline                                g_pipe;
bool                                    g_ready = false;
std::unordered_map<uint32_t, UiTexture> g_textures;

template <typename T>
void releaseCom(T*& object) {
    if (object) {
        object->Release();
        object = nullptr;
    }
}

void releaseTexture(UiTexture& texture) {
    releaseCom(texture.view);
    releaseCom(texture.texture);
    texture = UiTexture{};
}

ID3DBlob* compile(const char* entry, const char* target) {
    ID3DBlob* blob   = nullptr;
    ID3DBlob* errors = nullptr;
    HRESULT   hr     = D3DCompile(kShader, std::strlen(kShader), nullptr, nullptr, nullptr, entry, target, 0, 0, &blob, &errors);
    releaseCom(errors);
    return SUCCEEDED(hr) ? blob : nullptr;
}

bool createShaders(ID3D11Device* device) {
    ID3DBlob* vs    = compile("vs_main", "vs_4_0");
    ID3DBlob* psUi  = compile("ps_ui", "ps_4_0");
    ID3DBlob* psMap = compile("ps_map", "ps_4_0");
    bool      ok    = vs && psUi && psMap;
    if (ok) {
        D3D11_INPUT_ELEMENT_DESC layout[] = {
            {"POSITION", 0, DXGI_FORMAT_R32G32_FLOAT, 0, 0, D3D11_INPUT_PER_VERTEX_DATA, 0},
            {"TEXCOORD", 0, DXGI_FORMAT_R32G32_FLOAT, 0, 8, D3D11_INPUT_PER_VERTEX_DATA, 0},
            {"COLOR", 0, DXGI_FORMAT_R8G8B8A8_UNORM, 0, 16, D3D11_INPUT_PER_VERTEX_DATA, 0},
        };
        ok = SUCCEEDED(device->CreateVertexShader(vs->GetBufferPointer(), vs->GetBufferSize(), nullptr, &g_pipe.vs))
          && SUCCEEDED(device->CreatePixelShader(psUi->GetBufferPointer(), psUi->GetBufferSize(), nullptr, &g_pipe.psUi))
          && SUCCEEDED(device->CreatePixelShader(psMap->GetBufferPointer(), psMap->GetBufferSize(), nullptr, &g_pipe.psMap))
          && SUCCEEDED(device->CreateInputLayout(layout, 3, vs->GetBufferPointer(), vs->GetBufferSize(), &g_pipe.layout));
    }
    releaseCom(vs);
    releaseCom(psUi);
    releaseCom(psMap);
    return ok;
}

bool createStates(ID3D11Device* device) {
    D3D11_BUFFER_DESC screen = {};
    screen.ByteWidth         = 16;
    screen.Usage             = D3D11_USAGE_DYNAMIC;
    screen.BindFlags         = D3D11_BIND_CONSTANT_BUFFER;
    screen.CPUAccessFlags    = D3D11_CPU_ACCESS_WRITE;

    D3D11_BLEND_DESC blend                      = {};
    blend.RenderTarget[0].BlendEnable           = TRUE;
    blend.RenderTarget[0].SrcBlend              = D3D11_BLEND_ONE;
    blend.RenderTarget[0].DestBlend             = D3D11_BLEND_INV_SRC_ALPHA;
    blend.RenderTarget[0].BlendOp               = D3D11_BLEND_OP_ADD;
    blend.RenderTarget[0].SrcBlendAlpha         = D3D11_BLEND_ONE;
    blend.RenderTarget[0].DestBlendAlpha        = D3D11_BLEND_INV_SRC_ALPHA;
    blend.RenderTarget[0].BlendOpAlpha          = D3D11_BLEND_OP_ADD;
    blend.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL;

    D3D11_RASTERIZER_DESC raster = {};
    raster.FillMode              = D3D11_FILL_SOLID;
    raster.CullMode              = D3D11_CULL_NONE;
    raster.ScissorEnable         = TRUE;
    raster.DepthClipEnable       = TRUE;

    D3D11_DEPTH_STENCIL_DESC depth = {};
    depth.DepthEnable              = FALSE;
    depth.StencilEnable            = FALSE;

    D3D11_SAMPLER_DESC sampler = {};
    sampler.AddressU           = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler.AddressV           = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler.AddressW           = D3D11_TEXTURE_ADDRESS_CLAMP;
    sampler.Filter             = D3D11_FILTER_MIN_MAG_MIP_POINT;
    bool ok = SUCCEEDED(device->CreateBuffer(&screen, nullptr, &g_pipe.screen))
           && SUCCEEDED(device->CreateBlendState(&blend, &g_pipe.blend))
           && SUCCEEDED(device->CreateRasterizerState(&raster, &g_pipe.raster))
           && SUCCEEDED(device->CreateDepthStencilState(&depth, &g_pipe.depth))
           && SUCCEEDED(device->CreateSamplerState(&sampler, &g_pipe.point));
    sampler.Filter = D3D11_FILTER_MIN_MAG_MIP_LINEAR;
    return ok && SUCCEEDED(device->CreateSamplerState(&sampler, &g_pipe.linear));
}

// grows a dynamic buffer to hold `count` elements of `stride` bytes, doubling so it rarely grows
bool reserve(ID3D11Device* device, ID3D11Buffer*& buffer, uint32_t& capacity, uint32_t count, uint32_t stride, UINT bind) {
    if (buffer && capacity >= count) return true;
    releaseCom(buffer);
    uint32_t          size = std::max(count, std::max(capacity * 2, 4096u));
    D3D11_BUFFER_DESC desc = {};
    desc.ByteWidth         = size * stride;
    desc.Usage             = D3D11_USAGE_DYNAMIC;
    desc.BindFlags         = bind;
    desc.CPUAccessFlags    = D3D11_CPU_ACCESS_WRITE;
    if (FAILED(device->CreateBuffer(&desc, nullptr, &buffer))) {
        capacity = 0;
        return false;
    }
    capacity = size;
    return true;
}

bool upload(ID3D11DeviceContext* context, ID3D11Buffer* buffer, const void* data, size_t bytes) {
    D3D11_MAPPED_SUBRESOURCE mapped = {};
    if (FAILED(context->Map(buffer, 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped))) return false;
    std::memcpy(mapped.pData, data, bytes);
    context->Unmap(buffer, 0);
    return true;
}

void createTexture(ID3D11Device* device, const LmUiTextureDelta& delta) {
    UiTexture& texture = g_textures[delta.texture];
    releaseTexture(texture);
    D3D11_TEXTURE2D_DESC desc = {};
    desc.Width                = delta.width;
    desc.Height               = delta.height;
    desc.MipLevels            = 1;
    desc.ArraySize            = 1;
    desc.Format               = DXGI_FORMAT_R8G8B8A8_UNORM;
    desc.SampleDesc.Count     = 1;
    desc.Usage                = D3D11_USAGE_DEFAULT;
    desc.BindFlags            = D3D11_BIND_SHADER_RESOURCE;
    D3D11_SUBRESOURCE_DATA data = {};
    data.pSysMem                = delta.pixels;
    data.SysMemPitch            = delta.width * 4;
    if (FAILED(device->CreateTexture2D(&desc, &data, &texture.texture))
        || FAILED(device->CreateShaderResourceView(texture.texture, nullptr, &texture.view))) {
        releaseTexture(texture);
        g_textures.erase(delta.texture);
        return;
    }
    texture.width  = delta.width;
    texture.height = delta.height;
}

void patchTexture(ID3D11DeviceContext* context, const LmUiTextureDelta& delta) {
    auto found = g_textures.find(delta.texture);
    if (found == g_textures.end() || !found->second.texture) return;
    const UiTexture& texture = found->second;
    if (delta.x + delta.width > texture.width || delta.y + delta.height > texture.height) return;
    D3D11_BOX box = {delta.x, delta.y, 0, delta.x + delta.width, delta.y + delta.height, 1};
    context->UpdateSubresource(texture.texture, 0, &box, delta.pixels, delta.width * 4, 0);
}

void bindPipeline(ID3D11DeviceContext* context, float width, float height) {
    D3D11_VIEWPORT viewport = {0.0f, 0.0f, width, height, 0.0f, 1.0f};
    context->RSSetViewports(1, &viewport);
    UINT stride = 20;
    UINT offset = 0;
    context->IASetInputLayout(g_pipe.layout);
    context->IASetVertexBuffers(0, 1, &g_pipe.vertices, &stride, &offset);
    context->IASetIndexBuffer(g_pipe.indices, DXGI_FORMAT_R32_UINT, 0);
    context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
    context->VSSetShader(g_pipe.vs, nullptr, 0);
    context->VSSetConstantBuffers(0, 1, &g_pipe.screen);
    const float factor[4] = {0.0f, 0.0f, 0.0f, 0.0f};
    context->OMSetBlendState(g_pipe.blend, factor, 0xFFFFFFFF);
    context->OMSetDepthStencilState(g_pipe.depth, 0);
    context->RSSetState(g_pipe.raster);
    context->GSSetShader(nullptr, nullptr, 0);
    context->HSSetShader(nullptr, nullptr, 0);
    context->DSSetShader(nullptr, nullptr, 0);
    context->CSSetShader(nullptr, nullptr, 0);
}

} // namespace

bool EguiRenderer::init(ID3D11Device* device) {
    if (g_ready) return true;
    if (!device) return false;
    if (!createShaders(device) || !createStates(device)) {
        release();
        return false;
    }
    g_ready = true;
    return true;
}

void EguiRenderer::applyTextures(ID3D11Device* device, ID3D11DeviceContext* context, const LmFrame& frame) {
    if (!device || !context) return;
    for (uint32_t i = 0; i < frame.ui_texture_count; ++i) {
        const LmUiTextureDelta& delta = frame.ui_textures[i];
        if (!delta.pixels || delta.len < static_cast<uintptr_t>(delta.width) * delta.height * 4) continue;
        if (delta.full) {
            createTexture(device, delta);
        } else {
            patchTexture(context, delta);
        }
    }
}

void EguiRenderer::draw(ID3D11Device* device, ID3D11DeviceContext* context, const LmFrame& frame, float width, float height) {
    if (!g_ready || frame.draw_count == 0 || frame.vertex_count == 0 || width <= 0.0f || height <= 0.0f) return;
    if (!reserve(device, g_pipe.vertices, g_pipe.vertexCap, frame.vertex_count, sizeof(LmUiVertex), D3D11_BIND_VERTEX_BUFFER)) return;
    if (!reserve(device, g_pipe.indices, g_pipe.indexCap, frame.index_count, sizeof(uint32_t), D3D11_BIND_INDEX_BUFFER)) return;
    if (!upload(context, g_pipe.vertices, frame.vertices, frame.vertex_count * sizeof(LmUiVertex))) return;
    if (!upload(context, g_pipe.indices, frame.indices, frame.index_count * sizeof(uint32_t))) return;
    const float screen[4] = {width, height, 0.0f, 0.0f};
    if (!upload(context, g_pipe.screen, screen, sizeof(screen))) return;
    bindPipeline(context, width, height);
    ID3D11PixelShader* bound = nullptr;
    for (uint32_t i = 0; i < frame.draw_count; ++i) {
        const LmUiDraw& call = frame.draws[i];
        if (call.index_offset + call.index_count > frame.index_count) continue;
        ID3D11ShaderResourceView* view = nullptr;
        if (call.kind == kMapTexture) {
            view = TextureCache::view(call.texture);
        } else {
            auto found = g_textures.find(call.texture);
            view       = found == g_textures.end() ? nullptr : found->second.view;
        }
        if (!view) continue;
        ID3D11PixelShader* shader = call.kind == kMapTexture ? g_pipe.psMap : g_pipe.psUi;
        if (shader != bound) {
            context->PSSetShader(shader, nullptr, 0);
            bound = shader;
        }
        D3D11_RECT scissor = {
            static_cast<LONG>(call.clip[0]),
            static_cast<LONG>(call.clip[1]),
            static_cast<LONG>(call.clip[2] + 0.5f),
            static_cast<LONG>(call.clip[3] + 0.5f)
        };
        context->RSSetScissorRects(1, &scissor);
        ID3D11SamplerState* sampler = call.sampler == 0 ? g_pipe.point : g_pipe.linear;
        context->PSSetSamplers(0, 1, &sampler);
        context->PSSetShaderResources(0, 1, &view);
        context->DrawIndexed(call.index_count, call.index_offset, static_cast<INT>(call.vertex_offset));
    }
}

void EguiRenderer::freeTextures(const LmFrame& frame) {
    for (uint32_t i = 0; i < frame.ui_freed_count; ++i) {
        auto found = g_textures.find(frame.ui_freed[i]);
        if (found == g_textures.end()) continue;
        releaseTexture(found->second);
        g_textures.erase(found);
    }
}

void EguiRenderer::release() {
    for (auto& [id, texture] : g_textures) releaseTexture(texture);
    g_textures.clear();
    releaseCom(g_pipe.vs);
    releaseCom(g_pipe.psUi);
    releaseCom(g_pipe.psMap);
    releaseCom(g_pipe.layout);
    releaseCom(g_pipe.screen);
    releaseCom(g_pipe.blend);
    releaseCom(g_pipe.raster);
    releaseCom(g_pipe.depth);
    releaseCom(g_pipe.point);
    releaseCom(g_pipe.linear);
    releaseCom(g_pipe.vertices);
    releaseCom(g_pipe.indices);
    g_pipe  = Pipeline{};
    g_ready = false;
}

} // namespace levi_minimap
