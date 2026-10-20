#include "Scenario.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <fstream>

namespace lm_e2e {

namespace {

LmStr str(const std::string& text) { return LmStr{reinterpret_cast<const uint8_t*>(text.data()), text.size()}; }

void logToStdout(uint8_t level, const uint8_t* message, uintptr_t len) {
    if (level <= 2) std::printf("[rust %u] %.*s\n", level, static_cast<int>(len), reinterpret_cast<const char*>(message));
}

std::string utf8(const std::filesystem::path& path) {
    auto bytes = path.u8string();
    return std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
}

} // namespace

Scenario::Scenario(std::filesystem::path root) : mRoot(std::move(root)) {
    mStrings = {
        utf8(mRoot / "config" / "config.json"),
        utf8(mRoot / "data"),
        utf8(mRoot / "world"),
    };
}

Scenario::~Scenario() { destroy(); }

// exports go under the scenario's own folder, never to the user's Pictures folder
bool Scenario::create() {
    std::filesystem::path config = mRoot / "config" / "config.json";
    if (!std::filesystem::exists(config)) {
        std::filesystem::create_directories(config.parent_path());
        std::ofstream(config) << "{ \"export\": { \"folder\": \"" << (mRoot / "data" / "exports").generic_string() << "\" } }";
    }
    LmPaths paths{str(mStrings[0]), str(mStrings[1])};
    mHandle = lm_create(&paths, &logToStdout);
    return mHandle != nullptr;
}

bool Scenario::enterWorld(int32_t dim) {
    LmWorldInfo info{str(mStrings[2]), str(std::string()), 0, 0, dim};
    return lm_enter_world(mHandle, &info) == LM_STATUS_OK;
}

void Scenario::leaveWorld() { lm_leave_world(mHandle); }

void Scenario::destroy() {
    if (mHandle) lm_destroy(mHandle);
    mHandle = nullptr;
}

short Scenario::height(int x, int z) {
    return static_cast<short>(64 + std::abs(((x % 23) + 23) % 23 - 11) + std::abs(((z % 17) + 17) % 17 - 8));
}

void Scenario::probe(const LmChunkPos& pos, LmChunkSurface* out) const {
    for (int z = 0; z < 16; ++z) {
        for (int x = 0; x < 16; ++x) {
            int   index = z * 16 + x;
            short h     = height(pos.x * 16 + x, pos.z * 16 + z);
            auto  shade = static_cast<uint32_t>(std::clamp((h - 40) * 4, 0, 255));
            out->surface[index] = shade | (160u << 8) | (60u << 16) | (255u << 24);
            out->height[index]  = h;
            out->solid[index]   = h;
            out->biome[index]   = 1;
        }
    }
}

void Scenario::tick() {
    LmTickInput input{1, 1, 1, 0, playerX, 80.0f, playerZ, 0.0f, 0};
    LmScanPlan  plan{};
    if (lm_tick(mHandle, &input, &plan) != LM_STATUS_OK) return;
    cameraHeight = plan.camera_height;
    // the game names biome 1 "plains" and has no other
    for (uint32_t i = 0; i < plan.biome_request_count; ++i) {
        uint16_t    id   = plan.biome_requests[i];
        std::string name = id == 1 ? "plains" : "";
        lm_biome_name(mHandle, id, str(name));
    }
    uint32_t loaded = 0;
    for (uint32_t i = 0; i < plan.count; ++i) {
        if (loaded >= plan.limit) {
            lm_submit_chunk(mHandle, &plan.candidates[i], 3);
            continue;
        }
        LmChunkSurface* staging = lm_staging(mHandle);
        probe(plan.candidates[i], staging);
        lm_submit_chunk(mHandle, &plan.candidates[i], 0);
        ++loaded;
    }
}

void Scenario::frame(bool leftDown, float cursorX, float cursorY, bool rightDown) {
    LmFrameInput input{1280.0f, 720.0f, 0.016f, cursorX, cursorY, static_cast<uint8_t>(leftDown), 1, static_cast<uint8_t>(rightDown)};
    input.collect_text = 1;
    frameWith(input);
}

void Scenario::click(float x, float y, bool right) {
    frame(!right, x, y, right);
    frame(false, x, y, false);
}

void Scenario::type(const std::string& text) {
    for (unsigned char c : text) lm_text(mHandle, c);
}

void Scenario::frameWith(const LmFrameInput& input) {
    if (lm_frame(mHandle, &input, &mFrame) != LM_STATUS_OK) return;
    if (mFrame.clipboard_len > 0) clipboard.assign(reinterpret_cast<const char*>(mFrame.clipboard), mFrame.clipboard_len);
    for (uint32_t i = 0; i < mFrame.texture_count; ++i) {
        const LmTextureOp& op = mFrame.textures[i];
        if (op.kind == 1) {
            ++uploads;
            liveTextures.insert(op.tex);
            if (op.len != static_cast<uintptr_t>(op.width) * op.height * 4) {
                std::printf("FAIL upload %u has %zu bytes for %ux%u\n", op.tex, static_cast<size_t>(op.len), op.width, op.height);
                std::exit(1);
            }
        }
        if (op.kind == 2) {
            ++releases;
            liveTextures.erase(op.tex);
        }
    }
}

void Scenario::key(int code) {
    LmKeyResult result{};
    lm_key(mHandle, code, 1, 0, 1, &result);
}

std::string Scenario::textAt(uint32_t index) const {
    const LmUiText& t = mFrame.texts[index];
    if (static_cast<uint64_t>(t.start) + t.len > mFrame.text_bytes_len) return {};
    return std::string(reinterpret_cast<const char*>(mFrame.text_bytes + t.start), t.len);
}

bool Scenario::drawsText(const std::string& text) const {
    for (uint32_t i = 0; i < mFrame.text_count; ++i) {
        if (textAt(i) == text) return true;
    }
    return false;
}

bool Scenario::textRect(const std::string& text, float& centreX, float& centreY) const {
    for (uint32_t i = mFrame.text_count; i-- > 0;) {
        if (textAt(i) != text) continue;
        const float* rect = mFrame.texts[i].rect;
        centreX           = (rect[0] + rect[2]) * 0.5f;
        centreY           = (rect[1] + rect[3]) * 0.5f;
        return true;
    }
    return false;
}

uint32_t Scenario::vertexCount() const { return mFrame.vertex_count; }

bool Scenario::clickText(const std::string& text) {
    float x = 0.0f;
    float y = 0.0f;
    if (!textRect(text, x, y)) return false;
    frame(false, x, y);
    click(x, y, false);
    frame(false, 640.0f, 360.0f);
    frame(false, 640.0f, 360.0f);
    return true;
}

bool Scenario::drawsTextStarting(const std::string& prefix) const {
    for (uint32_t i = 0; i < mFrame.text_count; ++i) {
        if (textAt(i).rfind(prefix, 0) == 0) return true;
    }
    return false;
}

LmStats Scenario::stats() const {
    LmStats out{};
    lm_stats(mHandle, &out);
    return out;
}

} // namespace lm_e2e
