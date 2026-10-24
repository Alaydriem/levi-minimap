#pragma once

class ClientInstance;

namespace levi_minimap {

// follows the player in and out of worlds and dimensions, ticks Rust and probes its scan plan
class WorldTracker {
public:
    static void update(ClientInstance& client);

    // the cursor action of the latest tick also runs on the game thread
    static void applyCursor(unsigned char action);

    // stops listening for block changes
    static void detach();

    // forgets the world, so a runtime made by the next enable() enters it again; called when the mod is disabled
    static void reset();
};

} // namespace levi_minimap
