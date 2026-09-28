use crate::metrics::{CaptureAccumulator, CaptureMetrics, FrameSample};
use csv::{ReaderBuilder, StringRecord};
use std::env;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const REQUIRED_CLI_OPTIONS: &[&str] = &[
    "--process_id",
    "--output_stdout",
    "--no_console_stats",
    "--qpc_time",
    "--v2_metrics",
    "--no_track_input",
    "--no_track_gpu",
    "--timed",
    "--terminate_after_timed",
    "--terminate_on_proc_exit",
    "--session_name",
];

const ETW_STATUS_CLI_OPTION: &str = "--track_etw_status";

#[derive(Debug, Clone)]
pub struct PresentMonBackend {
    pub path: PathBuf,
    pub version: Option<String>,
    pub etw_status_tracking: bool,
}

pub struct PresentMonCapture {
    pub metrics: CaptureMetrics,
    pub rows_rejected: u64,
    pub exit_code: Option<i32>,
    pub gpu_tracking_requested: bool,
    pub gpu_metrics_available: bool,
    pub display_metrics_available: bool,
    pub etw_status_available: bool,
    pub etw_events_lost: Option<u64>,
    pub etw_buffers_lost: Option<u64>,
    pub overflowed_presents: Option<u64>,
}

pub fn resolve_presentmon(explicit: Option<&Path>) -> Result<PresentMonBackend, String> {
    let path = find_presentmon_path(explicit)?;
    inspect_presentmon(&path)
}

fn find_presentmon_path(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        return validate_candidate(path.to_path_buf());
    }

    if let Some(path) = env::var_os("ARGON_PROBE_PRESENTMON") {
        return validate_candidate(PathBuf::from(path));
    }

    if let Ok(current_exe) = env::current_exe()
        && let Some(parent) = current_exe.parent()
        && let Some(candidate) = find_in_directory(parent)
    {
        return Ok(candidate);
    }

    if let Some(path_value) = env::var_os("PATH") {
        for directory in env::split_paths(&path_value) {
            if let Some(candidate) = find_in_directory(&directory) {
                return Ok(candidate);
            }
        }
    }

    Err(
        "PresentMon was not found. Pass --presentmon PATH, set ARGON_PROBE_PRESENTMON, or place an official PresentMon console executable next to argon-probe / on PATH."
            .to_owned(),
    )
}

fn inspect_presentmon(path: &Path) -> Result<PresentMonBackend, String> {
    let output = Command::new(path)
        .arg("--help")
        .output()
        .map_err(|error| format!("failed to execute PresentMon --help: {error}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let help = format!("{stdout}\n{stderr}");

    validate_help_contract(&help)?;

    Ok(PresentMonBackend {
        path: path.to_path_buf(),
        version: version_from_help(&help),
        etw_status_tracking: help.contains(ETW_STATUS_CLI_OPTION),
    })
}

fn validate_help_contract(help: &str) -> Result<(), String> {
    let missing = REQUIRED_CLI_OPTIONS
        .iter()
        .copied()
        .filter(|option| !help.contains(option))
        .collect::<Vec<_>>();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "PresentMon does not expose the CLI surface required by Argon Probe P0; missing option(s): {}",
            missing.join(", ")
        ))
    }
}

fn version_from_help(help: &str) -> Option<String> {
    help.lines()
        .map(str::trim)
        .find(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains("presentmon") && !lower.starts_with("--")
        })
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

pub fn capture(
    backend: &PresentMonBackend,
    pid: u32,
    duration_seconds: u64,
    ring_seconds: u64,
    qpc_frequency_hz: u64,
    track_gpu: bool,
) -> Result<PresentMonCapture, Box<dyn std::error::Error>> {
    let session_name = format!("ArgonProbe-{pid}-{}", unix_millis());
    let pid_arg = pid.to_string();
    let duration_arg = duration_seconds.to_string();

    let mut command = Command::new(&backend.path);
    command
        .arg("--process_id")
        .arg(&pid_arg)
        .arg("--output_stdout")
        .arg("--no_console_stats")
        .arg("--qpc_time")
        .arg("--v2_metrics")
        .arg("--no_track_input");

    if !track_gpu {
        command.arg("--no_track_gpu");
    }

    if backend.etw_status_tracking {
        command.arg(ETW_STATUS_CLI_OPTION);
    }

    let mut child = command
        .arg("--timed")
        .arg(&duration_arg)
        .arg("--terminate_after_timed")
        .arg("--terminate_on_proc_exit")
        .arg("--session_name")
        .arg(&session_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or("failed to capture PresentMon stdout")?;

    let parsed = parse_presentmon_csv(stdout, pid, qpc_frequency_hz, ring_seconds)?;
    let status = child.wait()?;

    Ok(PresentMonCapture {
        metrics: parsed.metrics,
        rows_rejected: parsed.rows_rejected,
        gpu_tracking_requested: track_gpu,
        exit_code: status.code(),
        gpu_metrics_available: parsed.gpu_metrics_available,
        display_metrics_available: parsed.display_metrics_available,
        etw_status_available: parsed.etw_status_available,
        etw_events_lost: parsed.etw_events_lost,
        etw_buffers_lost: parsed.etw_buffers_lost,
        overflowed_presents: parsed.overflowed_presents,
    })
}

struct ParsedCapture {
    metrics: CaptureMetrics,
    rows_rejected: u64,
    gpu_metrics_available: bool,
    display_metrics_available: bool,
    etw_status_available: bool,
    etw_events_lost: Option<u64>,
    etw_buffers_lost: Option<u64>,
    overflowed_presents: Option<u64>,
}

fn parse_presentmon_csv<R: Read>(
    reader: R,
    expected_pid: u32,
    qpc_frequency_hz: u64,
    ring_seconds: u64,
) -> Result<ParsedCapture, Box<dyn std::error::Error>> {
    let mut csv = ReaderBuilder::new().flexible(true).from_reader(reader);
    let headers = csv.headers()?.clone();
    let layout = Layout::from_headers(&headers)?;
    let display_metrics_available = layout.displayed_time.is_some();
    let etw_status_available = layout.etw_events_lost.is_some()
        && layout.etw_buffers_lost.is_some()
        && layout.overflowed_presents.is_some();

    let mut accumulator =
        CaptureAccumulator::new(qpc_frequency_hz, ring_seconds, display_metrics_available);
    let mut rows_rejected = 0_u64;
    let mut etw_events_lost = None;
    let mut etw_buffers_lost = None;
    let mut overflowed_presents = None;

    for row in csv.records() {
        let row = match row {
            Ok(row) => row,
            Err(_) => {
                rows_rejected += 1;
                continue;
            }
        };

        etw_events_lost = max_optional(
            etw_events_lost,
            parse_optional_u64(&row, layout.etw_events_lost),
        );
        etw_buffers_lost = max_optional(
            etw_buffers_lost,
            parse_optional_u64(&row, layout.etw_buffers_lost),
        );
        overflowed_presents = max_optional(
            overflowed_presents,
            parse_optional_u64(&row, layout.overflowed_presents),
        );

        match layout.parse_row(&row, expected_pid) {
            Ok(parsed) => accumulator.observe(
                parsed.sample,
                parsed.present_mode.as_deref(),
                parsed.present_runtime.as_deref(),
            ),
            Err(_) => rows_rejected += 1,
        }
    }

    Ok(ParsedCapture {
        metrics: accumulator.finish()?,
        rows_rejected,
        gpu_metrics_available: layout.gpu_time.is_some() || layout.gpu_busy.is_some(),
        display_metrics_available,
        etw_status_available,
        etw_events_lost,
        etw_buffers_lost,
        overflowed_presents,
    })
}

struct ParsedRow {
    sample: FrameSample,
    present_mode: Option<String>,
    present_runtime: Option<String>,
}

struct Layout {
    process_id: usize,
    swapchain: usize,
    qpc: usize,
    cpu_frame_time: usize,
    cpu_busy: Option<usize>,
    cpu_wait: Option<usize>,
    gpu_time: Option<usize>,
    gpu_busy: Option<usize>,
    displayed_time: Option<usize>,
    present_mode: Option<usize>,
    present_runtime: Option<usize>,
    etw_events_lost: Option<usize>,
    etw_buffers_lost: Option<usize>,
    overflowed_presents: Option<usize>,
}

impl Layout {
    fn from_headers(headers: &StringRecord) -> Result<Self, String> {
        Ok(Self {
            process_id: required_index(headers, "ProcessID")?,
            swapchain: required_index(headers, "SwapChainAddress")?,
            qpc: required_index(headers, "CPUStartQPC")?,
            cpu_frame_time: required_index(headers, "FrameTime")?,
            cpu_busy: optional_index(headers, "CPUBusy"),
            cpu_wait: optional_index(headers, "CPUWait"),
            gpu_time: optional_index(headers, "GPUTime"),
            gpu_busy: optional_index(headers, "GPUBusy"),
            displayed_time: optional_index(headers, "DisplayedTime"),
            present_mode: optional_index(headers, "PresentMode"),
            present_runtime: optional_index(headers, "PresentRuntime"),
            etw_events_lost: optional_index(headers, "EtwEventsLost"),
            etw_buffers_lost: optional_index(headers, "EtwBuffersLost"),
            overflowed_presents: optional_index(headers, "OverflowedPresents"),
        })
    }

    fn parse_row(&self, row: &StringRecord, expected_pid: u32) -> Result<ParsedRow, String> {
        let pid = field(row, self.process_id)?
            .parse::<u32>()
            .map_err(|_| "invalid ProcessID".to_owned())?;
        if pid != expected_pid {
            return Err("unexpected ProcessID".to_owned());
        }

        let qpc = field(row, self.qpc)?
            .parse::<u64>()
            .map_err(|_| "invalid CPUStartQPC".to_owned())?;
        let cpu_frame_time_us = parse_required_ms(field(row, self.cpu_frame_time)?)?;

        Ok(ParsedRow {
            sample: FrameSample {
                swapchain: parse_swapchain(field(row, self.swapchain)?)?,
                qpc,
                cpu_frame_time_us,
                cpu_busy_us: parse_optional_ms(row, self.cpu_busy),
                cpu_wait_us: parse_optional_ms(row, self.cpu_wait),
                gpu_time_us: parse_optional_ms(row, self.gpu_time),
                gpu_busy_us: parse_optional_ms(row, self.gpu_busy),
                displayed_time_us: parse_optional_ms(row, self.displayed_time),
            },
            present_mode: optional_text(row, self.present_mode),
            present_runtime: optional_text(row, self.present_runtime),
        })
    }
}

fn required_index(headers: &StringRecord, name: &str) -> Result<usize, String> {
    optional_index(headers, name)
        .ok_or_else(|| format!("PresentMon output is missing required column {name}"))
}

fn optional_index(headers: &StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|header| header == name)
}

fn field(row: &StringRecord, index: usize) -> Result<&str, String> {
    row.get(index)
        .ok_or_else(|| format!("CSV row is missing column index {index}"))
}

fn parse_required_ms(value: &str) -> Result<u64, String> {
    parse_ms(value).ok_or_else(|| format!("invalid required millisecond value {value:?}"))
}

fn parse_optional_ms(row: &StringRecord, index: Option<usize>) -> Option<u64> {
    index.and_then(|index| row.get(index)).and_then(parse_ms)
}

fn parse_optional_u64(row: &StringRecord, index: Option<usize>) -> Option<u64> {
    index
        .and_then(|index| row.get(index))
        .and_then(|value| value.parse::<u64>().ok())
}

fn max_optional(current: Option<u64>, next: Option<u64>) -> Option<u64> {
    match (current, next) {
        (Some(current), Some(next)) => Some(current.max(next)),
        (Some(current), None) => Some(current),
        (None, Some(next)) => Some(next),
        (None, None) => None,
    }
}

fn parse_ms(value: &str) -> Option<u64> {
    if value.is_empty() || value.eq_ignore_ascii_case("NA") {
        return None;
    }

    let milliseconds = value.parse::<f64>().ok()?;
    if !milliseconds.is_finite() || milliseconds < 0.0 {
        return None;
    }

    Some((milliseconds * 1_000.0).round() as u64)
}

fn optional_text(row: &StringRecord, index: Option<usize>) -> Option<String> {
    let value = index.and_then(|index| row.get(index))?;
    (!value.is_empty() && !value.eq_ignore_ascii_case("NA")).then(|| value.to_owned())
}

fn parse_swapchain(value: &str) -> Result<u64, String> {
    let stripped = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);

    if stripped.is_empty() {
        return Err("empty SwapChainAddress".to_owned());
    }

    u64::from_str_radix(stripped, 16).map_err(|_| format!("invalid SwapChainAddress {value:?}"))
}

fn validate_candidate(path: PathBuf) -> Result<PathBuf, String> {
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!(
            "PresentMon executable does not exist: {}",
            path.display()
        ))
    }
}

fn find_in_directory(directory: &Path) -> Option<PathBuf> {
    let exact = directory.join(if cfg!(windows) {
        "PresentMon.exe"
    } else {
        "PresentMon"
    });
    if exact.is_file() {
        return Some(exact);
    }

    let mut candidates = fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        let lower = name.to_ascii_lowercase();
                        lower.starts_with("presentmon")
                            && !lower.contains("service")
                            && !lower.contains("application")
                            && !lower.contains("control")
                            && (lower.ends_with(".exe") || !cfg!(windows))
                    })
        })
        .collect::<Vec<_>>();

    candidates.sort();
    candidates.pop()
}

fn unix_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[cfg(test)]
mod tests {
    use super::{parse_presentmon_csv, validate_help_contract};

    const SUPPORTED_HELP: &str = "
PresentMon
--process_id
--output_stdout
--no_console_stats
--qpc_time
--v2_metrics
--no_track_input
--no_track_gpu
--timed
--terminate_after_timed
--terminate_on_proc_exit
--session_name
--track_etw_status
";

    #[test]
    fn accepts_required_presentmon_cli_surface() {
        validate_help_contract(SUPPORTED_HELP).expect("required CLI surface should pass");
    }

    #[test]
    fn rejects_incomplete_presentmon_cli_surface() {
        let error = validate_help_contract("--process_id --output_stdout")
            .expect_err("incomplete CLI surface should fail");

        assert!(error.contains("--v2_metrics"));
        assert!(error.contains("--qpc_time"));
    }

    #[test]
    fn parses_v2_qpc_csv_and_selects_primary_stream() {
        let csv = concat!(
            "Application,ProcessID,SwapChainAddress,PresentRuntime,PresentMode,CPUStartQPC,FrameTime,CPUBusy,CPUWait,GPUTime,GPUBusy,DisplayedTime\n",
            "javaw.exe,42,0xABC,Other,Composed: Flip,1000,10.000,7.000,3.000,8.000,6.000,10.000\n",
            "javaw.exe,42,0xABC,Other,Composed: Flip,2000,11.000,8.000,3.000,9.000,7.000,NA\n",
            "javaw.exe,42,0xDEF,Other,Composed: Flip,2100,40.000,30.000,10.000,20.000,15.000,40.000\n"
        );

        let parsed =
            parse_presentmon_csv(csv.as_bytes(), 42, 1_000, 120).expect("fixture should parse");

        assert_eq!(parsed.rows_rejected, 0);
        assert_eq!(parsed.metrics.summary.primary_swapchain, "0xABC");
        assert_eq!(parsed.metrics.summary.frames, 2);
        assert_eq!(parsed.metrics.summary.not_displayed_frames, Some(1));
        assert_eq!(
            parsed
                .metrics
                .summary
                .displayed_time
                .as_ref()
                .expect("display timing summary")
                .samples,
            1
        );
        assert!(parsed.gpu_metrics_available);
        assert!(parsed.display_metrics_available);
        assert!(!parsed.etw_status_available);
    }

    #[test]
    fn parses_etw_quality_counters_and_keeps_high_water_marks() {
        let csv = concat!(
            "ProcessID,SwapChainAddress,CPUStartQPC,FrameTime,EtwEventsLost,EtwBuffersLost,OverflowedPresents\n",
            "42,0x1,1000,10.0,0,0,0\n",
            "42,0x1,2000,10.0,2,1,3\n",
            "42,0x1,3000,10.0,1,0,2\n"
        );

        let parsed =
            parse_presentmon_csv(csv.as_bytes(), 42, 1_000, 120).expect("fixture should parse");

        assert!(parsed.etw_status_available);
        assert_eq!(parsed.etw_events_lost, Some(2));
        assert_eq!(parsed.etw_buffers_lost, Some(1));
        assert_eq!(parsed.overflowed_presents, Some(3));
    }

    #[test]
    fn parses_official_v2_style_columns_with_na_display_metrics() {
        let csv = concat!(
            "\u{feff}Application,ProcessID,SwapChainAddress,PresentRuntime,SyncInterval,PresentFlags,AllowsTearing,PresentMode,FrameType,CPUStartQPC,FrameTime,CPUBusy,CPUWait,GPULatency,GPUTime,GPUBusy,GPUWait,VideoBusy,DisplayLatency,DisplayedTime,AnimationError,AnimationTime,MsFlipDelay,AllInputToPhotonLatency,ClickToPhotonLatency,InstrumentedLatency\n",
            "javaw.exe,42,0x2A70D2CAC00,Other,0,0,0,Composed: Flip,Application,2466961521260,11.0804,10.5535,0.5269,1.1731,10.1685,1.5667,8.6018,0.0000,NA,NA,NA,NA,NA,NA,NA,NA\n",
            "javaw.exe,42,0x2A70D2CAC00,Other,0,0,0,Composed: Flip,Application,2466961632064,11.0437,10.4814,0.5623,0.2612,10.9128,0.5496,10.3632,0.0000,28.1718,16.6267,NA,NA,NA,NA,NA,NA\n"
        );

        let parsed = parse_presentmon_csv(csv.as_bytes(), 42, 10_000_000, 120)
            .expect("official-style v2 fixture should parse");

        assert_eq!(parsed.rows_rejected, 0);
        assert_eq!(parsed.metrics.summary.primary_swapchain, "0x2A70D2CAC00");
        assert_eq!(parsed.metrics.summary.frames, 2);
        assert_eq!(parsed.metrics.summary.not_displayed_frames, Some(1));
        assert!(parsed.gpu_metrics_available);
        assert!(parsed.display_metrics_available);
        assert_eq!(parsed.metrics.summary.cpu_frame_time.samples, 2);
    }

    #[test]
    fn absent_display_column_is_capability_absence_not_dropped_frames() {
        let csv = concat!(
            "ProcessID,SwapChainAddress,CPUStartQPC,FrameTime\n",
            "42,0x1,1000,10.0\n",
            "42,0x1,2000,10.0\n"
        );

        let parsed =
            parse_presentmon_csv(csv.as_bytes(), 42, 1_000, 120).expect("fixture should parse");

        assert!(!parsed.display_metrics_available);
        assert!(parsed.metrics.summary.displayed_time.is_none());
        assert!(parsed.metrics.summary.not_displayed_frames.is_none());
    }

    #[test]
    fn rejects_invalid_swapchain_instead_of_merging_it_into_zero() {
        let csv = concat!(
            "ProcessID,SwapChainAddress,CPUStartQPC,FrameTime\n",
            "42,not-a-pointer,1000,10.0\n",
            "42,0x1,2000,10.0\n"
        );

        let parsed = parse_presentmon_csv(csv.as_bytes(), 42, 1_000, 120)
            .expect("fixture should retain the valid row");

        assert_eq!(parsed.rows_rejected, 1);
        assert_eq!(parsed.metrics.summary.frames, 1);
        assert_eq!(parsed.metrics.summary.primary_swapchain, "0x1");
    }

    #[test]
    fn rejects_other_process_rows() {
        let csv = concat!(
            "ProcessID,SwapChainAddress,CPUStartQPC,FrameTime\n",
            "99,0x1,1000,10.0\n",
            "42,0x1,2000,10.0\n"
        );

        let parsed = parse_presentmon_csv(csv.as_bytes(), 42, 1_000, 120)
            .expect("fixture should retain one valid row");

        assert_eq!(parsed.rows_rejected, 1);
        assert_eq!(parsed.metrics.summary.frames, 1);
    }
}
