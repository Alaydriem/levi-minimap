#include "render/ClipboardWriter.h"

namespace levi_minimap {

void ClipboardWriter::write(HWND hwnd, const uint8_t* utf8, uint32_t len) {
    if (!utf8 || len == 0) return;
    const char* text  = reinterpret_cast<const char*>(utf8);
    int         units = MultiByteToWideChar(CP_UTF8, 0, text, static_cast<int>(len), nullptr, 0);
    if (units <= 0) return;
    HGLOBAL memory = GlobalAlloc(GMEM_MOVEABLE, (static_cast<size_t>(units) + 1) * sizeof(wchar_t));
    if (!memory) return;
    auto* wide = static_cast<wchar_t*>(GlobalLock(memory));
    if (!wide) {
        GlobalFree(memory);
        return;
    }
    MultiByteToWideChar(CP_UTF8, 0, text, static_cast<int>(len), wide, units);
    wide[units] = L'\0';
    GlobalUnlock(memory);
    if (!OpenClipboard(hwnd)) {
        GlobalFree(memory);
        return;
    }
    EmptyClipboard();
    // the clipboard owns the memory once SetClipboardData succeeds
    if (!SetClipboardData(CF_UNICODETEXT, memory)) GlobalFree(memory);
    CloseClipboard();
}

} // namespace levi_minimap
