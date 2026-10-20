#pragma once

#include "levi_minimap.h"

#include <filesystem>
#include <string>
#include <string_view>

namespace levi_minimap {

// owns the Rust handle and the log callback; every call into Rust goes through handle()
class RustBridge {
public:
    static bool create(const std::filesystem::path& configFile, const std::filesystem::path& dataDir);

    static void destroy();

    // null before create and after destroy
    static const LmHandle* handle();

    // UTF-8 view of a string the caller keeps alive for the call
    static LmStr str(std::string_view text);

    static std::string utf8(const std::filesystem::path& path);
};

} // namespace levi_minimap
