#pragma once

#include "render/DX11Hook.h"

#include <ll/api/memory/Hook.h>
#include <mc/client/renderer/screen/MinecraftUIRenderContext.h>

#include <mutex>

namespace levi_minimap {

inline std::once_flag g_dx11InitOnce;

// the first UI text flush means the swap chain exists: install the DX11 hooks once
LL_TYPE_INSTANCE_HOOK(
    UIRenderHook,
    ll::memory::HookPriority::Normal,
    MinecraftUIRenderContext,
    &MinecraftUIRenderContext::$flushText,
    void,
    float                deltaTime,
    std::optional<float> obfuscateSwitchTime
) {
    origin(deltaTime, obfuscateSwitchTime);
    std::call_once(g_dx11InitOnce, [] { DX11Hook::init(); });
}

} // namespace levi_minimap
