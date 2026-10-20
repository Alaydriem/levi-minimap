#pragma once

#include <mc/world/level/BlockSourceListener.h>

namespace levi_minimap {

// on a block type change, asks Rust whether one column needs probing and sends it
class BlockListener : public BlockSourceListener {
public:
    static BlockListener& getInstance();

    void onBlockChanged(
        ::BlockSource&                 source,
        ::BlockPos const&              pos,
        uint                           layer,
        ::Block const&                 block,
        ::Block const&                 oldBlock,
        int                            updateFlags,
        ::ActorBlockSyncMessage const* syncMsg,
        ::BlockChangedEventTarget      eventTarget,
        ::Actor*                       blockChangeSource
    ) override;
};

} // namespace levi_minimap
