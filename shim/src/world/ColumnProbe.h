#pragma once

#include "levi_minimap.h"

#include <cstdint>
#include <string>

class BlockSource;

namespace levi_minimap {

// reads loaded game chunks into the surface layout Rust consumes; holds no state beyond the plan's settings
class ColumnProbe {
public:
    static constexpr uint8_t LOADED          = 0;
    static constexpr uint8_t NOT_LOADED      = 1;
    static constexpr uint8_t TOP_PLACEHOLDER = 2;
    static constexpr uint8_t NOT_PROBED      = 3;

    // camera height, transparent water and block colours come from the latest scan plan; `colours` stays
    // valid until the next lm_tick
    static void configure(int cameraHeight, bool transparentWater, const LmBlockColor* colours, uint32_t colourCount);

    // fills `out` (a zeroed staging surface) and returns one of the status codes above
    static uint8_t probeChunk(BlockSource& region, const LmChunkPos& pos, LmChunkSurface* out);

    // false when the chunk is not loaded
    static bool probeColumn(BlockSource& region, const LmChunkPos& pos, uint32_t index, LmColumnSample* out);

    // the game's internal name of the biome at the column's surface; empty when the chunk is not loaded.
    // The client's biome registry does not know every ID, but the chunk's own biome carries its name
    static std::string biomeName(BlockSource& region, const LmChunkPos& pos, uint32_t index);
};

} // namespace levi_minimap
