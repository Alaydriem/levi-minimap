#include "bridge/RustBridge.h"

#include <atomic>

namespace levi_minimap {

namespace {

std::atomic<LmHandle*> g_handle{nullptr};

} // namespace

LmStatus RustBridge::create(const std::filesystem::path& configDir, const std::filesystem::path& dataDir) {
    std::string config = utf8(configDir);
    std::string data   = utf8(dataDir);
    LmPaths     paths{str(config), str(data)};
    LmStatus    status = LM_STATUS_FAILED;
    LmHandle*   handle = lm_create(&paths, &status);
    g_handle.store(handle, std::memory_order_release);
    return handle ? LM_STATUS_OK : status;
}

void RustBridge::destroy() {
    LmHandle* handle = g_handle.exchange(nullptr, std::memory_order_acq_rel);
    if (handle) lm_destroy(handle);
}

const LmHandle* RustBridge::handle() { return g_handle.load(std::memory_order_acquire); }

void RustBridge::log(uint8_t level, std::string_view message) { lm_log(level, str(message)); }

LmStr RustBridge::str(std::string_view text) {
    return LmStr{reinterpret_cast<const uint8_t*>(text.data()), text.size()};
}

std::string RustBridge::utf8(const std::filesystem::path& path) {
    auto bytes = path.u8string();
    return std::string(reinterpret_cast<const char*>(bytes.data()), bytes.size());
}

} // namespace levi_minimap
