mod metrics;
mod presentmon;
mod process;
mod report;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "argon-probe")]
#[command(version)]
#[command(about = "Standalone performance evidence collector for Argon development")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List Java processes that look like Minecraft clients.
    List,
    /// Capture a bounded PresentMon frame session and write a report ZIP.
    Capture {
        /// Explicit target PID. If omitted, Probe attempts conservative auto-detection.
        #[arg(long)]
        pid: Option<u32>,

        /// Path to an official PresentMon console executable.
        #[arg(long)]
        presentmon: Option<PathBuf>,

        /// Capture duration in seconds. P0 intentionally uses bounded timed captures.
        #[arg(long, default_value_t = 60)]
        duration_seconds: u64,

        /// Recent raw frame history retained in memory.
        #[arg(long, default_value_t = 120)]
        ring_seconds: u64,

        /// Directory where the report ZIP is written.
        #[arg(long, default_value = ".")]
        output: PathBuf,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("argon-probe: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_candidates(),
        Commands::Capture {
            pid,
            presentmon,
            duration_seconds,
            ring_seconds,
            output,
        } => capture(pid, presentmon, duration_seconds, ring_seconds, output),
    }
}

fn list_candidates() -> Result<(), Box<dyn std::error::Error>> {
    let candidates = process::discover_minecraft_candidates();

    if candidates.is_empty() {
        println!("No likely Minecraft Java client processes found.");
        return Ok(());
    }

    println!("Likely Minecraft Java processes:");
    for candidate in candidates {
        println!(
            "  PID {:>6}  score {:>4}  {}  [{}]",
            candidate.target.pid,
            candidate.score,
            candidate.target.name,
            candidate.reasons.join(", ")
        );
    }

    Ok(())
}

fn capture(
    pid: Option<u32>,
    presentmon_override: Option<PathBuf>,
    duration_seconds: u64,
    ring_seconds: u64,
    output: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(windows) {
        return Err("Argon Probe P0 capture is Windows-only".into());
    }
    if duration_seconds == 0 {
        return Err("--duration-seconds must be greater than zero in P0".into());
    }
    if ring_seconds == 0 {
        return Err("--ring-seconds must be greater than zero".into());
    }

    let target = process::select_target(pid)?;
    let presentmon_path = presentmon::find_presentmon(presentmon_override.as_deref())?;
    let qpc_anchor = metrics::qpc_clock_anchor()?;
    let started_unix_ms = report::unix_millis();
    let environment = report::environment_snapshot(&target);

    println!(
        "Argon Probe P0: PID {} ({}) for {} second(s)",
        target.pid, target.name, duration_seconds
    );
    println!(
        "PresentMon: {}",
        presentmon_path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default()
    );

    let capture = presentmon::capture(
        &presentmon_path,
        target.pid,
        duration_seconds,
        ring_seconds,
        qpc_anchor.frequency_hz,
    )?;

    let capabilities = report::capability_snapshot(&presentmon_path, &capture, qpc_anchor);
    let quality = report::quality_snapshot(&capture);
    let report_path = report::write_report(
        &output,
        started_unix_ms,
        qpc_anchor,
        &environment,
        &capabilities,
        &quality,
        &capture.metrics,
    )?;

    let summary = &capture.metrics.summary;
    println!();
    println!("Capture quality: {}", quality.capture_quality);
    println!("Primary stream: {}", summary.primary_swapchain);
    println!("Frames: {}", summary.frames);
    println!("Median frame time: {:.3} ms", summary.p50_frame_ms);
    println!("P99 frame time: {:.3} ms", summary.p99_frame_ms);
    println!("Report: {}", report_path.display());

    Ok(())
}
