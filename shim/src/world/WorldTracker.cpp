#include "world/WorldTracker.h"

#include "bridge/RustBridge.h"
#include "mod/MinimapMod.h"
#include "world/BlockListener.h"
#include "world/ColumnProbe.h"
#include "world/GameWindow.h"

#include <mc/client/game/ClientInstance.h>
#include <mc/client/player/LocalPlayer.h>
#include <mc/network/GameConnectionInfo.h>
#include <mc/world/level/BlockSource.h>
#include <mc/world/level/Level.h>
#include <mc/world/level/biome/Biome.h>
#include <mc/world/level/biome/BiomeIdType.h>
#include <mc/world/level/biome/registry/BiomeRegistry.h>

#include <windows.h>

#include <psapi.h>

#include <string>

namespace levi_minimap {

namespace {

bool         g_inWorld        = false;
uint64_t     g_workingSet     = 0;
uint64_t     g_privateBytes   = 0;
uint32_t     g_ticks          = 0;
int          g_dimension      = -1;
BlockSource* g_listenerSource = nullptr;

void attachListener(BlockSource& source) {
    if (g_listenerSource) g_listenerSource->removeListener(BlockListener::getInstance());
    g_listenerSource = &source;
    g_listenerSource->addListener(BlockListener::getInstance());
}

void enterWorld(const LmHandle* handle, ClientInstance& client, LocalPlayer& player, int dimension) {
    std::string localDir;
    std::string host;
    uint16_t    port = 0;
    if (auto worldDir = MinimapMod::getInstance().getSelf().getWorldDataDir(); worldDir.has_value()) {
        localDir = RustBridge::utf8(*worldDir);
    } else if (auto info = client.getGameConnectionInfo(); info.has_value()) {
        host = info->mHostIpAddress;
        port = static_cast<uint16_t>(info->mPort);
    }
    LmWorldInfo world{
        RustBridge::str(localDir),
        RustBridge::str(host),
        port,
        static_cast<uint32_t>(player.getLevel().getSeed()),
        dimension
    };
    lm_enter_world(handle, &world);
}

// a probed chunk names the plan's requested biome IDs it holds, from its own biome objects
void nameProbedBiomes(const LmHandle* handle, BlockSource& region, const LmChunkPos& pos, const LmChunkSurface& probed, const LmScanPlan& plan) {
    for (uint32_t r = 0; r < plan.biome_request_count; ++r) {
        uint16_t id = plan.biome_requests[r];
        for (uint32_t index = 0; index < 256; ++index) {
            if (probed.biome[index] != id) continue;
            std::string name = ColumnProbe::biomeName(region, pos, index);
            if (!name.empty()) lm_biome_name(handle, id, RustBridge::str(name));
            break;
        }
    }
}

void probePlan(const LmHandle* handle, BlockSource& region, const LmScanPlan& plan) {
    uint32_t loaded = 0;
    for (uint32_t i = 0; i < plan.count; ++i) {
        const LmChunkPos& pos = plan.candidates[i];
        if (loaded >= plan.limit) {
            lm_submit_chunk(handle, &pos, ColumnProbe::NOT_PROBED);
            continue;
        }
        LmChunkSurface* staging = lm_staging(handle);
        uint8_t         status  = ColumnProbe::probeChunk(region, pos, staging);
        if (status == ColumnProbe::LOADED) {
            ++loaded;
            if (staging) nameProbedBiomes(handle, region, pos, *staging, plan);
        }
        lm_submit_chunk(handle, &pos, status);
    }
}

// the names the level's biome registry knows; an ID it does not know stays asked, and a probed chunk
// holding that biome names it
void answerBiomes(const LmHandle* handle, Level& level, const LmScanPlan& plan) {
    if (plan.biome_request_count == 0) return;
    auto& registry = level.getBiomeRegistry();
    for (uint32_t i = 0; i < plan.biome_request_count; ++i) {
        uint16_t    id = plan.biome_requests[i];
        BiomeIdType key{};
        key.mValue   = id;
        Biome* biome = registry.lookupById(key);
        if (!biome) continue;
        std::string name = biome->mHash->getString();
        if (!name.empty()) lm_biome_name(handle, id, RustBridge::str(name));
    }
}

// process memory for telemetry, read about once a second at 20 ticks a second; the call takes microseconds
void sampleMemory() {
    constexpr uint32_t kEveryTicks = 20;
    if (g_ticks++ % kEveryTicks != 0) return;
    PROCESS_MEMORY_COUNTERS_EX counters{};
    counters.cb = sizeof(counters);
    if (K32GetProcessMemoryInfo(GetCurrentProcess(), reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&counters), sizeof(counters))) {
        g_workingSet   = counters.WorkingSetSize;
        g_privateBytes = counters.PrivateUsage;
    }
}

// the window and cursor state every tick sends, in a world or not
LmTickInput windowInput() {
    HWND       hwnd = GameWindow::hwnd();
    CURSORINFO cursor{sizeof(CURSORINFO)};
    LmTickInput input{};
    input.foreground    = static_cast<uint8_t>(hwnd && GetForegroundWindow() == hwnd);
    input.cursor_hidden = static_cast<uint8_t>(GetCursorInfo(&cursor) && cursor.flags == 0);
    return input;
}

// out of a world Rust still ticks with no player: it drains the manager's events and gives its leases back
void tickOutOfWorld(const LmHandle* handle, ClientInstance& client) {
    LmTickInput input   = windowInput();
    input.mouse_grabbed = static_cast<uint8_t>(client.getMouseGrabbed());
    LmScanPlan plan{};
    if (lm_tick(handle, &input, &plan) != LM_STATUS_OK) return;
    WorldTracker::applyCursor(plan.cursor);
}

} // namespace

void WorldTracker::update(ClientInstance& client) {
    const LmHandle* handle = RustBridge::handle();
    if (!handle) return;
    auto* player  = client.getLocalPlayer();
    bool  inWorld = player && client.isWorldActive();
    if (!inWorld) {
        if (g_inWorld) {
            detach();
            lm_leave_world(handle);
            g_inWorld = false;
        }
        tickOutOfWorld(handle, client);
        return;
    }

    int dimension = static_cast<int>(player->getDimensionId());
    if (!g_inWorld) {
        enterWorld(handle, client, *player, dimension);
        attachListener(player->getDimensionBlockSource());
        g_inWorld   = true;
        g_dimension = dimension;
    } else if (dimension != g_dimension) {
        attachListener(player->getDimensionBlockSource());
        g_dimension = dimension;
    }

    const auto& pos = player->getPosition();
    sampleMemory();
    LmTickInput input       = windowInput();
    input.present           = 1;
    input.dead              = static_cast<uint8_t>(!player->isAlive());
    input.mouse_grabbed     = static_cast<uint8_t>(client.getMouseGrabbed());
    input.x                 = pos.x;
    input.y                 = pos.y;
    input.z                 = pos.z;
    input.yaw               = player->getRotation().y;
    input.dim               = dimension;
    input.working_set_bytes = g_workingSet;
    input.private_bytes     = g_privateBytes;
    LmScanPlan plan{};
    if (lm_tick(handle, &input, &plan) != LM_STATUS_OK) return;
    ColumnProbe::configure(plan.camera_height, plan.transparent_water != 0, plan.block_colors, plan.block_color_count);
    answerBiomes(handle, player->getLevel(), plan);
    if (auto* region = client.getRegion()) probePlan(handle, *region, plan);
    applyCursor(plan.cursor);
}

void WorldTracker::applyCursor(unsigned char action) {
    HWND hwnd = GameWindow::hwnd();
    if (!hwnd || GetForegroundWindow() != hwnd) return;
    if (action == 1) {
        RECT client;
        GetClientRect(hwnd, &client);
        POINT centre{(client.right - client.left) / 2, (client.bottom - client.top) / 2};
        ClientToScreen(hwnd, &centre);
        RECT box{centre.x - 1, centre.y - 1, centre.x + 1, centre.y + 1};
        ClipCursor(&box);
    } else if (action == 2) {
        ClipCursor(nullptr);
    }
}

void WorldTracker::detach() {
    if (g_listenerSource) {
        g_listenerSource->removeListener(BlockListener::getInstance());
        g_listenerSource = nullptr;
    }
}

void WorldTracker::reset() {
    detach();
    g_inWorld   = false;
    g_dimension = -1;
}

} // namespace levi_minimap
