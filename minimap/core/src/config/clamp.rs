use std::collections::BTreeMap;

use crate::config::{
    BakeConfig, CaveModeConfig, Config, ConfigWarning, DebugConfig, ExportConfig, MemoryConfig, MiniMapConfig,
    ShadowConfig, SmoothCameraConfig, StoreConfig, TerrainConfig, WaypointsConfig, WorldMapConfig,
};

pub struct ConfigClamp {
    warnings: Vec<ConfigWarning>,
}

impl ConfigClamp {
    pub fn new() -> Self {
        Self { warnings: Vec::new() }
    }

    pub fn apply(mut self, config: Config) -> (Config, Vec<ConfigWarning>) {
        let clamped = Config {
            font_size: self.float("fontSize", config.font_size, 6.0, 96.0),
            mini_map: self.mini_map(config.mini_map),
            smooth_camera: self.smooth_camera(config.smooth_camera),
            terrain: self.terrain(config.terrain),
            world_map: self.world_map(config.world_map),
            memory: self.memory(config.memory),
            bake: self.bake(config.bake),
            store: self.store(config.store),
            debug: self.debug(config.debug),
            waypoints: self.waypoints(config.waypoints),
            cave_mode: self.cave_mode(config.cave_mode),
            export: ExportConfig {
                max_pixels: self.int("export.maxPixels", config.export.max_pixels, 256, 65_536),
                ..config.export
            },
            ..config
        };
        (clamped, self.warnings)
    }

    fn mini_map(&mut self, c: MiniMapConfig) -> MiniMapConfig {
        let min_radius = self.int("miniMap.minRadiusChunks", c.min_radius_chunks, 1, 64);
        let max_radius = self.int("miniMap.maxRadiusChunks", c.max_radius_chunks, min_radius, 64);
        MiniMapConfig {
            size_ratio: self.float("miniMap.sizeRatio", c.size_ratio, 0.02, 1.0),
            radius_chunks: self.int("miniMap.radiusChunks", c.radius_chunks, min_radius, max_radius),
            min_radius_chunks: min_radius,
            max_radius_chunks: max_radius,
            line_thickness: self.float("miniMap.lineThickness", c.line_thickness, 0.0, 16.0),
            border_thickness: self.float("miniMap.borderThickness", c.border_thickness, 0.0, 16.0),
            widget_scale: self.float("miniMap.widgetScale", c.widget_scale, 0.5, 3.0),
            ..c
        }
    }

    fn smooth_camera(&mut self, c: SmoothCameraConfig) -> SmoothCameraConfig {
        SmoothCameraConfig {
            spring_strength: self.float("smoothCamera.springStrength", c.spring_strength, 0.0, 1000.0),
            damping: self.float("smoothCamera.damping", c.damping, 0.0, 1000.0),
            max_delta_time: self.float("smoothCamera.maxDeltaTime", c.max_delta_time, 0.001, 1.0),
        }
    }

    fn terrain(&mut self, c: TerrainConfig) -> TerrainConfig {
        TerrainConfig {
            scan_max_chunks_per_frame: self.int("terrain.scanMaxChunksPerFrame", c.scan_max_chunks_per_frame, 1, 1024),
            scan_radius: self.int("terrain.scanRadius", c.scan_radius, 16, 1024),
            rescan_interval_frames: self.int("terrain.rescanIntervalFrames", c.rescan_interval_frames, 1, 1_000_000),
            region_bake_threshold: self.int("terrain.regionBakeThreshold", c.region_bake_threshold, 1, 256),
            water_opacity_per_depth: self.float("terrain.waterOpacityPerDepth", c.water_opacity_per_depth, 0.0, 1.0),
            water_max_opacity: self.float("terrain.waterMaxOpacity", c.water_max_opacity, 0.0, 1.0),
            shadow: self.shadow(c.shadow),
            block_colors: self.block_colors(c.block_colors),
            biome_color_overrides: self.biome_color_overrides(c.biome_color_overrides),
            ..c
        }
    }

    // a colour needs a block name, and the probe checks every entry, so the list is capped
    fn block_colors(&mut self, mut colours: BTreeMap<String, [u8; 4]>) -> BTreeMap<String, [u8; 4]> {
        if colours.remove("").is_some() {
            self.warn("terrain.blockColors", "an entry with no block name is ignored".to_string());
        }
        if colours.len() > TerrainConfig::MAX_BLOCK_COLORS {
            self.warn(
                "terrain.blockColors",
                format!("{} entries; only the first {} are used", colours.len(), TerrainConfig::MAX_BLOCK_COLORS),
            );
            colours = colours.into_iter().take(TerrainConfig::MAX_BLOCK_COLORS).collect();
        }
        colours
    }

    // an override needs a biome name, and the list is held in memory for the whole session, so it is capped
    fn biome_color_overrides(&mut self, mut colours: BTreeMap<String, [u8; 4]>) -> BTreeMap<String, [u8; 4]> {
        if colours.remove("").is_some() {
            self.warn("terrain.biomeColorOverrides", "an entry with no biome name is ignored".to_string());
        }
        if colours.len() > TerrainConfig::MAX_BIOME_COLOR_OVERRIDES {
            self.warn(
                "terrain.biomeColorOverrides",
                format!(
                    "{} entries; only the first {} are used",
                    colours.len(),
                    TerrainConfig::MAX_BIOME_COLOR_OVERRIDES
                ),
            );
            colours = colours.into_iter().take(TerrainConfig::MAX_BIOME_COLOR_OVERRIDES).collect();
        }
        colours
    }

    fn shadow(&mut self, c: ShadowConfig) -> ShadowConfig {
        let shadow_level = if c.shadow_level <= 0 {
            self.warn("terrain.shadow.shadowLevel", format!("{} is not above 0; using 100", c.shadow_level));
            100
        } else {
            self.int("terrain.shadow.shadowLevel", c.shadow_level, 1, 1000)
        };
        ShadowConfig {
            render_style: self.int("terrain.shadow.renderStyle", c.render_style, 0, 2),
            shadow_level,
            render_scale: self.int("terrain.shadow.renderScale", c.render_scale, 1, 16),
            pcf_radius: self.int("terrain.shadow.pcfRadius", c.pcf_radius, 0, 8),
            light_azimuth: self.float("terrain.shadow.lightAzimuth", c.light_azimuth, 0.0, 360.0),
            light_zenith: self.float("terrain.shadow.lightZenith", c.light_zenith, 1.0, 89.0),
        }
    }

    fn world_map(&mut self, c: WorldMapConfig) -> WorldMapConfig {
        let min_zoom = self.float("worldMap.minZoom", c.min_zoom, 0.01, 10.0);
        WorldMapConfig {
            min_zoom,
            max_zoom: self.float("worldMap.maxZoom", c.max_zoom, min_zoom, 400.0),
            zoom_step: self.float("worldMap.zoomStep", c.zoom_step, 1.01, 4.0),
            max_new_textures_per_frame: self.int(
                "worldMap.maxNewTexturesPerFrame",
                c.max_new_textures_per_frame,
                1,
                64,
            ),
            texture_idle_frames: self.int("worldMap.textureIdleFrames", c.texture_idle_frames, 1, 1_000_000),
            waypoint_label_zoom: self.float("worldMap.waypointLabelZoom", c.waypoint_label_zoom, 0.01, 400.0),
            hover_biome_regions: self.int("worldMap.hoverBiomeRegions", c.hover_biome_regions, 1, 1024),
            slime_cache_regions: self.int("worldMap.slimeCacheRegions", c.slime_cache_regions, 64, 65_536),
            ..c
        }
    }

    fn memory(&mut self, c: MemoryConfig) -> MemoryConfig {
        MemoryConfig {
            chunk_surface_budget_mb: self.int("memory.chunkSurfaceBudgetMb", c.chunk_surface_budget_mb, 8, 4096),
            tile_budget_mb: self.int("memory.tileBudgetMb", c.tile_budget_mb, 8, 4096),
            tile_overrun_percent: self.int("memory.tileOverrunPercent", c.tile_overrun_percent, 0, 100),
            gpu_texture_budget: self.int("memory.gpuTextureBudget", c.gpu_texture_budget, 16, 4096),
        }
    }

    fn bake(&mut self, c: BakeConfig) -> BakeConfig {
        BakeConfig {
            worker_threads: self.int("bake.workerThreads", c.worker_threads, 0, 64),
            queue_capacity: self.int("bake.queueCapacity", c.queue_capacity, 64, 1_000_000),
        }
    }

    fn store(&mut self, c: StoreConfig) -> StoreConfig {
        StoreConfig {
            journal_max_mb: self.int("store.journalMaxMb", c.journal_max_mb, 4, 4096),
            sync_interval_ms: self.int("store.syncIntervalMs", c.sync_interval_ms, 100, 60_000),
            write_queue_capacity: self.int("store.writeQueueCapacity", c.write_queue_capacity, 16, 1_000_000),
        }
    }

    fn debug(&mut self, c: DebugConfig) -> DebugConfig {
        DebugConfig {
            telemetry_interval_ms: self.int("debug.telemetryIntervalMs", c.telemetry_interval_ms, 100, 60_000),
            ..c
        }
    }

    // the band size snaps to the nearest of 8, 16 and 32; the margin stays under half a band
    fn cave_mode(&mut self, c: CaveModeConfig) -> CaveModeConfig {
        let band_size = match c.band_size {
            ..=11 => 8,
            12..=23 => 16,
            _ => 32,
        };
        if band_size != c.band_size {
            self.warn("caveMode.bandSize", format!("{} is not 8, 16 or 32; using {band_size}", c.band_size));
        }
        CaveModeConfig {
            band_size,
            band_margin: self.int("caveMode.bandMargin", c.band_margin, 0, band_size / 2 - 1),
            scan_radius: self.int("caveMode.scanRadius", c.scan_radius, 16, 1024),
            ..c
        }
    }

    fn waypoints(&mut self, c: WaypointsConfig) -> WaypointsConfig {
        WaypointsConfig {
            max_per_world: self.int("waypoints.maxPerWorld", c.max_per_world, 1, 4096),
            max_edge_arrows: self.int("waypoints.maxEdgeArrows", c.max_edge_arrows, 0, 64),
            label_scale: self.float("waypoints.labelScale", c.label_scale, 0.5, 4.0),
            ..c
        }
    }

    fn int(&mut self, key: &'static str, value: i32, min: i32, max: i32) -> i32 {
        let clamped = value.clamp(min, max);
        if clamped != value {
            self.warn(key, format!("{value} is outside {min}..={max}; using {clamped}"));
        }
        clamped
    }

    fn float(&mut self, key: &'static str, value: f32, min: f32, max: f32) -> f32 {
        if value.is_nan() {
            self.warn(key, format!("NaN is not a number; using {min}"));
            return min;
        }
        let clamped = value.clamp(min, max);
        if clamped != value {
            self.warn(key, format!("{value} is outside {min}..={max}; using {clamped}"));
        }
        clamped
    }

    fn warn(&mut self, key: &'static str, message: String) {
        self.warnings.push(ConfigWarning::new(key, message));
    }
}

impl Default for ConfigClamp {
    fn default() -> Self {
        Self::new()
    }
}
