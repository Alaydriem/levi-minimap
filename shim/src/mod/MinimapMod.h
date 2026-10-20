#pragma once

#include "ll/api/mod/NativeMod.h"

namespace levi_minimap {

// the LeviLamina entry point: starts Rust, installs hooks and input listeners
class MinimapMod {
public:
    static MinimapMod& getInstance();

    MinimapMod() : mSelf(*ll::mod::NativeMod::current()) {}

    [[nodiscard]] ll::mod::NativeMod& getSelf() const { return mSelf; }

    bool load();

    bool enable();

    bool disable();

private:
    ll::mod::NativeMod& mSelf;
};

} // namespace levi_minimap
