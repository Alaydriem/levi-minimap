<div align="center">

# Alaydriem's Mini Map

**A minimap and full world map for Minecraft Bedrock on Windows, loaded by LeviLamina.**

[![Release](https://img.shields.io/github/v/release/alaydriem/levi-minimap?include_prereleases)](https://github.com/alaydriem/levi-minimap/releases)
[![Build](https://img.shields.io/github/actions/workflow/status/alaydriem/levi-minimap/build.yml?branch=master&label=build)](https://github.com/alaydriem/levi-minimap/actions/workflows/build.yml)
[![License](https://img.shields.io/github/license/alaydriem/levi-minimap)](./LICENSE)
![Minecraft Bedrock](https://img.shields.io/badge/Minecraft_Bedrock-1.26.51-62B47A)
[![LeviLamina](https://img.shields.io/badge/LeviLamina-26.51.6-4A90D9)](https://github.com/LiteLDev/LeviLamina)
![Windows](https://img.shields.io/badge/Windows-x64-0078D6)

[Install](#-installation) · [Controls](#%EF%B8%8F-controls) · [Configuration](#%EF%B8%8F-configuration) · [FAQ](#-faq) · [Build](#%EF%B8%8F-building-from-source)

</div>

Alaydriem's Mini Map draws a minimap in the corner of your screen and a full-screen world map of everything you have explored. It keeps waypoints for each world, shows the way to your last death, has a cave mode for underground, and exports the map to PNG.

> [!NOTE]
> This is a **client-side** mod. It needs no server plugin, and it reads only the chunks that the game has already loaded on your machine.

---

## ✨ Features

- **🧭 Minimap**
  - Round minimap with a chunk grid, the current chunk outlined, and your position and heading.
  - Coordinates under the map, with the matching Nether or Overworld coordinates (1:8).
  - Zoom with keys. Optional mode that turns the map with the player.
- **🗺️ World map**
  - Full-screen map of every area you have explored, in each dimension.
  - Zoom with the mouse wheel, pan with a left-drag.
  - A tools panel shows the block, chunk and biome under the cursor.
  - Optional chunk grid and slime chunk overlays, drawn only over chunks you have explored.
  - Right-click menu: add, edit or remove a waypoint, copy coordinates, show or hide waypoints, export the view or the whole explored area.
- **📍 Waypoints**
  - Add a waypoint at your position with one key, or anywhere on the world map.
  - A waypoint list with search and the distance to each waypoint. Click a waypoint to centre the map on it.
  - 12 colours to choose from.
  - A death point for each world, kept even when the waypoint list is full.
  - Arrows at the minimap edge point to waypoints out of view.
  - Saved for each world, so they come back when you load the world again.
- **⛏️ Cave mode**
  - Shows a band of blocks around your feet, so you can map caves and tunnels.
- **🎨 Terrain shading**
  - Hill shading with three styles, adjustable light direction, and water that gets darker with depth.
  - Your own colour for any block or biome, set in the config file.
- **🖼️ Export**
  - Save the current view, or the whole explored area, as a PNG image.
- **💾 Disk cache**
  - Explored areas are saved to disk, so the map is ready the next time you play.
  - Memory use has a fixed budget that you can set in the config file.

<table>
  <tr>
    <td align="center" width="50%"><img src="./assets/minimap-overworld.jpg" alt="The minimap in the top-right corner while exploring" width="420"><br><sub>The minimap while exploring</sub></td>
    <td align="center" width="50%"><img src="./assets/world-map-tools.jpg" alt="The world map with the tools panel, the chunk grid and a waypoint" width="420"><br><sub>World map with the tools panel</sub></td>
  </tr>
  <tr>
    <td align="center" width="50%"><img src="./assets/export-overworld.jpg" alt="An exported view of the Overworld" width="420"><br><sub>Overworld, exported to PNG</sub></td>
    <td align="center" width="50%"><img src="./assets/export-overworld-chunk-grid.jpg" alt="An exported view of the Overworld with the chunk grid" width="420"><br><sub>Chunk grid over explored chunks</sub></td>
  </tr>
  <tr>
    <td align="center" width="50%"><img src="./assets/export-nether.jpg" alt="An exported view of the Nether" width="420"><br><sub>The Nether</sub></td>
    <td align="center" width="50%"><img src="./assets/export-end.jpg" alt="An exported view of the End island" width="420"><br><sub>The End</sub></td>
  </tr>
</table>

---

## 📦 Installation

You need [LeviLamina](https://github.com/LiteLDev/LeviLamina) for the **client**, on a supported game version (see [Compatibility](#-compatibility)).

Install with [lip](https://github.com/LiteLDev/lip). Run this command in your game folder:

```powershell
lip install github.com/alaydriem/levi-minimap#client
```

To remove the mod:

```powershell
lip uninstall github.com/alaydriem/levi-minimap
```

<details>
<summary><b>Manual install</b></summary>

1. Download `alaydriem-minimap-<version>-client-windows-x64.zip` from the [latest release](https://github.com/alaydriem/levi-minimap/releases/latest).
2. Extract it into your game's `mods` folder. The result must be `mods/alaydriem-minimap/alaydriem-minimap.dll`.
3. Start the game.

The folder name must stay `alaydriem-minimap`. LeviLamina does not load the mod from a folder with a different name.

</details>

---

## ✅ Compatibility

| Mod version | Minecraft Bedrock (Windows) | LeviLamina |
| --- | --- | --- |
| 0.1.0 | 1.26.51 | 26.51.6 or later 26.51.x |

The mod hooks into the game directly. A new game version needs a new mod release.

---

## ⌨️ Controls

| Key | Action | When |
| --- | --- | --- |
| <kbd>M</kbd> | Open or close the world map | In game |
| <kbd>Ctrl</kbd> + <kbd>M</kbd> | Show or hide the minimap | In game, or world map open |
| <kbd>=</kbd> / <kbd>-</kbd> | Zoom the minimap in / out | In game |
| <kbd>N</kbd> | Add a waypoint at your position | In game |
| <kbd>K</kbd> | Cave mode on / off | In game, or world map open |
| <kbd>Ctrl</kbd> + <kbd>K</kbd> | Show or hide coordinates | In game, or world map open |
| <kbd>Esc</kbd> | Close the world map, menu or dialog | World map open |
| Mouse wheel | Zoom the world map | World map open |
| Left-drag | Pan the world map | World map open |
| Right-click | Open the map menu | World map open |
| <kbd>Ctrl</kbd> + <kbd>Z</kbd> | Bring back the waypoint you just removed | A few seconds after a removal |
| <kbd>Enter</kbd> / <kbd>Esc</kbd> | Save / cancel the waypoint dialog | Dialog open |
| Click a waypoint in the list | Centre the world map on it | Waypoint list open |

> [!TIP]
> Every key is a setting in `config.json`, written as a [Windows virtual-key code](https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes). For example, `77` (`0x4D`) is <kbd>M</kbd>.

---

## ⚙️ Configuration

The config file is `mods/alaydriem-minimap/config/config.json`. The mod writes it with the default values the first time it starts. A key that you remove from the file uses its default. A value out of range is changed to the nearest allowed value, and the game log shows a warning.

The settings that most players change:

| Setting | Default | What it does |
| --- | --- | --- |
| `miniMap.radiusChunks` | `5` | How many chunks the minimap shows from its centre |
| `miniMap.sizeRatio` | `0.15` | Minimap size, as a fraction of the screen width |
| `miniMap.rotateWithPlayer` | `false` | Turn the minimap with the player |
| `miniMap.showNetherCoordinates` | `true` | Also show the Nether or Overworld coordinates |
| `worldMap.rotateWithPlayer` | `false` | Turn the world map with the player |
| `worldMap.showChunkGrid` / `showSlimeChunks` | `false` | Start with the chunk grid or the slime chunks shown |
| `waypoints.maxPerWorld` | `512` | Maximum waypoints in one world |
| `terrain.shadow.renderStyle` | `2` | Shading style: `0`, `1` or `2` |
| `terrain.enableTransparentWater` | `true` | Show the ground under shallow water |
| `terrain.blockColors` | lava only | Your own colour for a block, as `[r, g, b, a]` |
| `terrain.biomeColorOverrides` | the End only | Your own colour for a biome where a column has no blocks, as `[r, g, b, a]` |
| `export.folder` | empty | Export folder. Empty means `Pictures/Screenshots` |

<details>
<summary><b>Full reference</b></summary>

Colours in `miniMap` are numbers in the form `0xRRGGBBAA`. Colours in `terrain` are `[r, g, b, a]` arrays. Keys are Windows virtual-key codes.

| Section | Setting | Default |
| --- | --- | --- |
| top level | `fontSize` | `18.0` |
| `miniMap` | `sizeRatio` | `0.15` |
| | `radiusChunks` / `minRadiusChunks` / `maxRadiusChunks` | `5` / `2` / `16` |
| | `marginRight` / `marginTop` | `20` / `20` |
| | `widgetScale` | `1.0` |
| | `lineThickness` / `borderThickness` / `coordTextMargin` | `1.0` / `2.0` / `8.0` |
| | `backgroundColor` / `chunkLineColor` / `currentChunkColor` / `borderColor` | `0x000000FF` / `0x787878FF` / `0x3AA0FFFF` / `0x505050FF` |
| | `playerArrowOutlineColor` / `playerArrowFillColor` | `0x000000FF` / `0xDC3232FF` |
| | `coordTextColor` / `coordShadowColor` | `0xFFFFFFFF` / `0x000000C8` |
| | `zoomInKey` / `zoomOutKey` | `0xBB` (<kbd>=</kbd>) / `0xBD` (<kbd>-</kbd>) |
| | `showCoordinates` / `coordinatesKey` | `true` / `0x4B` (<kbd>Ctrl</kbd> + <kbd>K</kbd>) |
| | `showNetherCoordinates` / `rotateWithPlayer` | `true` / `false` |
| `worldMap` | `toggleKey` | `0x4D` (<kbd>M</kbd>) |
| | `minZoom` / `maxZoom` / `zoomStep` | `0.2` / `40.0` / `1.15` |
| | `showCoordinates` / `showNetherCoordinates` / `rotateWithPlayer` | `true` / `true` / `false` |
| | `waypointLabelZoom` | `0.5` |
| | `showChunkGrid` / `showSlimeChunks` | `false` / `false` |
| | `hoverBiomeRegions` / `slimeCacheRegions` / `uiEventCapacity` | `16` / `1024` / `64` |
| | `maxNewTexturesPerFrame` / `textureIdleFrames` | `4` / `600` |
| `waypoints` | `enabled` / `addKey` | `true` / `0x4E` (<kbd>N</kbd>) |
| | `maxPerWorld` / `showOnMinimap` / `maxEdgeArrows` / `labelScale` | `512` / `true` / `16` / `1.0` |
| `caveMode` | `toggleKey` | `0x4B` (<kbd>K</kbd>) |
| | `bandSize` / `bandMargin` / `scanRadius` / `solidColor` | `16` / `2` / `64` / `[96, 96, 96, 255]` |
| `terrain` | `enable` / `scanRadius` / `scanMaxChunksPerFrame` | `true` / `128` / `32` |
| | `rescanIntervalFrames` / `regionBakeThreshold` | `1200` / `64` |
| | `enableDiskCache` / `enableTransparentWater` | `true` / `true` |
| | `waterOpacityPerDepth` / `waterMaxOpacity` | `0.1` / `0.6` |
| | `blockColors` (max 64) | `{ "minecraft:lava": [230, 110, 20, 255] }` |
| | `biomeColorOverrides` (max 64) | `{ "the_end": [16, 12, 26, 255] }` |
| `terrain.shadow` | `renderStyle` / `shadowLevel` / `renderScale` / `pcfRadius` | `2` / `130` / `2` / `1` |
| | `lightAzimuth` / `lightZenith` | `315.0` / `60.0` |
| `smoothCamera` | `springStrength` / `damping` / `maxDeltaTime` | `36.0` / `7.0` / `0.1` |
| `memory` | `chunkSurfaceBudgetMb` / `tileBudgetMb` / `tileOverrunPercent` / `gpuTextureBudget` | `64` / `256` / `25` / `1024` |
| `bake` | `workerThreads` (`0` = automatic) / `queueCapacity` | `0` / `4096` |
| `store` | `journalMaxMb` / `syncIntervalMs` / `writeQueueCapacity` | `64` / `1000` / `1024` |
| `export` | `maxPixels` / `folder` | `16384` / `""` |
| `debug` | `telemetry` / `telemetryIntervalMs` / `captureProbes` | `false` / `1000` / `false` |

The default override draws the End's void near black with a purple tint. A biome name in `biomeColorOverrides` works with or without the `minecraft:` prefix:

```json
{
  "terrain": {
    "biomeColorOverrides": { "cherry_grove": [242, 184, 216, 255] }
  }
}
```

</details>

---

## ❓ FAQ

<details>
<summary><b>The mod does not load.</b></summary>

Check that LeviLamina is the **client** build for your game version (see [Compatibility](#-compatibility)), and that the mod folder is named exactly `mods/alaydriem-minimap/`. The LeviLamina log in the game folder's `logs` folder names the cause.

</details>

<details>
<summary><b>Part of the world map is empty.</b></summary>

The map shows only chunks that your game has loaded. Visit the area once, and it stays on the map, also after a restart.

</details>

<details>
<summary><b>Can I use it on servers and Realms?</b></summary>

The mod is client-side and works wherever LeviLamina runs. Some servers do not allow minimaps. Read the server's rules before you use it there.

</details>

<details>
<summary><b>The biome under the cursor shows a dash.</b></summary>

The mod learns each biome when it scans a chunk. Land you explored with an older version of the mod has no biome saved. Go back to it, and the mod fills in the biome when it scans the chunks again.

</details>

<details>
<summary><b>Where are my waypoints and the map cache?</b></summary>

In `mods/alaydriem-minimap/data/`. Waypoints are kept for each world.

</details>

<details>
<summary><b>A key does nothing.</b></summary>

Most keys do nothing while a game menu or chat is open. If another mod uses the same key, change the key in `config.json` (see [Controls](#%EF%B8%8F-controls)).

</details>

---

## 🛠️ Building from source

You need Windows 10 or 11 (x64) and [mise](https://mise.jdx.dev). mise installs the pinned versions of xmake and cbindgen. You also need Rust through rustup (the version is pinned in `rust-toolchain.toml`), [cargo-nextest](https://nexte.st), and LLVM with `clang-cl`.

```powershell
git clone https://github.com/alaydriem/levi-minimap
cd levi-minimap
mise trust
mise run preflight   # names any missing tool
mise run build-mod   # output: shim/bin/alaydriem-minimap/
mise run test        # every Rust test
```

<details>
<summary><b>How the code is organised</b></summary>

Nearly all the logic is in Rust: `minimap/core` (map logic, no game or GPU code), `minimap/store` (disk cache), `minimap/ui` (the interface, built with egui), `minimap/runtime`, and `minimap/ffi` (the C interface). A thin C++ part in `shim/` connects to the game through LeviLamina and draws the interface with Direct3D 11. Run `mise tasks` to see every build, test and bench task.

</details>

---

## 🔗 Related

- [Alaydriem's Freecam](https://github.com/alaydriem/levi-freecam): a free camera with smooth motion and camera paths, for the same game and loader.

## 📄 License and credits

BSD 3-Clause. See [LICENSE](./LICENSE).

Thanks to [CoralMap](https://github.com/CoralFans-Dev/CoralMap) for the inspiration, and to [LeviLamina](https://github.com/LiteLDev/LeviLamina) and [egui](https://github.com/emilk/egui).
