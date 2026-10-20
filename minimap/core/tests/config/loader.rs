use minimap_core::config::{Config, ConfigLoader, WriteBack};

#[test]
fn missing_file_uses_defaults_and_asks_for_creation() {
    let loaded = ConfigLoader::load(None);
    assert_eq!(loaded.config, Config::default());
    assert_eq!(loaded.write_back, WriteBack::Create);
}

#[test]
fn unreadable_file_uses_defaults_and_asks_to_keep_a_copy() {
    let loaded = ConfigLoader::load(Some("{ not json"));
    assert_eq!(loaded.config, Config::default());
    assert_eq!(loaded.write_back, WriteBack::ReplaceInvalid);
    assert_eq!(loaded.warnings.len(), 1);
}

#[test]
fn existing_keys_are_read_and_missing_keys_keep_their_defaults() {
    let text = r#"{
        "fontSize": 20.0,
        "miniMap": { "radiusChunks": 7, "sizeRatio": 0.25 },
        "terrain": { "shadow": { "renderScale": 4 }, "enableDiskCache": false },
        "worldMap": { "toggleKey": 78 },
        "somethingUnknown": 1
    }"#;
    let loaded = ConfigLoader::load(Some(text));
    let config = loaded.config;
    assert_eq!(loaded.write_back, WriteBack::None);
    assert!(loaded.warnings.is_empty());
    assert_eq!(config.font_size, 20.0);
    assert_eq!(config.mini_map.radius_chunks, 7);
    assert_eq!(config.mini_map.size_ratio, 0.25);
    assert_eq!(config.mini_map.margin_right, 20.0);
    assert_eq!(config.terrain.shadow.render_scale, 4);
    assert_eq!(config.terrain.shadow.pcf_radius, 1);
    assert!(!config.terrain.enable_disk_cache);
    assert_eq!(config.world_map.toggle_key, 78);
    assert!(config.mini_map.show_coordinates);
    assert_eq!(config.memory.tile_budget_mb, 256);
    assert_eq!(config.memory.gpu_texture_budget, 1024);
    assert_eq!(
        config.terrain.biome_color_overrides.get("the_end"),
        Some(&[16, 12, 26, 255]),
        "the default End void override"
    );
}

#[test]
fn written_config_keeps_the_existing_key_names() {
    let json = ConfigLoader::to_json(&Config::default()).expect("serializes");
    for key in [
        "\"miniMap\"",
        "\"radiusChunks\"",
        "\"scanMaxChunksPerFrame\"",
        "\"renderScale\"",
        "\"toggleKey\"",
        "\"widgetScale\"",
    ] {
        assert!(json.contains(key), "missing {key}");
    }
}

#[test]
fn a_wrong_type_in_one_section_keeps_every_other_section() {
    let text = r#"{ "miniMap": { "radiusChunks": "five" }, "terrain": { "scanRadius": 256 } }"#;
    let loaded = ConfigLoader::load(Some(text));
    assert_eq!(loaded.write_back, WriteBack::None);
    assert_eq!(loaded.config.mini_map, Config::default().mini_map);
    assert_eq!(loaded.config.terrain.scan_radius, 256);
    assert_eq!(loaded.warnings.len(), 1);
    assert_eq!(loaded.warnings[0].key, "miniMap");
}

#[test]
fn a_top_level_that_is_not_an_object_is_invalid() {
    assert_eq!(ConfigLoader::load(Some("[1, 2]")).write_back, WriteBack::ReplaceInvalid);
}

#[test]
fn lava_is_drawn_orange_unless_the_config_names_its_own_block_colours() {
    let defaults = ConfigLoader::load(None).config.terrain.block_colors;
    assert_eq!(defaults.get("minecraft:lava"), Some(&[230, 110, 20, 255]));
    let text = r#"{ "terrain": { "blockColors": { "minecraft:magma": [200, 60, 10, 255] } } }"#;
    let colours = ConfigLoader::load(Some(text)).config.terrain.block_colors;
    assert_eq!(colours.len(), 1);
    assert_eq!(colours.get("minecraft:magma"), Some(&[200, 60, 10, 255]));
}

#[test]
fn block_colours_with_no_name_are_dropped_and_the_list_is_capped() {
    let mut entries: Vec<String> = (0..80).map(|i| format!("\"minecraft:block_{i:02}\": [1, 2, 3, 255]")).collect();
    entries.push("\"\": [9, 9, 9, 255]".to_string());
    let text = format!(r#"{{ "terrain": {{ "blockColors": {{ {} }} }} }}"#, entries.join(", "));
    let loaded = ConfigLoader::load(Some(&text));
    let colours = &loaded.config.terrain.block_colors;
    assert_eq!(colours.len(), minimap_core::config::TerrainConfig::MAX_BLOCK_COLORS);
    assert!(!colours.contains_key(""));
    assert!(loaded.warnings.iter().all(|w| w.key == "terrain.blockColors"));
    assert_eq!(loaded.warnings.len(), 2);
}

#[test]
fn biome_colour_overrides_with_no_name_are_dropped_and_the_list_is_capped() {
    let mut entries: Vec<String> = (0..70).map(|i| format!("\"biome_{i:02}\": [1, 2, 3, 255]")).collect();
    entries.push("\"\": [9, 9, 9, 255]".to_string());
    let text = format!(r#"{{ "terrain": {{ "biomeColorOverrides": {{ {} }} }} }}"#, entries.join(", "));
    let loaded = ConfigLoader::load(Some(&text));
    let overrides = &loaded.config.terrain.biome_color_overrides;
    assert_eq!(overrides.len(), minimap_core::config::TerrainConfig::MAX_BIOME_COLOR_OVERRIDES);
    assert_eq!(overrides.len(), 64);
    assert!(!overrides.contains_key(""));
    assert!(overrides.contains_key("biome_00"));
    assert!(!overrides.contains_key("biome_69"), "entries after the first 64 by name are dropped");
    assert!(loaded.warnings.iter().all(|w| w.key == "terrain.biomeColorOverrides"));
    assert_eq!(loaded.warnings.len(), 2);
}

#[test]
fn the_world_map_tools_default_off() {
    let config = ConfigLoader::load(Some("{}")).config;
    assert!(!config.world_map.show_chunk_grid);
    assert!(!config.world_map.show_slime_chunks);
    assert_eq!(config.world_map.slime_cache_regions, 1024);
}
