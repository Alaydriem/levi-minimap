#include "CaptureReplay.h"
#include "Scenario.h"

#include <algorithm>
#include <chrono>
#include <cstdlib>
#include <cstdio>
#include <filesystem>
#include <fstream>
#include <sstream>
#include <functional>
#include <thread>
#include <vector>

namespace {

int g_failures = 0;

void check(bool condition, const char* what) {
    std::printf("%s %s\n", condition ? "PASS" : "FAIL", what);
    if (!condition) ++g_failures;
}

// runs ticks and frames until `done` holds or `steps` run out
bool runUntil(lm_e2e::Scenario& scenario, int steps, const std::function<bool()>& done) {
    for (int i = 0; i < steps; ++i) {
        scenario.tick();
        scenario.frame(false, 640.0f, 360.0f);
        if (done()) return true;
        std::this_thread::sleep_for(std::chrono::milliseconds(2));
    }
    return false;
}

std::string readText(const std::filesystem::path& path) {
    std::ifstream     file(path);
    std::stringstream text;
    text << file.rdbuf();
    return text.str();
}

// every unpacked capture in LM_E2E_CAPTURES replays through the C ABI the way the shim drives it
void replayCaptures(const std::filesystem::path& root) {
    const char* dir = std::getenv("LM_E2E_CAPTURES");
    if (!dir || !std::filesystem::is_directory(dir)) {
        std::printf("SKIP capture replay: LM_E2E_CAPTURES is not a folder\n");
        return;
    }
    std::vector<std::filesystem::path> captures;
    for (const auto& entry : std::filesystem::directory_iterator(dir)) {
        if (entry.path().extension() == ".lmraw") captures.push_back(entry.path());
    }
    std::sort(captures.begin(), captures.end());
    check(!captures.empty(), "at least one unpacked capture is found");
    for (const auto& capture : captures) {
        auto name  = capture.stem().string();
        auto where = root / ("replay-" + name);
        std::filesystem::create_directories(where);
        lm_e2e::Scenario scenario(where);
        if (!scenario.create()) {
            check(false, "the library starts for a capture replay");
            continue;
        }
        lm_e2e::ReplayResult result = lm_e2e::CaptureReplay::run(scenario, capture);
        std::printf("     %s: %llu events, %llu frames\n", name.c_str(), static_cast<unsigned long long>(result.events),
                    static_cast<unsigned long long>(result.frames));
        check(result.ok && result.events > 0, ("the capture " + name + " replays through the C ABI").c_str());
        check(result.maxBakes > 0 && scenario.uploads > 0, ("the replayed " + name + " bakes and uploads textures").c_str());
        check(result.withinBudget, ("memory stays inside its budgets while " + name + " replays").c_str());
        // the capture ends with leaving the world, so the next frame retires every texture
        LmFrameInput last{1920.0f, 1080.0f, 0.016f, 0.0f, 0.0f, 0, 1};
        scenario.frameWith(last);
        check(scenario.liveTextures.empty(), ("every texture uploaded during " + name + " is released").c_str());
        scenario.destroy();
    }
}

} // namespace

int main() {
    auto root = std::filesystem::temp_directory_path() / "levi-minimap-e2e";
    std::filesystem::remove_all(root);
    std::filesystem::create_directories(root);
    {
        lm_e2e::Scenario scenario(root);
        check(scenario.create(), "the library starts through lm_create");
        check(scenario.enterWorld(), "a local world is entered");
        check(runUntil(scenario, 3000, [&] { return scenario.stats().bakes_completed > 8; }), "chunks are scanned and baked");
        check(runUntil(scenario, 200, [&] { return scenario.uploads > 0; }), "the minimap texture is uploaded");
        check(scenario.vertexCount() > 100, "the minimap draws");

        scenario.key(0x4D);
        uint64_t before = scenario.uploads;
        check(runUntil(scenario, 300, [&] { return scenario.uploads > before; }), "the world map uploads region tiles");
        check((scenario.lastFrame().cursor_flags & 1) != 0, "the open world map asks the UI to draw the cursor");
        for (int i = 0; i < 60; ++i) scenario.frame(i > 0, 640.0f - static_cast<float>(i) * 20.0f, 360.0f);
        scenario.key(0x1B);

        LmStats walking{};
        for (int step = 0; step < 400; ++step) {
            scenario.playerX += 16.0f;
            scenario.tick();
            scenario.frame(false, 640.0f, 360.0f);
            walking = scenario.stats();
            if (walking.surfaces_resident > walking.surface_capacity || walking.tiles_resident > walking.tile_capacity + walking.tile_capacity / 4) {
                break;
            }
        }
        check(walking.surfaces_resident <= walking.surface_capacity, "chunk surfaces stay inside their budget on a long walk");
        check(walking.tiles_resident <= walking.tile_capacity + walking.tile_capacity / 4, "tiles stay inside their budget and overrun");

        scenario.playerX = 40.5;
        scenario.tick();
        scenario.key(0x4E);
        scenario.leaveWorld();
        check(std::filesystem::exists(root / "world" / "levi_minimap_cache"), "the cache folder is written beside the world");
        check(scenario.enterWorld(), "the world is entered again");
        scenario.playerX = 8.5f;
        scenario.key(0x4D);
        bool fromDisk = false;
        for (int i = 0; i < 600 && !fromDisk; ++i) {
            scenario.frame(false, 640.0f, 360.0f);
            scenario.tick();
            fromDisk = scenario.stats().tiles_loaded > 0;
            std::this_thread::sleep_for(std::chrono::milliseconds(2));
        }
        check(fromDisk, "tiles load from disk after re-entering");
        scenario.key(0x1B);
        check(runUntil(scenario, 300, [&] { return scenario.drawsText("Waypoint 1"); }), "the hotkey waypoint is drawn after re-entering");

        // the menu opens at the cursor and is laid out in the next frames; its rows are clicked by name
        scenario.key(0x4D);
        for (int i = 0; i < 30; ++i) scenario.frame(false, 640.0f, 360.0f);
        auto openMenu = [&](float x, float y) {
            scenario.click(x, y, true);
            scenario.frame(false, x, y);
            scenario.frame(false, x, y);
        };
        openMenu(640.0f, 360.0f);
        check(scenario.clickText("Copy coordinates") && scenario.clipboard.find(", ~, ") != std::string::npos,
              "copy coordinates puts the clicked column on the clipboard");
        openMenu(640.0f, 360.0f);
        scenario.clickText("Add waypoint here");
        check((scenario.lastFrame().cursor_flags & 4) != 0 && scenario.drawsText("New waypoint"), "the waypoint dialog asks for text input");
        scenario.type("Base");
        scenario.key(0x0D);
        check(runUntil(scenario, 300, [&] { return scenario.drawsText("Base") && !scenario.drawsText("New waypoint"); }),
              "a waypoint added from the menu is drawn with its typed name");

        // the export rows, from a menu away from the markers near the centre
        openMenu(400.0f, 300.0f);
        scenario.clickText("Export this view");
        bool capture = (scenario.lastFrame().cursor_flags & 8) != 0
                    || runUntil(scenario, 10, [&] { return (scenario.lastFrame().cursor_flags & 8) != 0; });
        check(capture, "export this view asks the shim for a capture");
        std::vector<uint8_t> pixels(1280 * 720 * 4, 90);
        lm_view_captured(scenario.handle(), pixels.data(), 1280, 720, 1280 * 4, 87);
        check(runUntil(scenario, 1500, [&] { return scenario.drawsTextStarting("Exported "); }), "the captured view is exported");
        for (int i = 0; i < 400; ++i) scenario.frame(false, 640.0f, 360.0f);
        openMenu(400.0f, 300.0f);
        scenario.clickText("Export explored area");
        check(runUntil(scenario, 3000, [&] { return scenario.drawsTextStarting("Exported "); }), "the explored area is exported");
        int pngs = 0;
        for (const auto& entry : std::filesystem::directory_iterator(root / "data" / "exports")) {
            if (entry.path().extension() == ".png" && readText(entry.path()).rfind("\x89PNG", 0) == 0) ++pngs;
        }
        check(pngs == 2, "both exports are PNG files in the exports folder");

        // the panels are found by their text
        check(runUntil(scenario, 600, [&] { scenario.frame(false, 640.0f, 360.0f); return scenario.drawsText("Plains"); }),
              "hover names the biome under the cursor");
        check(scenario.drawsText("Chunk") && scenario.drawsText("0, 0"), "hover names the chunk under the cursor");
        uint32_t verticesBefore = scenario.vertexCount();
        check(scenario.clickText("Chunk grid") && scenario.vertexCount() > verticesBefore, "the chunk grid toggle draws grid lines");
        check(scenario.clickText("Waypoints") && scenario.drawsText("Search waypoints"), "the waypoint list opens");
        check(scenario.drawsText("Waypoint 1"), "the waypoint list shows the hotkey waypoint");
        check(scenario.clickText("Waypoint 1") && scenario.drawsText("X 40  Z 8"), "picking a list row centres the map on the waypoint");
        // the list's close button, the Segoe Fluent Icons cross
        check(scenario.clickText("\xEE\xA2\xBB") && !scenario.drawsText("Search waypoints"), "the close button closes the list");
        scenario.key(0x1B);

        // the player's feet are at y 80: band 5 of 16 blocks, layer 1055
        scenario.key(0x4B);
        check(runUntil(scenario, 200, [&] { return scenario.cameraHeight == 95; }), "cave mode scans the band around the feet from its top");
        check(runUntil(scenario, 200, [&] { return scenario.drawsText("CAVE \xC2\xB7 Y 80 to 95"); }), "the cave badge names the band");
        scenario.destroy();
        auto cave = root / "world" / "levi_minimap_cache";
        check(std::filesystem::exists(cave / "journal" / "1055") || std::filesystem::exists(cave / "regions" / "1055"),
              "the cave band is saved in its own layer folder");
        check(readText(root / "world" / "waypoints.json").find("\"name\": \"Waypoint 1\"") != std::string::npos,
              "the hotkey waypoint is saved in waypoints.json beside the world");
        check(readText(root / "world" / "waypoints.json").find("\"name\": \"Base\"") != std::string::npos,
              "the menu waypoint is saved in waypoints.json");
        check(scenario.releases > 0, "textures are released when worlds change");
        check(readText(root / "config" / "config.json").find("\"showChunkGrid\": true") != std::string::npos,
              "the chunk grid toggle is saved in config.json");
    }
    replayCaptures(root);
    std::filesystem::remove_all(root);
    std::printf("%s: %d failure(s)\n", g_failures == 0 ? "E2E PASSED" : "E2E FAILED", g_failures);
    return g_failures == 0 ? 0 : 1;
}
