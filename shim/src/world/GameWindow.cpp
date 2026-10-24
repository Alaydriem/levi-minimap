#include "world/GameWindow.h"

#include <cwchar>

namespace levi_minimap {

namespace {

HWND      g_hwnd       = nullptr;
ULONGLONG g_nextSearch = 0;

// the first top-level window of this process with the game's class; another game client has its own
BOOL CALLBACK findOwnWindow(HWND hwnd, LPARAM found) {
    DWORD process = 0;
    GetWindowThreadProcessId(hwnd, &process);
    if (process != GetCurrentProcessId()) return TRUE;
    wchar_t name[16]{};
    if (GetClassNameW(hwnd, name, 16) == 0 || wcscmp(name, L"Minecraft") != 0) return TRUE;
    *reinterpret_cast<HWND*>(found) = hwnd;
    return FALSE;
}

} // namespace

HWND GameWindow::hwnd() {
    constexpr ULONGLONG kSearchEveryMs = 1000;
    if (g_hwnd && IsWindow(g_hwnd)) return g_hwnd;
    g_hwnd         = nullptr;
    ULONGLONG now = GetTickCount64();
    if (now < g_nextSearch) return nullptr;
    g_nextSearch = now + kSearchEveryMs;
    EnumWindows(findOwnWindow, reinterpret_cast<LPARAM>(&g_hwnd));
    return g_hwnd;
}

} // namespace levi_minimap
