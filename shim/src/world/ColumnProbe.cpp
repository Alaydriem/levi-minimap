#include "world/ColumnProbe.h"

#include "bridge/RustBridge.h"

#include <mc/world/level/BlockPos.h>
#include <mc/world/level/BlockSource.h>
#include <mc/world/level/ChunkBlockPos.h>
#include <mc/world/level/ChunkLocalHeight.h>
#include <mc/world/level/biome/Biome.h>
#include <mc/world/level/biome/biome_color_sampling/BiomeColorSampling.h>
#include <mc/world/level/biome/biome_color_sampling/MapPolicy.h>
#include <mc/world/level/block/Block.h>
#include <mc/world/level/block/BlockType.h>
#include <mc/world/level/block/components/BlockMapColorComponent.h>
#include <mc/world/level/chunk/ChunkState.h>
#include <mc/world/level/chunk/LevelChunk.h>
#include <mc/world/level/chunk/SubChunk.h>
#include <mc/world/level/dimension/Dimension.h>
#include <mc/world/level/material/Material.h>

#include <algorithm>
#include <cstring>
#include <atomic>
#include <string>

namespace levi_minimap {

namespace {

constexpr const char* kPlaceholder = "minecraft:client_request_placeholder_block";

std::atomic<int>  g_cameraHeight{320};
std::atomic<bool> g_transparentWater{true};

// game thread only, like every probe call
const LmBlockColor* g_blockColours     = nullptr;
uint32_t            g_blockColourCount = 0;

struct Colour {
    uint8_t r{0};
    uint8_t g{0};
    uint8_t b{0};
    uint8_t a{0};
};

// one column's walk result: the colour shown, and the water above it when transparent water applies
struct Column {
    Colour  colour;
    Colour  waterColour;
    uint8_t waterDepth{0};
    bool    retry{false};
};

uint32_t pack(Colour c) {
    return static_cast<uint32_t>(c.r) | (static_cast<uint32_t>(c.g) << 8) | (static_cast<uint32_t>(c.b) << 16)
         | (static_cast<uint32_t>(c.a) << 24);
}

Colour fromArgb(int argb) {
    return Colour{
        static_cast<uint8_t>((argb >> 16) & 0xFF),
        static_cast<uint8_t>((argb >> 8) & 0xFF),
        static_cast<uint8_t>(argb & 0xFF),
        255
    };
}

Colour fromMce(mce::Color const& c) {
    auto channel = [](float v) { return static_cast<uint8_t>(std::clamp(v, 0.0f, 1.0f) * 255.0f + 0.5f); };
    return Colour{channel(c.r), channel(c.g), channel(c.b), channel(c.a)};
}

// the colour the scan plan names for this block, if any
bool overrideColour(std::string const& name, Colour& out) {
    for (uint32_t i = 0; i < g_blockColourCount; ++i) {
        const LmBlockColor& entry = g_blockColours[i];
        if (name.size() == entry.name.len && std::memcmp(name.data(), entry.name.ptr, entry.name.len) == 0) {
            out = Colour{
                static_cast<uint8_t>(entry.colour & 0xFF),
                static_cast<uint8_t>((entry.colour >> 8) & 0xFF),
                static_cast<uint8_t>((entry.colour >> 16) & 0xFF),
                static_cast<uint8_t>((entry.colour >> 24) & 0xFF)
            };
            return true;
        }
    }
    return false;
}

bool isAir(std::string const& name) { return name == "minecraft:air" || name == "air"; }

class Walker {
public:
    Walker(LevelChunk& chunk, int x, int z) : mChunk(chunk), mX(x), mZ(z), mMinY(chunk.mMin->y), mMaxY(chunk.mMax->y) {}

    Column walk(int camera, bool transparentWater) {
        mTransparentWater = transparentWater;
        Column column;
        column.colour = pick(camera, column);
        column.retry  = mRetry;
        return column;
    }

private:
    Block const& blockAt(int y) const {
        return mChunk.getBlock(ChunkBlockPos(
            static_cast<uchar>(mX),
            ChunkLocalHeight{static_cast<short>(y - mMinY)},
            static_cast<uchar>(mZ)
        ));
    }

    std::string nameAt(int y) {
        std::string name = blockAt(y).getTypeName();
        if (name == kPlaceholder) mRetry = true;
        return name;
    }

    BlockPos worldPos(int y) const {
        return BlockPos(mChunk.mPosition->x * 16 + mX, y, mChunk.mPosition->z * 16 + mZ);
    }

    // vanilla map colour: the map colour component tinted by its biome policy, else the block type's colour
    Colour mapColour(int y) {
        Block const& block = blockAt(y);
        std::string const& name = block.getTypeName();
        if (name == kPlaceholder) return Colour{};
        if (Colour chosen; overrideColour(name, chosen)) return chosen;
        BlockSource& source = mChunk.mDimension.getBlockSourceFromMainChunkSource();
        BlockPos     pos    = worldPos(y);
        if (name == "minecraft:water") {
            ChunkBlockPos local(static_cast<uchar>(mX), ChunkLocalHeight{static_cast<short>(y - mMinY)}, static_cast<uchar>(mZ));
            return fromArgb(BiomeColorSampling::getWaterColor(mChunk.getBiome(local), pos));
        }
        if (auto const* comp = block.tryGetComponent<BlockMapColorComponent>()) {
            auto const& policy = BiomeColorSampling::getMapPolicy(comp->mTintMethod);
            mce::Color  tint   = policy.get(source, pos);
            mce::Color  base   = comp->mMapColor.get();
            return fromMce(mce::Color(base.r * tint.r, base.g * tint.g, base.b * tint.b, base.a));
        }
        return fromMce(block.getBlockType().getMapColor(source, pos, block));
    }

    // a column with no blocks: the biome's terrain.biomeColorOverrides entry, else the game's map grass colour
    Colour voidColour() {
        ChunkBlockPos local(static_cast<uchar>(mX), ChunkLocalHeight{0}, static_cast<uchar>(mZ));
        Biome const&  biome    = mChunk.getBiome(local);
        Colour        fallback = fromArgb(BiomeColorSampling::getMapGrassColor(biome, worldPos(mMinY)));
        std::string   name     = biome.mHash->getString();
        uint32_t      packed   = lm_resolve_void_color(RustBridge::handle(), RustBridge::str(name), pack(fallback));
        return Colour{
            static_cast<uint8_t>(packed & 0xFF),
            static_cast<uint8_t>((packed >> 8) & 0xFF),
            static_cast<uint8_t>((packed >> 16) & 0xFF),
            static_cast<uint8_t>((packed >> 24) & 0xFF)
        };
    }

    // transparent water: the solid colour below goes in the result, the water colour and depth beside it
    Colour water(int surfaceY, Column& column) {
        column.waterColour = mapColour(surfaceY);
        for (int y = surfaceY - 1; y >= mMinY; --y) {
            std::string name = nameAt(y);
            if (name != "minecraft:water" && !isAir(name)) {
                column.waterDepth = static_cast<uint8_t>(std::min(surfaceY - y, 255));
                return mapColour(y);
            }
        }
        column.waterDepth = static_cast<uint8_t>(std::min(surfaceY - mMinY, 255));
        return column.waterColour;
    }

    Colour show(int y, std::string const& name, Column& column) {
        if (mTransparentWater && name == "minecraft:water") return water(y, column);
        return mapColour(y);
    }

    // the block below the first air at or above `from`, up to `to`; the block at `to` when none is air
    Colour firstBelowAir(int from, int to, Column& column) {
        for (int y = from; y <= to; ++y) {
            if (!isAir(nameAt(y))) continue;
            if (y <= mMinY) return voidColour();
            return show(y - 1, nameAt(y - 1), column);
        }
        return show(to, nameAt(to), column);
    }

    Colour pick(int camera, Column& column) {
        int height = (*mChunk.mHeightmap)[16 * mZ + mX].mVal + mMinY;
        if (camera < mMinY) return Colour{};
        if (camera > mMaxY) {
            int from = (height < mMinY || height > mMaxY) ? mMaxY : height;
            return firstBelowAir(from, mMaxY, column);
        }
        std::string cameraName = nameAt(camera);
        if (isAir(cameraName)) {
            if (height < camera) return firstBelowAir(height, camera, column);
            for (int y = camera; y >= mMinY; --y) {
                std::string name = nameAt(y);
                if (!isAir(name)) return show(y, name, column);
            }
            return voidColour();
        }
        if (!blockAt(camera).getMaterial().mSolid) return show(camera, cameraName, column);
        // under a solid run: the first non-solid block, or the first block below the first air gap
        for (int y = camera - 1; y >= mMinY; --y) {
            std::string name = nameAt(y);
            if (isAir(name)) {
                for (int below = y - 1; below >= mMinY; --below) {
                    std::string belowName = nameAt(below);
                    if (!isAir(belowName)) return show(below, belowName, column);
                }
                return voidColour();
            }
            if (!blockAt(y).getMaterial().mSolid) return show(y, name, column);
        }
        return Colour{};
    }

    LevelChunk& mChunk;
    int         mX;
    int         mZ;
    int         mMinY;
    int         mMaxY;
    bool        mTransparentWater{true};
    bool        mRetry{false};
};

LevelChunk* loadedChunk(BlockSource& region, const LmChunkPos& pos) {
    LevelChunk* chunk = region.getChunk(pos.x, pos.z);
    if (!chunk || chunk->mLoadState.get() != ChunkState::Loaded) return nullptr;
    return chunk;
}

// the biome at the column's surface block; the heightmap names the first block above it
Biome const& surfaceBiomeAt(LevelChunk& chunk, int x, int z, int index) {
    int top   = chunk.mMax->y - chunk.mMin->y;
    int local = std::clamp(static_cast<int>(chunk.mHeightmap.get()[index].mVal) - 1, 0, top);
    ChunkBlockPos at(static_cast<uchar>(x), ChunkLocalHeight{static_cast<short>(local)}, static_cast<uchar>(z));
    return chunk.getBiome(at);
}

uint16_t surfaceBiome(LevelChunk& chunk, int x, int z, int index) {
    return surfaceBiomeAt(chunk, x, z, index).mId->mValue;
}

// the column's surface sub-chunk is still a placeholder: the client has not received it yet
bool surfacePending(LevelChunk& chunk, int index) {
    auto const& subChunks = chunk.mSubChunks.get();
    if (subChunks.empty()) return true;
    int height = chunk.mHeightmap.get()[index].mVal;
    int sub    = std::min(height >> 4, static_cast<int>(subChunks.size()) - 1);
    return sub >= 0 && subChunks[sub].isPlaceHolderSubChunk();
}

} // namespace

void ColumnProbe::configure(int cameraHeight, bool transparentWater, const LmBlockColor* colours, uint32_t colourCount) {
    g_blockColours     = colours;
    g_blockColourCount = colours ? colourCount : 0;
    g_cameraHeight.store(cameraHeight, std::memory_order_relaxed);
    g_transparentWater.store(transparentWater, std::memory_order_relaxed);
}

uint8_t ColumnProbe::probeChunk(BlockSource& region, const LmChunkPos& pos, LmChunkSurface* out) {
    LevelChunk* chunk = loadedChunk(region, pos);
    if (!chunk) return NOT_LOADED;
    auto const& subChunks = chunk->mSubChunks.get();
    if (subChunks.empty() || subChunks[subChunks.size() - 1].isPlaceHolderSubChunk()) return TOP_PLACEHOLDER;
    if (!out) return NOT_PROBED;
    int  minY   = chunk->mMin->y;
    int  camera = g_cameraHeight.load(std::memory_order_relaxed);
    bool water  = g_transparentWater.load(std::memory_order_relaxed);
    for (int z = 0; z < 16; ++z) {
        for (int x = 0; x < 16; ++x) {
            int index = z * 16 + x;
            if (surfacePending(*chunk, index)) {
                out->skipped[index / 64] |= uint64_t{1} << (index % 64);
                continue;
            }
            Column column       = Walker(*chunk, x, z).walk(camera, water);
            Colour waterRgb     = column.waterColour;
            out->surface[index] = pack(column.colour);
            out->water[index]   = column.waterDepth
                                    ? (pack(Colour{waterRgb.r, waterRgb.g, waterRgb.b, 0}) | (uint32_t{column.waterDepth} << 24))
                                    : 0;
            out->height[index]  = static_cast<int16_t>(chunk->mHeightmap.get()[index].mVal + minY);
            out->solid[index]   = static_cast<int16_t>(chunk->mRenderHeightmap.get()[index].mVal + minY);
            if (column.retry) out->retry[index / 64] |= uint64_t{1} << (index % 64);
            out->biome[index] = surfaceBiome(*chunk, x, z, index);
        }
    }
    return LOADED;
}

std::string ColumnProbe::biomeName(BlockSource& region, const LmChunkPos& pos, uint32_t index) {
    LevelChunk* chunk = loadedChunk(region, pos);
    if (!chunk || index >= 256) return {};
    int x = static_cast<int>(index % 16);
    int z = static_cast<int>(index / 16);
    return surfaceBiomeAt(*chunk, x, z, static_cast<int>(index)).mHash->getString();
}

bool ColumnProbe::probeColumn(BlockSource& region, const LmChunkPos& pos, uint32_t index, LmColumnSample* out) {
    LevelChunk* chunk = loadedChunk(region, pos);
    if (!chunk || !out || index >= 256) return false;
    int    x      = static_cast<int>(index % 16);
    int    z      = static_cast<int>(index / 16);
    int    minY   = chunk->mMin->y;
    Column column = Walker(*chunk, x, z).walk(
        g_cameraHeight.load(std::memory_order_relaxed),
        g_transparentWater.load(std::memory_order_relaxed)
    );
    Colour waterRgb = column.waterColour;
    out->surface    = pack(column.colour);
    out->water      = column.waterDepth ? (pack(Colour{waterRgb.r, waterRgb.g, waterRgb.b, 0}) | (uint32_t{column.waterDepth} << 24)) : 0;
    out->height     = static_cast<int16_t>(chunk->mHeightmap.get()[index].mVal + minY);
    out->solid      = static_cast<int16_t>(chunk->mRenderHeightmap.get()[index].mVal + minY);
    out->retry      = column.retry ? 1 : 0;
    out->biome      = surfaceBiome(*chunk, x, z, static_cast<int>(index));
    return true;
}

} // namespace levi_minimap
