#include "world/BlockListener.h"

#include "bridge/RustBridge.h"
#include "world/ColumnProbe.h"

#include <mc/world/level/BlockPos.h>
#include <mc/world/level/BlockSource.h>
#include <mc/world/level/block/Block.h>
#include <mc/world/level/dimension/DimensionType.h>

namespace levi_minimap {

BlockListener& BlockListener::getInstance() {
    static BlockListener instance;
    return instance;
}

void BlockListener::onBlockChanged(
    ::BlockSource&    source,
    ::BlockPos const& pos,
    uint /*layer*/,
    ::Block const& block,
    ::Block const& oldBlock,
    int /*updateFlags*/,
    ::ActorBlockSyncMessage const* /*syncMsg*/,
    ::BlockChangedEventTarget /*eventTarget*/,
    ::Actor* /*blockChangeSource*/
) {
    if (block.getTypeName() == oldBlock.getTypeName()) return;
    const LmHandle* handle = RustBridge::handle();
    if (!handle) return;
    LmProbeRequest request{};
    int            dim = static_cast<int>(source.getDimensionId());
    if (lm_block_changed(handle, pos.x, pos.z, dim, &request) != LM_STATUS_OK || request.kind != 1) return;
    LmChunkPos     chunk{request.x, request.z, request.dim};
    LmColumnSample sample{};
    if (ColumnProbe::probeColumn(source, chunk, request.index, &sample)) {
        lm_submit_column(handle, &chunk, request.index, &sample);
    }
}

} // namespace levi_minimap
