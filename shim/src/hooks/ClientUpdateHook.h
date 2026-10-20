#pragma once

#include "world/WorldTracker.h"

#include <ll/api/memory/Hook.h>
#include <mc/client/game/ClientInstance.h>

namespace levi_minimap {

// once per client tick: world lifecycle, the Rust tick, probing the scan plan
LL_TYPE_INSTANCE_HOOK(
    ClientUpdateHook,
    ll::memory::HookPriority::Normal,
    ClientInstance,
    &ClientInstance::$update,
    bool,
    bool a1
) {
    bool result = origin(a1);
    WorldTracker::update(*this);
    return result;
}

} // namespace levi_minimap
