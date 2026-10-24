#pragma once

#include "ll/api/mod/NativeMod.h"

#include <filesystem>

namespace levi_minimap {

// the LeviLamina entry point: registers with the mod manager on enable and installs the game hooks
class MinimapMod {
public:
    static MinimapMod& getInstance();

    MinimapMod() : mSelf(*ll::mod::NativeMod::current()) {}

    [[nodiscard]] ll::mod::NativeMod& getSelf() const { return mSelf; }

    bool load();

    bool enable();

    bool disable();

private:
    ll::mod::NativeMod&   mSelf;
    std::filesystem::path mConfigDir;
    std::filesystem::path mDataDir;
    bool                  mRegistered = false;
};

} // namespace levi_minimap
