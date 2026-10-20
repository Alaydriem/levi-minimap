use std::time::{Duration, Instant};

use minimap_runtime::services::TelemetryRecorder;
use minimap_runtime::structs::{FrameInput, RuntimeStats, TelemetryRow};

fn frame(render_us: u32) -> FrameInput {
    FrameInput {
        render_us,
        ui_vtx: 640,
        working_set_bytes: 300 * 1024 * 1024,
        private_bytes: 200 * 1024 * 1024,
        ..FrameInput::default()
    }
}

#[test]
fn a_row_holds_the_interval_quantiles_probe_and_bake_means() {
    let start = Instant::now();
    let mut recorder = TelemetryRecorder::new(Duration::from_secs(1), start);
    for us in [1000, 2000, 3000, 4000] {
        recorder.on_frame(&frame(us), us / 10);
    }
    // the tick call takes 2 ms, then the shim probes two chunks over the next 0.3 ms
    recorder.on_tick(start, start + Duration::from_millis(2));
    recorder.on_probe(start + Duration::from_micros(2100));
    recorder.on_probe(start + Duration::from_micros(2300));
    assert!(!recorder.due(start + Duration::from_millis(500)));
    assert!(recorder.due(start + Duration::from_secs(1)));
    let stats = RuntimeStats { bake_queue: 3, gpu_textures: 9, disk_written_bytes: 4096, ..RuntimeStats::default() };
    let row = recorder.take_row(start + Duration::from_secs(1), &stats, (4, 400_000));
    assert_eq!(row.t_ms, 1000);
    assert_eq!(row.render_ms_p50, 2.0);
    assert_eq!(row.render_ms_p99, 4.0);
    assert!((row.tick_ms_p50 - 2.3).abs() < 1e-3, "{}", row.tick_ms_p50);
    assert_eq!(row.chunks_probed, 2);
    assert!((row.probe_us_mean - 150.0).abs() < 1.0, "{}", row.probe_us_mean);
    assert_eq!(row.bake_us_mean, 100.0);
    assert_eq!((row.ui_build_ms_p50, row.ui_build_ms_p99), (0.2, 0.4));
    assert_eq!((row.bake_queue, row.gpu_textures, row.ui_vtx), (3, 9, 640));
    assert_eq!((row.working_set_mb, row.private_mb, row.disk_write_kb), (300.0, 200.0, 4.0));
}

#[test]
fn each_row_covers_only_its_own_interval() {
    let start = Instant::now();
    let mut recorder = TelemetryRecorder::new(Duration::from_secs(1), start);
    recorder.on_frame(&frame(5000), 0);
    let stats = RuntimeStats { disk_written_bytes: 10_240, ..RuntimeStats::default() };
    recorder.take_row(start + Duration::from_secs(1), &stats, (10, 1_000_000));
    let later = RuntimeStats { disk_written_bytes: 2048, ..RuntimeStats::default() };
    let row = recorder.take_row(start + Duration::from_secs(2), &later, (10, 1_000_000));
    assert_eq!(row.render_ms_p50, 0.0);
    assert_eq!(row.bake_us_mean, 0.0);
    // a new world restarted the store's byte count
    assert_eq!(row.disk_write_kb, 2.0);
}

#[test]
fn a_row_has_one_value_per_header_column() {
    let line = TelemetryRow::default().to_csv();
    assert_eq!(line.split(',').count(), TelemetryRow::HEADER.split(',').count());
    assert_eq!(TelemetryRow::HEADER.split(',').count(), 19);
    assert!(TelemetryRow::HEADER.starts_with("t_ms,render_ms_p50,"));
}
