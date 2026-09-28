use crate::metrics::{CaptureMetrics, FrameSample, FrameSummary, QpcAnchor};
use crate::presentmon::{PresentMonBackend, PresentMonCapture};
use crate::process::TargetProcess;
use serde::Serialize;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{Pid, System};
use zip::write::SimpleFileOptions;

const REPORT_ENTRIES: &[&str] = &[
    "manifest.json",
    "summary.json",
    "quality.json",
    "environment.json",
    "capabilities.json",
    "frames/recent.csv",
];

#[derive(Debug, Serialize)]
pub struct EnvironmentSnapshot {
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub architecture: String,
    pub cpu_brand: Option<String>,
    pub logical_cpu_count: usize,
    pub physical_cpu_count: Option<usize>,
    pub total_memory_bytes: u64,
    pub target: TargetProcess,
    pub target_resident_memory_bytes_at_start: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CapabilitySnapshot {
    pub platform_windows: bool,
    pub qpc: bool,
    pub qpc_frequency_hz: Option<u64>,
    pub presentmon: BackendCapability,
    pub presentmon_gpu_tracking_requested: bool,
    pub presentmon_gpu_metrics: bool,
    pub presentmon_display_metrics: bool,
    pub presentmon_etw_status: bool,
    pub jfr: bool,
    pub deep_agent: bool,
}

#[derive(Debug, Serialize)]
pub struct BackendCapability {
    pub available: bool,
    pub executable_name: Option<String>,
    pub version: Option<String>,
    pub cli_contract: Option<&'static str>,
    pub cli_contract_verified: bool,
}

#[derive(Debug, Serialize)]
pub struct QualitySnapshot {
    pub capture_quality: &'static str,
    pub frames_parsed: u64,
    pub all_stream_frames: u64,
    pub stream_count: usize,
    pub primary_frame_share: f64,
    pub rows_rejected: u64,
    pub etw_loss_detection_available: bool,
    pub etw_events_lost: Option<u64>,
    pub etw_buffers_lost: Option<u64>,
    pub overflowed_presents: Option<u64>,
    pub presentmon_exit_code: Option<i32>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Manifest {
    pub report_schema: u32,
    pub probe_version: &'static str,
    pub probe_phase: &'static str,
    pub mode: &'static str,
    pub started_unix_ms: u128,
    pub ended_unix_ms: u128,
    pub qpc_anchor: QpcAnchor,
    pub entries: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct Summary<'a> {
    pub frames: &'a FrameSummary,
    pub metric_notes: MetricNotes,
}

#[derive(Debug, Serialize)]
pub struct MetricNotes {
    pub cpu_frame_time: &'static str,
    pub displayed_time: &'static str,
    pub low_fps: &'static str,
    pub runtime_semantics_validation: &'static str,
}

pub fn environment_snapshot(target: &TargetProcess) -> EnvironmentSnapshot {
    let system = System::new_all();
    let target_memory = system
        .process(Pid::from_u32(target.pid))
        .map(sysinfo::Process::memory);

    EnvironmentSnapshot {
        os_name: System::name(),
        os_version: System::os_version(),
        kernel_version: System::kernel_version(),
        architecture: std::env::consts::ARCH.to_owned(),
        cpu_brand: system.cpus().first().map(|cpu| cpu.brand().to_owned()),
        logical_cpu_count: system.cpus().len(),
        physical_cpu_count: System::physical_core_count(),
        total_memory_bytes: system.total_memory(),
        target: target.clone(),
        target_resident_memory_bytes_at_start: target_memory,
    }
}

pub fn quality_snapshot(capture: &PresentMonCapture) -> QualitySnapshot {
    let summary = &capture.metrics.summary;
    let mut notes = Vec::new();

    if capture.rows_rejected > 0 {
        notes.push(format!(
            "{} PresentMon CSV row(s) were rejected during parsing",
            capture.rows_rejected
        ));
    }
    if summary.stream_count > 1 {
        notes.push(format!(
            "{} present streams were observed; the stream with the most frames was selected as primary",
            summary.stream_count
        ));
    }
    if summary.primary_frame_share < 0.80 {
        notes.push(
            "primary present stream accounted for less than 80% of captured frames".to_owned(),
        );
    }
    if summary.frames < 300 {
        notes.push(
            "primary stream contains fewer than 300 CPU-frame samples; percentile estimates are weak"
                .to_owned(),
        );
    }
    if capture.gpu_tracking_requested && !capture.gpu_metrics_available {
        notes.push(
            "GPU tracking was requested, but PresentMon did not expose the expected GPU timing columns"
                .to_owned(),
        );
    }
    if !capture.display_metrics_available {
        notes.push(
            "PresentMon did not expose DisplayedTime; display-side timing is omitted rather than treated as dropped frames"
                .to_owned(),
        );
    }
    if summary.present_runtimes.contains_key("Other") {
        notes.push(
            "PresentRuntime includes Other (typical for OpenGL/Vulkan); PresentMon documents CPU FrameTime as potentially slightly less accurate for this runtime, so Minecraft/OpenGL semantics still require runtime validation"
                .to_owned(),
        );
    }
    if !capture.etw_status_available {
        notes.push(
            "PresentMon ETW loss counters are unavailable; this capture cannot be treated as benchmark-quality evidence"
                .to_owned(),
        );
    }
    if capture.etw_events_lost.unwrap_or(0) > 0 {
        notes.push(format!(
            "{} ETW event(s) were reported lost",
            capture.etw_events_lost.unwrap_or(0)
        ));
    }
    if capture.etw_buffers_lost.unwrap_or(0) > 0 {
        notes.push(format!(
            "{} ETW buffer(s) were reported lost",
            capture.etw_buffers_lost.unwrap_or(0)
        ));
    }
    if capture.overflowed_presents.unwrap_or(0) > 0 {
        notes.push(format!(
            "{} PresentMon present event(s) overflowed the consumer buffer",
            capture.overflowed_presents.unwrap_or(0)
        ));
    }

    let exit_failed = capture.exit_code.is_some_and(|code| code != 0);
    let trace_loss = capture.etw_events_lost.unwrap_or(0) > 0
        || capture.etw_buffers_lost.unwrap_or(0) > 0
        || capture.overflowed_presents.unwrap_or(0) > 0;
    let capture_quality = if summary.frames < 30 || exit_failed {
        "INVALID"
    } else if capture.rows_rejected > 0
        || summary.primary_frame_share < 0.80
        || summary.frames < 300
        || (capture.gpu_tracking_requested && !capture.gpu_metrics_available)
        || !capture.etw_status_available
        || trace_loss
    {
        "DEGRADED"
    } else {
        "GOOD"
    };

    QualitySnapshot {
        capture_quality,
        frames_parsed: summary.frames,
        all_stream_frames: summary.all_stream_frames,
        stream_count: summary.stream_count,
        primary_frame_share: summary.primary_frame_share,
        rows_rejected: capture.rows_rejected,
        etw_loss_detection_available: capture.etw_status_available,
        etw_events_lost: capture.etw_events_lost,
        etw_buffers_lost: capture.etw_buffers_lost,
        overflowed_presents: capture.overflowed_presents,
        presentmon_exit_code: capture.exit_code,
        notes,
    }
}

pub fn capability_snapshot(
    presentmon: &PresentMonBackend,
    capture: &PresentMonCapture,
    qpc_anchor: QpcAnchor,
) -> CapabilitySnapshot {
    CapabilitySnapshot {
        platform_windows: cfg!(windows),
        qpc: true,
        qpc_frequency_hz: Some(qpc_anchor.frequency_hz),
        presentmon: BackendCapability {
            available: true,
            executable_name: presentmon
                .path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            version: presentmon.version.clone(),
            cli_contract: Some("presentmon-console-v2-qpc"),
            cli_contract_verified: true,
        },
        presentmon_gpu_tracking_requested: capture.gpu_tracking_requested,
        presentmon_gpu_metrics: capture.gpu_metrics_available,
        presentmon_display_metrics: capture.display_metrics_available,
        presentmon_etw_status: capture.etw_status_available,
        jfr: false,
        deep_agent: false,
    }
}

pub fn write_report(
    output_directory: &Path,
    started_unix_ms: u128,
    qpc_anchor: QpcAnchor,
    environment: &EnvironmentSnapshot,
    capabilities: &CapabilitySnapshot,
    quality: &QualitySnapshot,
    metrics: &CaptureMetrics,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    fs::create_dir_all(output_directory)?;

    let ended_unix_ms = unix_millis();
    let file_name = format!("argon-report-{ended_unix_ms}.zip");
    let report_path = output_directory.join(&file_name);
    let temporary_path = output_directory.join(format!(".{file_name}.tmp"));

    let write_result = write_report_archive(
        &temporary_path,
        started_unix_ms,
        ended_unix_ms,
        qpc_anchor,
        environment,
        capabilities,
        quality,
        metrics,
    );
    if let Err(error) = write_result {
        fs::remove_file(&temporary_path).ok();
        return Err(error);
    }

    if let Err(error) = validate_report(&temporary_path) {
        fs::remove_file(&temporary_path).ok();
        return Err(error);
    }

    fs::rename(&temporary_path, &report_path)?;
    Ok(report_path)
}

fn write_report_archive(
    path: &Path,
    started_unix_ms: u128,
    ended_unix_ms: u128,
    qpc_anchor: QpcAnchor,
    environment: &EnvironmentSnapshot,
    capabilities: &CapabilitySnapshot,
    quality: &QualitySnapshot,
    metrics: &CaptureMetrics,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest = Manifest {
        report_schema: 1,
        probe_version: env!("CARGO_PKG_VERSION"),
        probe_phase: "P0",
        mode: "CAPTURE",
        started_unix_ms,
        ended_unix_ms,
        qpc_anchor,
        entries: REPORT_ENTRIES.to_vec(),
    };

    let summary = Summary {
        frames: &metrics.summary,
        metric_notes: MetricNotes {
            cpu_frame_time: "PresentMon v2 FrameTime: time between CPU frame starts. This is recorded explicitly as CPU frame time, not treated as interchangeable with display duration.",
            displayed_time: "PresentMon v2 DisplayedTime: how long a displayed frame remained on screen. NA rows are counted as not displayed only when the DisplayedTime column is actually available.",
            low_fps: "Approximate slow-tail FPS values are derived from a 0.1 ms bounded histogram for the corresponding timing source.",
            runtime_semantics_validation: "P0 metric structure is test-verified, but Minecraft 26.2/OpenGL metric semantics remain runtime-unverified until real runtime validation is completed.",
        },
    };

    write_json(&mut zip, options, "manifest.json", &manifest)?;
    write_json(&mut zip, options, "summary.json", &summary)?;
    write_json(&mut zip, options, "quality.json", quality)?;
    write_json(&mut zip, options, "environment.json", environment)?;
    write_json(&mut zip, options, "capabilities.json", capabilities)?;

    zip.start_file("frames/recent.csv", options)?;
    zip.write_all(&frames_csv(&metrics.recent_frames)?)?;

    let file = zip.finish()?;
    file.sync_all()?;
    Ok(())
}

pub fn validate_report(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for required in REPORT_ENTRIES {
        let entry = archive
            .by_name(required)
            .map_err(|_| format!("report is missing required entry {required}"))?;
        if entry.size() == 0 {
            return Err(format!("report entry {required} is empty").into());
        }
    }

    for json_entry in [
        "manifest.json",
        "summary.json",
        "quality.json",
        "environment.json",
        "capabilities.json",
    ] {
        let bytes = read_zip_entry(&mut archive, json_entry)?;
        serde_json::from_slice::<serde_json::Value>(&bytes)
            .map_err(|error| format!("report entry {json_entry} is invalid JSON: {error}"))?;
    }

    let manifest_bytes = read_zip_entry(&mut archive, "manifest.json")?;
    let manifest: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
    if manifest["report_schema"].as_u64() != Some(1) {
        return Err("report manifest has an unsupported or missing report_schema".into());
    }

    let frames = read_zip_entry(&mut archive, "frames/recent.csv")?;
    let mut csv = csv::Reader::from_reader(frames.as_slice());
    let headers = csv.headers()?;
    for required in ["swapchain", "qpc", "cpu_frame_time_ms"] {
        if !headers.iter().any(|header| header == required) {
            return Err(format!("frames/recent.csv is missing required column {required}").into());
        }
    }

    Ok(())
}

fn read_zip_entry(
    archive: &mut zip::ZipArchive<File>,
    name: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut entry = archive.by_name(name)?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn write_json<T: Serialize>(
    zip: &mut zip::ZipWriter<File>,
    options: SimpleFileOptions,
    name: &str,
    value: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    zip.start_file(name, options)?;
    zip.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn frames_csv(frames: &[FrameSample]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record([
        "swapchain",
        "qpc",
        "cpu_frame_time_ms",
        "cpu_busy_ms",
        "cpu_wait_ms",
        "gpu_time_ms",
        "gpu_busy_ms",
        "displayed_time_ms",
    ])?;

    for frame in frames {
        writer.write_record([
            format!("0x{:X}", frame.swapchain),
            frame.qpc.to_string(),
            format_ms(Some(frame.cpu_frame_time_us)),
            format_ms(frame.cpu_busy_us),
            format_ms(frame.cpu_wait_us),
            format_ms(frame.gpu_time_us),
            format_ms(frame.gpu_busy_us),
            format_ms(frame.displayed_time_us),
        ])?;
    }

    Ok(writer.into_inner()?)
}

fn format_ms(value: Option<u64>) -> String {
    value
        .map(|microseconds| format!("{:.3}", microseconds as f64 / 1_000.0))
        .unwrap_or_default()
}

pub fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::{CaptureAccumulator, FrameSample};
    use std::io::Read;

    #[test]
    fn quality_requires_available_zero_loss_etw_evidence_for_good_capture() {
        let mut accumulator = CaptureAccumulator::new(1_000, 120, false);
        for qpc in 1..=400 {
            accumulator.observe(
                FrameSample {
                    swapchain: 1,
                    qpc,
                    cpu_frame_time_us: 10_000,
                    cpu_busy_us: None,
                    cpu_wait_us: None,
                    gpu_time_us: None,
                    gpu_busy_us: None,
                    displayed_time_us: None,
                },
                None,
                None,
            );
        }

        let mut capture = PresentMonCapture {
            metrics: accumulator.finish().expect("metrics"),
            rows_rejected: 0,
            exit_code: Some(0),
            gpu_tracking_requested: false,
            gpu_metrics_available: false,
            display_metrics_available: false,
            etw_status_available: false,
            etw_events_lost: None,
            etw_buffers_lost: None,
            overflowed_presents: None,
        };

        assert_eq!(quality_snapshot(&capture).capture_quality, "DEGRADED");

        capture.etw_status_available = true;
        capture.etw_events_lost = Some(0);
        capture.etw_buffers_lost = Some(0);
        capture.overflowed_presents = Some(0);
        assert_eq!(quality_snapshot(&capture).capture_quality, "GOOD");

        capture.etw_events_lost = Some(1);
        let degraded = quality_snapshot(&capture);
        assert_eq!(degraded.capture_quality, "DEGRADED");
        assert!(
            degraded
                .notes
                .iter()
                .any(|note| note.contains("ETW event(s) were reported lost"))
        );
    }

    #[test]
    fn report_contains_required_entries_and_explicit_metric_semantics() {
        let unique = format!("argon-probe-test-{}-{}", std::process::id(), unix_millis());
        let directory = std::env::temp_dir().join(unique);

        let target = TargetProcess {
            pid: 42,
            name: "javaw.exe".to_owned(),
            executable_name: Some("javaw.exe".to_owned()),
        };
        let environment = EnvironmentSnapshot {
            os_name: Some("Windows".to_owned()),
            os_version: None,
            kernel_version: None,
            architecture: "x86_64".to_owned(),
            cpu_brand: None,
            logical_cpu_count: 1,
            physical_cpu_count: Some(1),
            total_memory_bytes: 1,
            target,
            target_resident_memory_bytes_at_start: Some(1),
        };

        let mut accumulator = CaptureAccumulator::new(1_000, 120, true);
        for qpc in 1..=400 {
            accumulator.observe(
                FrameSample {
                    swapchain: 1,
                    qpc,
                    cpu_frame_time_us: 10_000,
                    cpu_busy_us: Some(8_000),
                    cpu_wait_us: Some(2_000),
                    gpu_time_us: Some(7_000),
                    gpu_busy_us: Some(6_000),
                    displayed_time_us: Some(10_000),
                },
                Some("Composed: Flip"),
                Some("Other"),
            );
        }
        let metrics = accumulator.finish().expect("metrics");

        let capabilities = CapabilitySnapshot {
            platform_windows: true,
            qpc: true,
            qpc_frequency_hz: Some(1_000),
            presentmon: BackendCapability {
                available: true,
                executable_name: Some("PresentMon.exe".to_owned()),
                version: None,
                cli_contract: Some("presentmon-console-v2-qpc"),
                cli_contract_verified: true,
            },
            presentmon_gpu_tracking_requested: true,
            presentmon_gpu_metrics: true,
            presentmon_display_metrics: true,
            presentmon_etw_status: true,
            jfr: false,
            deep_agent: false,
        };
        let quality = QualitySnapshot {
            capture_quality: "GOOD",
            frames_parsed: 400,
            all_stream_frames: 400,
            stream_count: 1,
            primary_frame_share: 1.0,
            rows_rejected: 0,
            etw_loss_detection_available: true,
            etw_events_lost: Some(0),
            etw_buffers_lost: Some(0),
            overflowed_presents: Some(0),
            presentmon_exit_code: Some(0),
            notes: Vec::new(),
        };

        let path = write_report(
            &directory,
            unix_millis(),
            QpcAnchor {
                qpc: 1,
                frequency_hz: 1_000,
            },
            &environment,
            &capabilities,
            &quality,
            &metrics,
        )
        .expect("report should be written");

        validate_report(&path).expect("fresh report should validate");

        let file = File::open(&path).expect("report file");
        let mut archive = zip::ZipArchive::new(file).expect("zip should open");

        for required in [
            "manifest.json",
            "summary.json",
            "quality.json",
            "environment.json",
            "capabilities.json",
            "frames/recent.csv",
        ] {
            let mut entry = archive.by_name(required).expect("required entry");
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).expect("entry should read");
            assert!(!bytes.is_empty());
        }

        let mut summary_entry = archive.by_name("summary.json").expect("summary entry");
        let mut summary_bytes = Vec::new();
        summary_entry
            .read_to_end(&mut summary_bytes)
            .expect("summary should read");
        drop(summary_entry);

        let summary_json: serde_json::Value =
            serde_json::from_slice(&summary_bytes).expect("summary JSON");
        assert!(
            summary_json["frames"]["cpu_frame_time"]["p99_ms"]
                .as_f64()
                .is_some()
        );
        assert!(summary_json["frames"].get("average_frame_ms").is_none());

        fs::remove_dir_all(directory).ok();
    }
}
