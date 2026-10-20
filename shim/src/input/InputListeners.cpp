#include "input/InputListeners.h"

#include "bridge/RustBridge.h"

#include <ll/api/event/EventBus.h>
#include <ll/api/event/input/KeyInputEvent.h>
#include <ll/api/event/input/MouseInputEvent.h>
#include <mc/client/game/ClientInstance.h>

#include <windows.h>

#include <atomic>

namespace levi_minimap {

namespace {

ll::event::ListenerPtr       g_keyListener;
ll::event::ListenerPtr       g_mouseListener;
std::atomic<ClientInstance*> g_client{nullptr};

void applyMouseAction(uint8_t action) {
    ClientInstance* client = g_client.load(std::memory_order_acquire);
    if (!client) return;
    if (action == 1) client->releaseMouse();
    if (action == 2) client->grabMouse();
}

} // namespace

void InputListeners::registerListeners() {
    auto& bus = ll::event::EventBus::getInstance();
    g_keyListener = bus.emplaceListener<ll::event::KeyInputEvent>([](ll::event::KeyInputEvent& event) {
        const LmHandle* handle = RustBridge::handle();
        if (!handle) return;
        ClientInstance* client  = g_client.load(std::memory_order_acquire);
        bool            ctrl    = (GetAsyncKeyState(VK_CONTROL) & 0x8000) != 0;
        bool            grabbed = client && client->getMouseGrabbed();
        LmKeyResult     result{};
        if (lm_key(handle, event.keyCode(), event.isDown() ? 1 : 0, ctrl ? 1 : 0, grabbed ? 1 : 0, &result) != LM_STATUS_OK) return;
        if (result.consume) event.cancel();
        applyMouseAction(result.action);
    });
    g_mouseListener = bus.emplaceListener<ll::event::MouseInputEvent>([](ll::event::MouseInputEvent& event) {
        const LmHandle* handle = RustBridge::handle();
        if (!handle) return;
        int button = static_cast<int>(event.actionButtonId());
        int data   = static_cast<int>(static_cast<signed char>(event.buttonData()));
        if (lm_mouse(handle, button, data)) event.cancel();
    });
}

void InputListeners::unregisterListeners() {
    auto& bus = ll::event::EventBus::getInstance();
    if (g_keyListener) {
        bus.removeListener<ll::event::KeyInputEvent>(g_keyListener);
        g_keyListener = nullptr;
    }
    if (g_mouseListener) {
        bus.removeListener<ll::event::MouseInputEvent>(g_mouseListener);
        g_mouseListener = nullptr;
    }
}

void InputListeners::setClientInstance(ClientInstance* client) { g_client.store(client, std::memory_order_release); }

} // namespace levi_minimap
