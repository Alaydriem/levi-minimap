#include "bridge/RustBridge.h"

#include "mod/MinimapMod.h"

#include <atomic>

namespace levi_minimap {

namespace {

std::atomic<LmHandle*> g_handle{nullptr};

void logFromRust(uint8_t level, const uint8_t* message, uintptr_t len) {
    std::string_view text(reinterpret_cast<const char*>(message), len);
    auto&            logger = MinimapMod::getInstance().getSelf().getLogger();
    switch (level) {
    case 1:
        logger.error("{}", text);
        break;
    case 2:
        logger.warn("{}", text);
        break;
    case 3:
        logger.info("{}", text);
        break;
    default:
        logger.debug("{}", text);
        break;
    }
}

} // namespace

bool RustBridge::create(const std::filesystem::path& configFile, const std::filesystem::path& dataDir) {
    std::string config = utf8(configFile);
    std::string data   = utf8(dataDir);
    LmPaths     paths{str(config), str(data)};
    LmHandle*   handle = lm_create(&paths, &logFromRust);
    g_handle.store(handle, std::memory_order_release);
    return handle != nullptr;
}

void RustBridge::destroy() {
    LmHandle* handle = g_handle.exchange(nullptr, std::memory_order_acq_rel);
    if (handle) lm_destroy(handle);
}

const LmHandle* RustBridge::handle() { return g_handle.load(std::memory_order_acquire); }

LmStr RustBridge::str(std::string_view text) {
    return LmStr{reinterpret_cast<const uint8_t*>(text.data()), text.size()};
}

std::string RustBridge::utf8(const std::filesystem::path& path) {
    auto bytes = path.u8string();
    return std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
}

} // namespace levi_minimap
