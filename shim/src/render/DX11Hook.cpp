#include "render/DX11Hook.h"

#include "bridge/RustBridge.h"
#include "render/EguiRenderer.h"
#include "render/FrameRenderer.h"
#include "render/TextureCache.h"
#include "render/ViewCapture.h"

#include "ll/api/memory/Hook.h"

#include <atomic>
#include <chrono>

namespace levi_minimap {

namespace {

using Present_t             = HRESULT(__stdcall*)(IDXGISwapChain*, UINT, UINT);
using Present1_t            = HRESULT(__stdcall*)(IDXGISwapChain1*, UINT, UINT, const DXGI_PRESENT_PARAMETERS*);
using ResizeBuffers_t       = HRESULT(__stdcall*)(IDXGISwapChain*, UINT, UINT, UINT, DXGI_FORMAT, UINT);
using ExecuteCommandLists_t = void(__stdcall*)(ID3D12CommandQueue*, UINT, ID3D12CommandList* const*);

Present_t             oPresent             = nullptr;
Present1_t            oPresent1            = nullptr;
ResizeBuffers_t       oResizeBuffers       = nullptr;
ExecuteCommandLists_t oExecuteCommandLists = nullptr;

ll::memory::FuncPtr g_targetPresent             = nullptr;
ll::memory::FuncPtr g_detourPresent             = nullptr;
ll::memory::FuncPtr g_targetPresent1            = nullptr;
ll::memory::FuncPtr g_detourPresent1            = nullptr;
ll::memory::FuncPtr g_targetResizeBuffers       = nullptr;
ll::memory::FuncPtr g_detourResizeBuffers       = nullptr;
ll::memory::FuncPtr g_targetExecuteCommandLists = nullptr;
ll::memory::FuncPtr g_detourExecuteCommandLists = nullptr;

ID3D11Device*        g_device          = nullptr;
ID3D11DeviceContext* g_context         = nullptr;
ID3D11On12Device*    g_d3d11On12Device = nullptr;
ID3D12CommandQueue*  g_commandQueue    = nullptr;

HWND              g_hwnd = nullptr;
std::atomic<bool> g_overlayReady{false};
std::atomic<bool> g_shuttingDown{false};
std::atomic<int>  g_renderInFlight{0};
WNDPROC           g_origWndProc = nullptr;

// the game hides the cursor in WM_SETCURSOR; while the world map is open, show the arrow.
// While a dialog has the keyboard, typed characters go to Rust and not to the game
LRESULT CALLBACK wndProcHook(HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam) {
    bool live = !g_shuttingDown.load(std::memory_order_acquire);
    if (live && msg == WM_SETCURSOR && FrameRenderer::mapOpen()) {
        SetCursor(LoadCursorW(nullptr, IDC_ARROW));
        return TRUE;
    }
    if (live && msg == WM_CHAR && FrameRenderer::textInput()) {
        if (const LmHandle* handle = RustBridge::handle()) lm_text(handle, static_cast<uint32_t>(wParam));
        return 0;
    }
    if (g_origWndProc) return CallWindowProcW(g_origWndProc, hwnd, msg, wParam, lParam);
    return DefWindowProcW(hwnd, msg, wParam, lParam);
}

void installWndProcHook() {
    if (g_hwnd && !g_origWndProc) {
        g_origWndProc = reinterpret_cast<WNDPROC>(SetWindowLongPtrW(g_hwnd, GWLP_WNDPROC, reinterpret_cast<LONG_PTR>(&wndProcHook)));
    }
}

// restore only while our hook is still on top; another mod may have subclassed after us
void removeWndProcHook() {
    if (g_hwnd && g_origWndProc
        && GetWindowLongPtrW(g_hwnd, GWLP_WNDPROC) == reinterpret_cast<LONG_PTR>(&wndProcHook)) {
        SetWindowLongPtrW(g_hwnd, GWLP_WNDPROC, reinterpret_cast<LONG_PTR>(g_origWndProc));
        g_origWndProc = nullptr;
    }
}

void shutdownOverlay() {
    if (g_overlayReady.exchange(false)) EguiRenderer::release();
}

void __stdcall hkExecuteCommandLists(ID3D12CommandQueue* queue, UINT count, ID3D12CommandList* const* lists) {
    if (!g_commandQueue) {
        D3D12_COMMAND_QUEUE_DESC desc = queue->GetDesc();
        if (desc.Type == D3D12_COMMAND_LIST_TYPE_DIRECT) {
            g_commandQueue = queue;
            g_commandQueue->AddRef();
        }
    }
    oExecuteCommandLists(queue, count, lists);
}

// the overlay holds no back buffer views between frames, so a resize needs nothing released; the
// UI textures stay, because Rust sends the font atlas only once
HRESULT __stdcall hkResizeBuffers(IDXGISwapChain* swapChain, UINT count, UINT width, UINT height, DXGI_FORMAT format, UINT flags) {
    return oResizeBuffers(swapChain, count, width, height, format, flags);
}

bool createDevices(IDXGISwapChain* swapChain) {
    if (g_device) return true;
    if (SUCCEEDED(swapChain->GetDevice(__uuidof(ID3D11Device), reinterpret_cast<void**>(&g_device)))) {
        g_device->GetImmediateContext(&g_context);
        return true;
    }
    if (!g_commandQueue) return false;
    ID3D12Device* d3d12 = nullptr;
    if (FAILED(swapChain->GetDevice(__uuidof(ID3D12Device), reinterpret_cast<void**>(&d3d12)))) return false;
    HRESULT created = D3D11On12CreateDevice(
        d3d12,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        nullptr,
        0,
        reinterpret_cast<IUnknown**>(&g_commandQueue),
        1,
        0,
        &g_device,
        &g_context,
        nullptr
    );
    if (SUCCEEDED(created)) g_device->QueryInterface(__uuidof(ID3D11On12Device), reinterpret_cast<void**>(&g_d3d11On12Device));
    d3d12->Release();
    return g_device != nullptr;
}

bool initOverlay(IDXGISwapChain* swapChain) {
    if (g_overlayReady) return true;
    if (!createDevices(swapChain)) return false;
    DXGI_SWAP_CHAIN_DESC desc;
    swapChain->GetDesc(&desc);
    g_hwnd = desc.OutputWindow ? desc.OutputWindow : FindWindowW(L"Minecraft", nullptr);
    installWndProcHook();
    g_overlayReady = true;
    return true;
}

void renderFrame(ID3D11RenderTargetView* rtv) {
    g_context->OMSetRenderTargets(1, &rtv, nullptr);
    auto overlayStart = std::chrono::steady_clock::now();
    FrameRenderer::render(g_device, g_context, g_hwnd);
    if (FrameRenderer::captureView()) ViewCapture::capture(g_device, g_context, rtv);
    auto overlayUs = std::chrono::duration_cast<std::chrono::microseconds>(std::chrono::steady_clock::now() - overlayStart);
    FrameRenderer::reportFrame(static_cast<uint32_t>(overlayUs.count()));
    ID3D11RenderTargetView* none = nullptr;
    g_context->OMSetRenderTargets(1, &none, nullptr);
    rtv->Release();
}

void renderOverlay(IDXGISwapChain* swapChain) {
    if (g_shuttingDown.load(std::memory_order_acquire)) return;
    g_renderInFlight.fetch_add(1, std::memory_order_acq_rel);
    struct InFlight {
        ~InFlight() { g_renderInFlight.fetch_sub(1, std::memory_order_acq_rel); }
    } inFlight;
    if (g_shuttingDown.load(std::memory_order_acquire) || !initOverlay(swapChain)) return;

    if (g_d3d11On12Device) {
        UINT             index = 0;
        IDXGISwapChain3* chain3 = nullptr;
        if (SUCCEEDED(swapChain->QueryInterface(__uuidof(IDXGISwapChain3), reinterpret_cast<void**>(&chain3)))) {
            index = chain3->GetCurrentBackBufferIndex();
            chain3->Release();
        }
        ID3D12Resource* backBuffer = nullptr;
        if (FAILED(swapChain->GetBuffer(index, __uuidof(ID3D12Resource), reinterpret_cast<void**>(&backBuffer)))) return;
        ID3D11Resource*      wrapped = nullptr;
        D3D11_RESOURCE_FLAGS flags   = {D3D11_BIND_RENDER_TARGET};
        if (SUCCEEDED(g_d3d11On12Device->CreateWrappedResource(
                backBuffer,
                &flags,
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_PRESENT,
                __uuidof(ID3D11Resource),
                reinterpret_cast<void**>(&wrapped)
            ))) {
            ID3D11RenderTargetView* rtv = nullptr;
            g_device->CreateRenderTargetView(wrapped, nullptr, &rtv);
            g_d3d11On12Device->AcquireWrappedResources(&wrapped, 1);
            if (rtv) renderFrame(rtv);
            g_d3d11On12Device->ReleaseWrappedResources(&wrapped, 1);
            wrapped->Release();
            g_context->ClearState();
            g_context->Flush();
        }
        backBuffer->Release();
        return;
    }
    ID3D11Texture2D* backBuffer = nullptr;
    if (FAILED(swapChain->GetBuffer(0, __uuidof(ID3D11Texture2D), reinterpret_cast<void**>(&backBuffer)))) return;
    ID3D11RenderTargetView* rtv = nullptr;
    g_device->CreateRenderTargetView(backBuffer, nullptr, &rtv);
    backBuffer->Release();
    if (rtv) {
        renderFrame(rtv);
        g_context->ClearState();
        g_context->Flush();
    }
}

HRESULT __stdcall hkPresent(IDXGISwapChain* swapChain, UINT sync, UINT flags) {
    renderOverlay(swapChain);
    return oPresent(swapChain, sync, flags);
}

HRESULT __stdcall hkPresent1(IDXGISwapChain1* swapChain, UINT sync, UINT flags, const DXGI_PRESENT_PARAMETERS* params) {
    renderOverlay(swapChain);
    return oPresent1(swapChain, sync, flags, params);
}

template <typename T>
void hookVtable(void* entry, void* detour, ll::memory::FuncPtr& target, ll::memory::FuncPtr& detourOut, T& original) {
    target    = ll::memory::toFuncPtr(reinterpret_cast<T>(entry));
    detourOut = ll::memory::toFuncPtr(reinterpret_cast<T>(detour));
    ll::memory::hook(target, detourOut, reinterpret_cast<ll::memory::FuncPtr*>(&original), ll::memory::HookPriority::Normal);
}

void unhook(ll::memory::FuncPtr target, ll::memory::FuncPtr detour) {
    if (target && detour) ll::memory::unhook(target, detour);
}

} // namespace

// creates throwaway devices only to read the swap chain and command queue vtables
bool DX11Hook::init() {
    g_shuttingDown.store(false, std::memory_order_release);
    HWND hwnd = FindWindowW(L"Minecraft", nullptr);
    if (!hwnd) hwnd = GetForegroundWindow();
    if (!hwnd) return false;

    D3D_FEATURE_LEVEL    level = D3D_FEATURE_LEVEL_11_0;
    DXGI_SWAP_CHAIN_DESC sd    = {};
    sd.BufferCount             = 1;
    sd.BufferDesc.Format       = DXGI_FORMAT_R8G8B8A8_UNORM;
    sd.BufferUsage             = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    sd.OutputWindow            = hwnd;
    sd.SampleDesc.Count        = 1;
    sd.Windowed                = TRUE;
    sd.SwapEffect              = DXGI_SWAP_EFFECT_DISCARD;
    ID3D11Device*        device  = nullptr;
    IDXGISwapChain*      chain   = nullptr;
    ID3D11DeviceContext* context = nullptr;
    if (FAILED(D3D11CreateDeviceAndSwapChain(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, 0, &level, 1, D3D11_SDK_VERSION, &sd, &chain, &device, nullptr, &context))) {
        return false;
    }
    void** vtable = *reinterpret_cast<void***>(chain);
    hookVtable(vtable[8], reinterpret_cast<void*>(&hkPresent), g_targetPresent, g_detourPresent, oPresent);
    hookVtable(vtable[13], reinterpret_cast<void*>(&hkResizeBuffers), g_targetResizeBuffers, g_detourResizeBuffers, oResizeBuffers);
    IDXGISwapChain1* chain1 = nullptr;
    if (SUCCEEDED(chain->QueryInterface(__uuidof(IDXGISwapChain1), reinterpret_cast<void**>(&chain1)))) {
        void** vtable1 = *reinterpret_cast<void***>(chain1);
        hookVtable(vtable1[22], reinterpret_cast<void*>(&hkPresent1), g_targetPresent1, g_detourPresent1, oPresent1);
        chain1->Release();
    }
    chain->Release();
    device->Release();
    context->Release();

    ID3D12Device* d3d12 = nullptr;
    if (SUCCEEDED(D3D12CreateDevice(nullptr, D3D_FEATURE_LEVEL_11_0, __uuidof(ID3D12Device), reinterpret_cast<void**>(&d3d12)))) {
        D3D12_COMMAND_QUEUE_DESC queueDesc = {};
        queueDesc.Type                     = D3D12_COMMAND_LIST_TYPE_DIRECT;
        ID3D12CommandQueue* queue          = nullptr;
        if (SUCCEEDED(d3d12->CreateCommandQueue(&queueDesc, __uuidof(ID3D12CommandQueue), reinterpret_cast<void**>(&queue)))) {
            void** vtable12 = *reinterpret_cast<void***>(queue);
            hookVtable(vtable12[10], reinterpret_cast<void*>(&hkExecuteCommandLists), g_targetExecuteCommandLists, g_detourExecuteCommandLists, oExecuteCommandLists);
            queue->Release();
        }
        d3d12->Release();
    }
    return true;
}

// stops new frames, waits up to 2 s for frames in flight, then releases everything
void DX11Hook::shutdown() {
    g_shuttingDown.store(true, std::memory_order_release);
    for (int i = 0; i < 200 && g_renderInFlight.load(std::memory_order_acquire) > 0; ++i) Sleep(10);
    unhook(g_targetPresent, g_detourPresent);
    unhook(g_targetPresent1, g_detourPresent1);
    unhook(g_targetResizeBuffers, g_detourResizeBuffers);
    unhook(g_targetExecuteCommandLists, g_detourExecuteCommandLists);
    TextureCache::releaseAll();
    shutdownOverlay();
    removeWndProcHook();
    if (g_context) {
        g_context->Release();
        g_context = nullptr;
    }
    if (g_device) {
        g_device->Release();
        g_device = nullptr;
    }
    if (g_d3d11On12Device) {
        g_d3d11On12Device->Release();
        g_d3d11On12Device = nullptr;
    }
    if (g_commandQueue) {
        g_commandQueue->Release();
        g_commandQueue = nullptr;
    }
    g_hwnd = nullptr;
}

ID3D11Device* DX11Hook::getDevice() { return g_device; }

ID3D11DeviceContext* DX11Hook::getContext() { return g_context; }

HWND DX11Hook::getHwnd() { return g_hwnd; }

} // namespace levi_minimap
