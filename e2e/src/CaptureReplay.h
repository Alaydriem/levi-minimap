#pragma once

#include "Scenario.h"

#include <cstdint>
#include <filesystem>

namespace lm_e2e {

struct ReplayResult {
    bool     ok           = false;
    bool     withinBudget = true;
    uint64_t events       = 0;
    uint64_t frames       = 0;
    uint64_t maxBakes     = 0;
};

// replays an unpacked (.lmraw) capture: every recorded call goes to the same lm_* function the shim
// would call, with the recorded arguments. The record layout is CaptureCodec's in minimap-runtime
class CaptureReplay {
public:
    static ReplayResult run(Scenario& scenario, const std::filesystem::path& path);
};

} // namespace lm_e2e
