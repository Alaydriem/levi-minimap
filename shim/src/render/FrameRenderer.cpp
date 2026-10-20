#include "render/FrameRenderer.h"

#include "bridge/RustBridge.h"
#include "render/ClipboardWriter.h"
#include "render/EguiRenderer.h"
#include "render/TextureCache.h"

#include <psapi.h>

#include <atomic>
#include <chrono>

namespace levi_minimap {

namespace {

constexpr uint32_t kDrawCursor  = 1;
constexpr uint32_t kReleaseClip = 2;
constexpr uint32_t kTextInput   = 4;
constexpr uint32_t kCaptureView = 8;

std::atomic<bool> g_mapOpen{false};
std::atomic<bool> g_textInput{false};
bool              g_captureView = false;

constexpr uint32_t kMemoryEveryFrames = 60;
constexpr float    kFallbackDt        = 1.0f / 60.0f;

uint32_t g_renderUs     = 0;
uint32_t g_vertexCount  = 0;
uint32_t g_frames       = 0;
uint64_t g_workingSet   = 0;
uint64_t g_privateBytes = 0;

std::chrono::steady_clock::time_point g_lastFrame = std::chrono::steady_clock::now();

// process memory, read once a second at 60 fps; the call takes a few microseconds
void sampleMemory() {
    if (g_frames++ % kMemoryEveryFrames != 0) return;
    PROCESS_MEMORY_COUNTERS_EX counters{};
    counters.cb = sizeof(counters);
    if (K32GetProcessMemoryInfo(GetCurrentProcess(), reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&counters), sizeof(counters))) {
        g_workingSet   = counters.WorkingSetSize;
        g_privateBytes = counters.PrivateUsage;
    }
}

float frameDt() {
    auto  now = std::chrono::steady_clock::now();
    float dt  = std::chrono::duration<float>(now - g_lastFrame).count();
    g_lastFrame = now;
    return dt > 0.0f ? dt : kFallbackDt;
}

} // namespace

void FrameRenderer::render(ID3D11Device* device, ID3D11DeviceContext* context, HWND hwnd) {
    const LmHandle* handle = RustBridge::handle();
    float           dt     = frameDt();
    if (!handle || !EguiRenderer::init(device)) return;
    RECT client{};
    if (!hwnd || !GetClientRect(hwnd, &client)) return;
    float width      = static_cast<float>(client.right - client.left);
    float height     = static_cast<float>(client.bottom - client.top);
    bool  foreground = GetForegroundWindow() == hwnd;
    POINT cursor{};
    if (foreground && !(GetCursorPos(&cursor) && ScreenToClient(hwnd, &cursor))) cursor = POINT{};
    bool  leftDown   = foreground && (GetAsyncKeyState(VK_LBUTTON) & 0x8000) != 0;
    bool  rightDown  = foreground && (GetAsyncKeyState(VK_RBUTTON) & 0x8000) != 0;
    sampleMemory();
    LmFrameInput input{};
    input.display_width     = width;
    input.display_height    = height;
    input.dt                = dt;
    input.cursor_x          = static_cast<float>(cursor.x);
    input.cursor_y          = static_cast<float>(cursor.y);
    input.left_down         = static_cast<uint8_t>(leftDown);
    input.foreground        = static_cast<uint8_t>(foreground);
    input.right_down        = static_cast<uint8_t>(rightDown);
    input.render_us         = g_renderUs;
    input.ui_vtx            = g_vertexCount;
    input.working_set_bytes = g_workingSet;
    input.private_bytes     = g_privateBytes;
    LmFrame frame{};
    if (lm_frame(handle, &input, &frame) != LM_STATUS_OK) return;
    TextureCache::apply(device, context, frame.textures, frame.texture_count);
    EguiRenderer::applyTextures(device, context, frame);

    g_captureView = (frame.cursor_flags & kCaptureView) != 0;
    g_mapOpen.store((frame.cursor_flags & kDrawCursor) != 0, std::memory_order_relaxed);
    g_textInput.store((frame.cursor_flags & kTextInput) != 0, std::memory_order_relaxed);
    if (frame.cursor_flags & kReleaseClip) {
        ClipCursor(nullptr);
        SetCursor(LoadCursorW(nullptr, IDC_ARROW));
    }
    ClipboardWriter::write(hwnd, frame.clipboard, frame.clipboard_len);
    EguiRenderer::draw(device, context, frame, width, height);
    EguiRenderer::freeTextures(frame);
    g_vertexCount = frame.vertex_count;
}

bool FrameRenderer::mapOpen() { return g_mapOpen.load(std::memory_order_relaxed); }

bool FrameRenderer::textInput() { return g_textInput.load(std::memory_order_relaxed); }

bool FrameRenderer::captureView() { return g_captureView; }

void FrameRenderer::reportFrame(uint32_t renderUs) { g_renderUs = renderUs; }

} // namespace levi_minimap
