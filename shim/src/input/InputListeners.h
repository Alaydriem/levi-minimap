#pragma once

class ClientInstance;

namespace levi_minimap {

// forwards key and mouse events to Rust; cancels the ones Rust consumes and grabs or releases the mouse
class InputListeners {
public:
    static void registerListeners();
    static void unregisterListeners();

    // set on world entry, cleared on world exit
    static void setClientInstance(ClientInstance* client);
};

} // namespace levi_minimap
