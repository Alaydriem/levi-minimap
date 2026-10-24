#include "mod/MinimapMod.h"

#include "bridge/RustBridge.h"
#include "hooks/ClientUpdateHook.h"
#include "world/WorldTracker.h"

#include "ll/api/mod/RegisterHelper.h"

namespace levi_minimap {

MinimapMod& MinimapMod::getInstance() {
    static MinimapMod instance;
    return instance;
}

// load() runs under the OS loader lock, so it only notes the folders; nothing starts a thread here
bool MinimapMod::load() {
    auto& self = getSelf();
    mConfigDir = std::filesystem::absolute(self.getConfigDir());
    mDataDir   = std::filesystem::absolute(self.getDataDir());
    return true;
}

bool MinimapMod::enable() {
    LmStatus status = RustBridge::create(mConfigDir, mDataDir);
    if (status != LM_STATUS_OK) {
        // without a registration there is no manager log, so this one line goes to the LeviLamina console
        if (status == LM_STATUS_MANAGER_MISSING) {
            getSelf().getLogger().error("alaydriem-mod-manager is not loaded, the minimap stays inert");
        } else if (status == LM_STATUS_REGISTER_FAILED) {
            getSelf().getLogger().error("the mod manager refused the minimap, the minimap stays inert");
        } else {
            getSelf().getLogger().error(
                "the minimap could not start (status {}), see the mod manager log; the minimap stays inert",
                static_cast<int>(status)
            );
        }
        return true;
    }
    mRegistered = true;
    ClientUpdateHook::hook();
    return true;
}

// the hook goes first, so no new tick enters Rust; then the session unregisters and the runtime stops
bool MinimapMod::disable() {
    if (!mRegistered) return true;
    ClientUpdateHook::unhook();
    WorldTracker::reset();
    RustBridge::destroy();
    mRegistered = false;
    return true;
}

} // namespace levi_minimap

LL_REGISTER_MOD(levi_minimap::MinimapMod, levi_minimap::MinimapMod::getInstance());
