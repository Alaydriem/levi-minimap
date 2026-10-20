#include "mod/MinimapMod.h"

#include "bridge/RustBridge.h"
#include "hooks/ClientUpdateHook.h"
#include "hooks/UIRenderHook.h"
#include "input/InputListeners.h"
#include "render/DX11Hook.h"
#include "world/WorldTracker.h"

#include "ll/api/mod/RegisterHelper.h"

namespace levi_minimap {

MinimapMod& MinimapMod::getInstance() {
    static MinimapMod instance;
    return instance;
}

bool MinimapMod::load() {
    auto& self = getSelf();
    auto  configFile = self.getConfigDir() / "config.json";
    auto  dataDir    = self.getDataDir();
    if (!RustBridge::create(configFile, dataDir)) {
        self.getLogger().error("levi-minimap could not start its Rust core");
        return false;
    }
    ClientUpdateHook::hook();
    UIRenderHook::hook();
    return true;
}

bool MinimapMod::enable() {
    InputListeners::registerListeners();
    return true;
}

bool MinimapMod::disable() {
    InputListeners::unregisterListeners();
    // unhook first so no game or render call can enter Rust, then stop Rust
    ClientUpdateHook::unhook();
    UIRenderHook::unhook();
    WorldTracker::detach();
    DX11Hook::shutdown();
    RustBridge::destroy();
    return true;
}

} // namespace levi_minimap

LL_REGISTER_MOD(levi_minimap::MinimapMod, levi_minimap::MinimapMod::getInstance());
