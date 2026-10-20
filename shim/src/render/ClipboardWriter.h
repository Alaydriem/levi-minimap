#pragma once

#include <windows.h>

#include <cstdint>

namespace levi_minimap {

// puts UTF-8 text from Rust on the Windows clipboard as Unicode text
class ClipboardWriter {
public:
    static void write(HWND hwnd, const uint8_t* utf8, uint32_t len);
};

} // namespace levi_minimap
