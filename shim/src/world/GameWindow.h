#pragma once

#include <windows.h>

namespace levi_minimap {

// the game's window, found once and kept while it still exists
class GameWindow {
public:
    // null while the window cannot be found; it is looked for at most once a second
    static HWND hwnd();
};

} // namespace levi_minimap
