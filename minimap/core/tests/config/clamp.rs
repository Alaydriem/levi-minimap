use minimap_core::config::{Config, ConfigClamp};

#[test]
fn defaults_need_no_clamping() {
    let (config, warnings) = ConfigClamp::new().apply(Config::default());
    assert_eq!(config, Config::default());
    assert!(warnings.is_empty());
}

#[test]
fn out_of_range_values_are_clamped_with_a_warning_naming_the_key() {
    let mut input = Config::default();
    input.terrain.shadow.render_scale = 40;
    input.mini_map.widget_scale = 9.0;
    input.terrain.shadow.pcf_radius = -3;
    let (config, warnings) = ConfigClamp::new().apply(input);
    assert_eq!(config.terrain.shadow.render_scale, 16);
    assert_eq!(config.mini_map.widget_scale, 3.0);
    assert_eq!(config.terrain.shadow.pcf_radius, 0);
    let keys: Vec<&str> = warnings.iter().map(|w| w.key).collect();
    assert_eq!(keys, vec!["miniMap.widgetScale", "terrain.shadow.renderScale", "terrain.shadow.pcfRadius"]);
}

#[test]
fn minimap_radius_is_held_between_its_own_limits() {
    let mut input = Config::default();
    input.mini_map.min_radius_chunks = 4;
    input.mini_map.max_radius_chunks = 3;
    input.mini_map.radius_chunks = 1;
    let (config, _) = ConfigClamp::new().apply(input);
    assert_eq!(config.mini_map.min_radius_chunks, 4);
    assert_eq!(config.mini_map.max_radius_chunks, 4);
    assert_eq!(config.mini_map.radius_chunks, 4);
}

#[test]
fn shadow_level_at_or_below_zero_means_one_hundred() {
    let mut input = Config::default();
    input.terrain.shadow.shadow_level = 0;
    let (config, warnings) = ConfigClamp::new().apply(input);
    assert_eq!(config.terrain.shadow.shadow_level, 100);
    assert_eq!(warnings[0].key, "terrain.shadow.shadowLevel");
}

#[test]
fn waypoint_limits_are_held_in_range() {
    let mut input = Config::default();
    input.waypoints.max_per_world = 0;
    input.waypoints.max_edge_arrows = 500;
    input.world_map.waypoint_label_zoom = 0.0;
    let (config, warnings) = ConfigClamp::new().apply(input);
    assert_eq!(config.waypoints.max_per_world, 1);
    assert_eq!(config.waypoints.max_edge_arrows, 64);
    assert_eq!(config.world_map.waypoint_label_zoom, 0.01);
    let keys: Vec<&str> = warnings.iter().map(|w| w.key).collect();
    assert_eq!(keys, vec!["worldMap.waypointLabelZoom", "waypoints.maxPerWorld", "waypoints.maxEdgeArrows"]);
}

#[test]
fn the_export_size_limit_is_held_in_range() {
    let mut input = Config::default();
    input.export.max_pixels = 10;
    let (config, warnings) = ConfigClamp::new().apply(input);
    assert_eq!(config.export.max_pixels, 256);
    assert_eq!(warnings[0].key, "export.maxPixels");
}

#[test]
fn the_cave_band_size_snaps_to_eight_sixteen_or_thirty_two_and_the_margin_stays_inside_it() {
    let mut input = Config::default();
    input.cave_mode.band_size = 12;
    input.cave_mode.band_margin = 9;
    input.cave_mode.scan_radius = 2;
    let (config, warnings) = ConfigClamp::new().apply(input);
    assert_eq!((config.cave_mode.band_size, config.cave_mode.band_margin, config.cave_mode.scan_radius), (16, 7, 16));
    let keys: Vec<&str> = warnings.iter().map(|w| w.key).collect();
    assert_eq!(keys, vec!["caveMode.bandSize", "caveMode.bandMargin", "caveMode.scanRadius"]);
    let mut big = Config::default();
    big.cave_mode.band_size = 100;
    assert_eq!(ConfigClamp::new().apply(big).0.cave_mode.band_size, 32);
}

#[test]
fn hover_biome_regions_is_clamped() {
    let mut input = Config::default();
    assert_eq!(input.world_map.hover_biome_regions, 16);
    input.world_map.hover_biome_regions = 0;
    let (config, warnings) = ConfigClamp::new().apply(input);
    assert_eq!(config.world_map.hover_biome_regions, 1);
    assert!(warnings.iter().any(|w| w.key == "worldMap.hoverBiomeRegions"));
}

#[test]
fn the_slime_cache_size_is_held_in_range() {
    let mut input = Config::default();
    input.world_map.slime_cache_regions = 10;
    let (config, warnings) = ConfigClamp::new().apply(input);
    assert_eq!(config.world_map.slime_cache_regions, 64);
    assert!(warnings.iter().any(|w| w.key == "worldMap.slimeCacheRegions"));
}
