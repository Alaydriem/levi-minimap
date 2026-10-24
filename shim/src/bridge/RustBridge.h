#pragma once

#include "levi_minimap.h"

#include <filesystem>
#include <string>
#include <string_view>

namespace levi_minimap {

// owns the Rust handle; every call into Rust goes through handle()
class RustBridge {
public:
    // LM_STATUS_OK when the mod registered with the manager and the runtime started
    static LmStatus create(const std::filesystem::path& configDir, const std::filesystem::path& dataDir);

    static void destroy();

    // null before create and after destroy
    static const LmHandle* handle();

    // one line into the mod's log through Rust; level 1 error, 2 warn, 3 info, else debug
    static void log(uint8_t level, std::string_view message);

    // UTF-8 view of a string the caller keeps alive for the call
    static LmStr str(std::string_view text);

    static std::string utf8(const std::filesystem::path& path);
};

} // namespace levi_minimap
