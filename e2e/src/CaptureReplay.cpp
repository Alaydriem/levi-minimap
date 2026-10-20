#include "CaptureReplay.h"

#include <cstdio>
#include <cstring>
#include <fstream>
#include <vector>

namespace lm_e2e {

namespace {

constexpr uint8_t kEnter  = 1;
constexpr uint8_t kLeave  = 2;
constexpr uint8_t kTick   = 3;
constexpr uint8_t kSubmit = 4;
constexpr uint8_t kColumn = 5;
constexpr uint8_t kBlock  = 6;
constexpr uint8_t kKey    = 7;
constexpr uint8_t kMouse  = 8;
constexpr uint8_t kFrame  = 9;
constexpr uint8_t kBiomeName = 10;
// a biome name record holds this many name bytes
constexpr size_t kNameBytes = 64;

constexpr size_t kHeaderBytes = 20;

// reads little-endian fields from one record
class Fields {
public:
    explicit Fields(const std::vector<uint8_t>& bytes) : mBytes(bytes) {}

    template <typename T>
    T get() {
        T value{};
        std::memcpy(&value, mBytes.data() + mAt, sizeof(T));
        mAt += sizeof(T);
        return value;
    }

    const uint8_t* here() const { return mBytes.data() + mAt; }

private:
    const std::vector<uint8_t>& mBytes;
    size_t                      mAt = 0;
};

LmChunkPos pos(Fields& f) {
    LmChunkPos p{};
    p.x   = f.get<int32_t>();
    p.z   = f.get<int32_t>();
    p.dim = f.get<int32_t>();
    return p;
}

void apply(Scenario& scenario, uint8_t tag, const std::vector<uint8_t>& payload, ReplayResult& result) {
    const LmHandle* handle = scenario.handle();
    Fields          f(payload);
    switch (tag) {
    case kEnter: scenario.enterWorld(f.get<int32_t>()); break;
    case kLeave: scenario.leaveWorld(); break;
    case kTick: {
        LmTickInput input{};
        input.present       = f.get<uint8_t>();
        input.x             = f.get<float>();
        input.y             = f.get<float>();
        input.z             = f.get<float>();
        input.yaw           = f.get<float>();
        input.dim           = f.get<int32_t>();
        input.foreground    = f.get<uint8_t>();
        input.cursor_hidden = f.get<uint8_t>();
        LmScanPlan plan{};
        lm_tick(handle, &input, &plan);
        LmStats stats = scenario.stats();
        result.maxBakes = stats.bakes_completed;
        if (stats.surfaces_resident > stats.surface_capacity
            || stats.tiles_resident > stats.tile_capacity + stats.tile_capacity / 4) {
            result.withinBudget = false;
        }
        break;
    }
    case kSubmit: {
        LmChunkPos p      = pos(f);
        uint8_t    status = f.get<uint8_t>();
        std::memcpy(lm_staging(handle), f.here(), sizeof(LmChunkSurface));
        lm_submit_chunk(handle, &p, status);
        break;
    }
    case kColumn: {
        LmChunkPos     p     = pos(f);
        uint32_t       index = f.get<uint32_t>();
        LmColumnSample sample{};
        sample.surface = f.get<uint32_t>();
        sample.water   = f.get<uint32_t>();
        sample.height  = f.get<int16_t>();
        sample.solid   = f.get<int16_t>();
        sample.retry   = f.get<uint8_t>();
        sample.biome   = f.get<uint16_t>();
        lm_submit_column(handle, &p, index, &sample);
        break;
    }
    case kBiomeName: {
        uint16_t id  = f.get<uint16_t>();
        uint8_t  len = f.get<uint8_t>();
        if (len > kNameBytes) break;
        std::string name(reinterpret_cast<const char*>(f.here()), len);
        lm_biome_name(handle, id, LmStr{reinterpret_cast<const uint8_t*>(name.data()), name.size()});
        break;
    }
    case kBlock: {
        LmChunkPos     p = pos(f);
        LmProbeRequest request{};
        lm_block_changed(handle, p.x, p.z, p.dim, &request);
        break;
    }
    case kKey: {
        int32_t     code   = f.get<int32_t>();
        uint8_t     down   = f.get<uint8_t>();
        uint8_t     ctrl   = f.get<uint8_t>();
        uint8_t     grab   = f.get<uint8_t>();
        LmKeyResult answer{};
        lm_key(handle, code, down, ctrl, grab, &answer);
        break;
    }
    case kMouse: {
        int32_t button = f.get<int32_t>();
        int32_t data   = f.get<int32_t>();
        lm_mouse(handle, button, data);
        break;
    }
    case kFrame: {
        LmFrameInput input{};
        input.display_width  = f.get<float>();
        input.display_height = f.get<float>();
        input.dt             = f.get<float>();
        input.cursor_x       = f.get<float>();
        input.cursor_y       = f.get<float>();
        input.left_down      = f.get<uint8_t>();
        input.foreground     = f.get<uint8_t>();
        scenario.frameWith(input);
        ++result.frames;
        break;
    }
    default: break;
    }
}

} // namespace

ReplayResult CaptureReplay::run(Scenario& scenario, const std::filesystem::path& path) {
    ReplayResult  result;
    std::ifstream in(path, std::ios::binary);
    uint8_t       header[kHeaderBytes];
    if (!in.read(reinterpret_cast<char*>(header), kHeaderBytes) || std::memcmp(header, "LMCP", 4) != 0 || header[6] != 0) {
        std::printf("     %s is not an unpacked capture\n", path.string().c_str());
        return result;
    }
    uint32_t jsonLen = 0;
    std::memcpy(&jsonLen, header + 16, 4);
    in.seekg(jsonLen, std::ios::cur);
    std::vector<uint8_t> payload;
    uint8_t              head[5];
    while (in.read(reinterpret_cast<char*>(head), 5)) {
        uint32_t len = 0;
        std::memcpy(&len, head + 1, 4);
        payload.resize(len);
        if (len && !in.read(reinterpret_cast<char*>(payload.data()), len)) break;
        apply(scenario, head[0], payload, result);
        ++result.events;
    }
    result.ok = true;
    return result;
}

} // namespace lm_e2e
