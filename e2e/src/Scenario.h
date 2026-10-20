#pragma once

#include "levi_minimap.h"

#include <filesystem>
#include <set>
#include <string>
#include <vector>

namespace lm_e2e {

// drives the static library through the C header exactly as the shim does, over a synthetic world
class Scenario {
public:
    explicit Scenario(std::filesystem::path root);
    ~Scenario();

    bool create();
    bool enterWorld(int32_t dim = 0);
    void leaveWorld();
    void destroy();

    // one game tick: tick, then probe candidates up to the plan's limit
    void tick();
    void frame(bool leftDown, float cursorX, float cursorY, bool rightDown = false);
    // one frame with the button down at the point, then one with it up
    void click(float x, float y, bool right);
    // each character through lm_text, as WM_CHAR delivers it
    void type(const std::string& text);
    // the message line, if one is drawn, starts with `prefix`
    bool drawsTextStarting(const std::string& prefix) const;
    // one frame with the input a capture recorded; counts and checks its texture work
    void frameWith(const LmFrameInput& input);
    void key(int code);

    LmStats stats() const;
    // the last frame laid out exactly this text
    bool drawsText(const std::string& text) const;
    // the centre of the last text that is exactly `text` (panels are drawn after the map); false
    // when the last frame has none
    bool textRect(const std::string& text, float& centreX, float& centreY) const;
    // the last frame's UI vertices
    uint32_t vertexCount() const;
    // hovers and clicks the text, then draws two frames with the cursor on the map so a panel that
    // opens is laid out; false when the text is not drawn
    bool clickText(const std::string& text);

    const LmHandle* handle() const { return mHandle; }

    const LmFrame& lastFrame() const { return mFrame; }

    float playerX = 8.5f;
    float playerZ = 8.5f;
    // the probe's starting height in the last scan plan
    int32_t cameraHeight = 0;

    uint64_t uploads  = 0;
    uint64_t releases = 0;
    // texture ids uploaded and not yet released
    std::set<uint32_t> liveTextures;
    // the last clipboard text a frame handed over
    std::string clipboard;

private:
    static short height(int x, int z);
    std::string  textAt(uint32_t index) const;
    void         probe(const LmChunkPos& pos, LmChunkSurface* out) const;

    std::filesystem::path mRoot;
    std::vector<std::string> mStrings;
    LmHandle*             mHandle = nullptr;
    LmFrame               mFrame{};
};

} // namespace lm_e2e
