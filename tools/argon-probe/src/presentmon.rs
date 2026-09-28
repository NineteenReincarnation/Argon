use crate::metrics::{CaptureAccumulator, CaptureMetrics, FrameSample};
use csv::{ReaderBuilder, StringRecord};
use serde::Serialize;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Serialize)]
pub struct PresentMonCapture {
    pub metrics: CaptureMetrics,
    pub backend_version: Option<String>,
    pub rows_rejected: u64,
    pub exit_code: Option<i32>,
    pub gpu_metrics_available: bool,
    pub display_metrics_available: bool,
}

pub fn find_presentmon(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = explicit {
        return validate_candidate(path.to_path_buf());
    }

    if let Some(path) = env::var_os("ARGON_PROBE_PRESENTMON") {
        if let Ok(candidate) = validate_candidate(PathBuf::from(path)) {
            return Ok(candidate);
        }
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
        "PresentMon was not found. Pass --presentmon PATH, set ARGON_PROBE_PRESENTMON, or place an official PresentMon executable next to argon-probe / on PATH."
            .to_owned(),
    )
}

pub fn presentmon_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("--help").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    stdout
        .lines()
        .chain(stderr.lines())
        .map(str::trim)
        .find(|line| line.to_ascii_lowercase().contains("presentmon"))
        .map(ToOwned::to_owned)
}

pub fn capture(
    presentmon: &Path,
    pid: u32,
    duration_seconds: u64,
    ring_seconds: u64,
    qpc_frequency_hz: u64,
) -> Result<PresentMonCapture, Box<dyn std::error::Error>> {
    let session_name = format!("ArgonProbe-{pid}-{}", unix_millis());

    let mut child = Command::new(presentmon)
        .args([
            OsStr::new("--process_id"),
            OsStr::new(&pid.to_string()),
            OsStr::new("--output_stdout"),
            OsStr::new("--no_console_stats"),
            OsStr::new("--qpc_time"),
            OsStr::new("--v2_metrics"),
            OsStr::new("--no_track_input"),
            OsStr::new("--timed"),
            OsStr::new(&duration_seconds.to_string()),
            OsStr::new("--terminate_after_timed"),
            OsStr::new("--terminate_on_proc_exit"),
            OsStr::new("--session_name"),
            OsStr::new(&session_name),
        ])
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
        backend_version: presentmon_version(presentmon),
        rows_rejected: parsed.rows_rejected,
        exit_code: status.code(),
        gpu_metrics_available: parsed.gpu_metrics_available,
        display_metrics_available: parsed.display_metrics_available,
    })
}

struct ParsedCapture {
    metrics: CaptureMetrics,
    rows_rejected: u64,
    gpu_metrics_available: bool,
    display_metrics_available: bool,
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

    let mut accumulator = CaptureAccumulator::new(qpc_frequency_hz, ring_seconds);
    let mut rows_rejected = 0_u64;

    for row in csv.records() {
        let row = match row {
            Ok(row) => row,
            Err(_) => {
                rows_rejected += 1;
                continue;
            }
        };

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
        display_metrics_available: layout.displayed_time.is_some(),
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
    frame_time: usize,
    cpu_busy: Option<usize>,
    cpu_wait: Option<usize>,
    gpu_time: Option<usize>,
    gpu_busy: Option<usize>,
    displayed_time: Option<usize>,
    present_mode: Option<usize>,
    present_runtime: Option<usize>,
}

impl Layout {
    fn from_headers(headers: &StringRecord) -> Result<Self, String> {
        Ok(Self {
            process_id: required_index(headers, "ProcessID")?,
            swapchain: required_index(headers, "SwapChainAddress")?,
            qpc: required_index(headers, "CPUStartQPC")?,
            frame_time: required_index(headers, "FrameTime")?,
            cpu_busy: optional_index(headers, "CPUBusy"),
            cpu_wait: optional_index(headers, "CPUWait"),
            gpu_time: optional_index(headers, "GPUTime"),
            gpu_busy: optional_index(headers, "GPUBusy"),
            displayed_time: optional_index(headers, "DisplayedTime"),
            present_mode: optional_index(headers, "PresentMode"),
            present_runtime: optional_index(headers, "PresentRuntime"),
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
        let frame_time_us = parse_required_ms(field(row, self.frame_time)?)?;

        Ok(ParsedRow {
            sample: FrameSample {
                swapchain: parse_swapchain(field(row, self.swapchain)?),
                qpc,
                frame_time_us,
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
    optional_index(headers, name).ok_or_else(|| format!("PresentMon output is missing required column {name}"))
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

fn parse_swapchain(value: &str) -> u64 {
    let stripped = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);
    u64::from_str_radix(stripped, 16).unwrap_or(0)
}

fn validate_candidate(path: PathBuf) -> Result<PathBuf, String> {
    if path.is_file() {
        Ok(path)
    } else {
        Err(format!("PresentMon executable does not exist: {}", path.display()))
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
    use super::parse_presentmon_csv;

    #[test]
    fn parses_v2_qpc_csv_and_selects_primary_stream() {
        let csv = concat!(
            "Application,ProcessID,SwapChainAddress,PresentRuntime,PresentMode,CPUStartQPC,FrameTime,CPUBusy,CPUWait,GPUTime,GPUBusy,DisplayedTime\n",
            "javaw.exe,42,0xABC,Other,Composed: Flip,1000,10.000,7.000,3.000,8.000,6.000,10.000\n",
            "javaw.exe,42,0xABC,Other,Composed: Flip,2000,11.000,8.000,3.000,9.000,7.000,NA\n",
            "javaw.exe,42,0xDEF,Other,Composed: Flip,2100,40.000,30.000,10.000,20.000,15.000,40.000\n"
        );

        let parsed = parse_presentmon_csv(csv.as_bytes(), 42, 1_000, 120)
            .expect("fixture should parse");

        assert_eq!(parsed.rows_rejected, 0);
        assert_eq!(parsed.metrics.summary.primary_swapchain, "0xABC");
        assert_eq!(parsed.metrics.summary.frames, 2);
        assert_eq!(parsed.metrics.summary.undisplayed_frames, 1);
        assert!(parsed.gpu_metrics_available);
        assert!(parsed.display_metrics_available);
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
