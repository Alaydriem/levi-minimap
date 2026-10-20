#include "world/WorldTracker.h"

#include "bridge/RustBridge.h"
#include "input/InputListeners.h"
#include "mod/MinimapMod.h"
#include "render/DX11Hook.h"
#include "world/BlockListener.h"
#include "world/ColumnProbe.h"

#include <mc/client/game/ClientInstance.h>
#include <mc/client/player/LocalPlayer.h>
#include <mc/network/GameConnectionInfo.h>
#include <mc/world/level/BlockSource.h>
#include <mc/world/level/Level.h>
#include <mc/world/level/biome/Biome.h>
#include <mc/world/level/biome/BiomeIdType.h>
#include <mc/world/level/biome/registry/BiomeRegistry.h>

#include <windows.h>

#include <string>

namespace levi_minimap {

namespace {

bool         g_inWorld        = false;
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
            InputListeners::setClientInstance(nullptr);
            g_inWorld = false;
        }
        return;
    }

    int dimension = static_cast<int>(player->getDimensionId());
    if (!g_inWorld) {
        enterWorld(handle, client, *player, dimension);
        InputListeners::setClientInstance(&client);
        attachListener(player->getDimensionBlockSource());
        g_inWorld   = true;
        g_dimension = dimension;
    } else if (dimension != g_dimension) {
        attachListener(player->getDimensionBlockSource());
        g_dimension = dimension;
    }

    HWND       hwnd       = DX11Hook::getHwnd();
    bool       foreground = hwnd && GetForegroundWindow() == hwnd;
    CURSORINFO cursor{sizeof(CURSORINFO)};
    bool       hidden = GetCursorInfo(&cursor) && cursor.flags == 0;
    const auto& pos   = player->getPosition();
    LmTickInput input{
        1,
        static_cast<uint8_t>(foreground),
        static_cast<uint8_t>(hidden),
        static_cast<uint8_t>(!player->isAlive()),
        pos.x,
        pos.y,
        pos.z,
        player->getRotation().y,
        dimension
    };
    LmScanPlan plan{};
    if (lm_tick(handle, &input, &plan) != LM_STATUS_OK) return;
    ColumnProbe::configure(plan.camera_height, plan.transparent_water != 0, plan.block_colors, plan.block_color_count);
    answerBiomes(handle, player->getLevel(), plan);
    if (auto* region = client.getRegion()) probePlan(handle, *region, plan);
    applyCursor(plan.cursor);
}

void WorldTracker::applyCursor(unsigned char action) {
    HWND hwnd = DX11Hook::getHwnd();
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

} // namespace levi_minimap
