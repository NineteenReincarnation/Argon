use serde::Serialize;
use std::collections::{BTreeMap, HashMap, VecDeque};

const HISTOGRAM_BUCKET_US: u64 = 100;
const HISTOGRAM_MAX_US: u64 = 5_000_000;
const HISTOGRAM_BUCKETS: usize = (HISTOGRAM_MAX_US / HISTOGRAM_BUCKET_US) as usize + 1;

#[derive(Debug, Clone, Copy)]
pub struct FrameSample {
    pub swapchain: u64,
    pub qpc: u64,
    pub frame_time_us: u64,
    pub cpu_busy_us: Option<u64>,
    pub cpu_wait_us: Option<u64>,
    pub gpu_time_us: Option<u64>,
    pub gpu_busy_us: Option<u64>,
    pub displayed_time_us: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct FrameSummary {
    pub primary_swapchain: String,
    pub frames: u64,
    pub all_stream_frames: u64,
    pub stream_count: usize,
    pub primary_frame_share: f64,
    pub average_frame_ms: f64,
    pub average_fps: f64,
    pub p50_frame_ms: f64,
    pub p95_frame_ms: f64,
    pub p99_frame_ms: f64,
    pub p999_frame_ms: f64,
    pub one_percent_low_fps_approx: f64,
    pub point_one_percent_low_fps_approx: f64,
    pub frames_over_16_67_ms: u64,
    pub frames_over_33_33_ms: u64,
    pub frames_over_50_ms: u64,
    pub frames_over_100_ms: u64,
    pub undisplayed_frames: u64,
    pub present_modes: BTreeMap<String, u64>,
    pub present_runtimes: BTreeMap<String, u64>,
}

pub struct CaptureAccumulator {
    ring_window_ticks: u64,
    latest_qpc: u64,
    ring: VecDeque<FrameSample>,
    streams: HashMap<u64, StreamState>,
    all_stream_frames: u64,
}

struct StreamState {
    stats: FrameStats,
    present_modes: BTreeMap<String, u64>,
    present_runtimes: BTreeMap<String, u64>,
    undisplayed_frames: u64,
}

struct FrameStats {
    count: u64,
    sum_us: u128,
    histogram: Vec<u64>,
    overflow_count: u64,
    overflow_sum_us: u128,
    max_us: u64,
    over_16_67_ms: u64,
    over_33_33_ms: u64,
    over_50_ms: u64,
    over_100_ms: u64,
}

pub struct CaptureMetrics {
    pub summary: FrameSummary,
    pub recent_frames: Vec<FrameSample>,
}

impl CaptureAccumulator {
    pub fn new(qpc_frequency_hz: u64, ring_seconds: u64) -> Self {
        Self {
            ring_window_ticks: qpc_frequency_hz.saturating_mul(ring_seconds),
            latest_qpc: 0,
            ring: VecDeque::new(),
            streams: HashMap::new(),
            all_stream_frames: 0,
        }
    }

    pub fn observe(
        &mut self,
        sample: FrameSample,
        present_mode: Option<&str>,
        present_runtime: Option<&str>,
    ) {
        self.latest_qpc = self.latest_qpc.max(sample.qpc);
        self.all_stream_frames += 1;

        let stream = self
            .streams
            .entry(sample.swapchain)
            .or_insert_with(StreamState::new);
        stream.stats.observe(sample.frame_time_us);

        if sample.displayed_time_us.is_none() {
            stream.undisplayed_frames += 1;
        }

        if let Some(mode) = present_mode.filter(|value| !value.is_empty()) {
            *stream.present_modes.entry(mode.to_owned()).or_insert(0) += 1;
        }

        if let Some(runtime) = present_runtime.filter(|value| !value.is_empty()) {
            *stream
                .present_runtimes
                .entry(runtime.to_owned())
                .or_insert(0) += 1;
        }

        if self.ring_window_ticks > 0 {
            let cutoff = self.latest_qpc.saturating_sub(self.ring_window_ticks);
            if sample.qpc >= cutoff {
                self.ring.push_back(sample);
            }
            while self.ring.front().is_some_and(|oldest| oldest.qpc < cutoff) {
                self.ring.pop_front();
            }
        }
    }

    pub fn finish(self) -> Result<CaptureMetrics, String> {
        let (&primary_swapchain, primary) = self
            .streams
            .iter()
            .max_by_key(|(_, stream)| stream.stats.count)
            .ok_or_else(|| "PresentMon returned no usable frame samples".to_owned())?;

        let frames = primary.stats.count;
        let primary_frame_share = if self.all_stream_frames == 0 {
            0.0
        } else {
            frames as f64 / self.all_stream_frames as f64
        };

        let summary = FrameSummary {
            primary_swapchain: format!("0x{primary_swapchain:X}"),
            frames,
            all_stream_frames: self.all_stream_frames,
            stream_count: self.streams.len(),
            primary_frame_share,
            average_frame_ms: primary.stats.average_us() / 1_000.0,
            average_fps: fps_from_us(primary.stats.average_us()),
            p50_frame_ms: primary.stats.percentile_us(0.50) / 1_000.0,
            p95_frame_ms: primary.stats.percentile_us(0.95) / 1_000.0,
            p99_frame_ms: primary.stats.percentile_us(0.99) / 1_000.0,
            p999_frame_ms: primary.stats.percentile_us(0.999) / 1_000.0,
            one_percent_low_fps_approx: primary.stats.tail_low_fps(0.01),
            point_one_percent_low_fps_approx: primary.stats.tail_low_fps(0.001),
            frames_over_16_67_ms: primary.stats.over_16_67_ms,
            frames_over_33_33_ms: primary.stats.over_33_33_ms,
            frames_over_50_ms: primary.stats.over_50_ms,
            frames_over_100_ms: primary.stats.over_100_ms,
            undisplayed_frames: primary.undisplayed_frames,
            present_modes: primary.present_modes.clone(),
            present_runtimes: primary.present_runtimes.clone(),
        };

        let recent_frames = self
            .ring
            .into_iter()
            .filter(|sample| sample.swapchain == primary_swapchain)
            .collect();

        Ok(CaptureMetrics {
            summary,
            recent_frames,
        })
    }

}

impl StreamState {
    fn new() -> Self {
        Self {
            stats: FrameStats::new(),
            present_modes: BTreeMap::new(),
            present_runtimes: BTreeMap::new(),
            undisplayed_frames: 0,
        }
    }
}

impl FrameStats {
    fn new() -> Self {
        Self {
            count: 0,
            sum_us: 0,
            histogram: vec![0; HISTOGRAM_BUCKETS],
            overflow_count: 0,
            overflow_sum_us: 0,
            max_us: 0,
            over_16_67_ms: 0,
            over_33_33_ms: 0,
            over_50_ms: 0,
            over_100_ms: 0,
        }
    }

    fn observe(&mut self, frame_us: u64) {
        self.count += 1;
        self.sum_us += frame_us as u128;
        self.max_us = self.max_us.max(frame_us);

        if frame_us > HISTOGRAM_MAX_US {
            self.overflow_count += 1;
            self.overflow_sum_us += frame_us as u128;
        } else {
            let index = (frame_us / HISTOGRAM_BUCKET_US) as usize;
            self.histogram[index] += 1;
        }

        self.over_16_67_ms += u64::from(frame_us > 16_670);
        self.over_33_33_ms += u64::from(frame_us > 33_330);
        self.over_50_ms += u64::from(frame_us > 50_000);
        self.over_100_ms += u64::from(frame_us > 100_000);
    }

    fn average_us(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.sum_us as f64 / self.count as f64
    }

    fn percentile_us(&self, percentile: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let rank = ((self.count as f64 * percentile).ceil() as u64).clamp(1, self.count);
        let histogram_count = self.count - self.overflow_count;

        if rank > histogram_count {
            return self.max_us as f64;
        }

        let mut seen = 0_u64;
        for (index, count) in self.histogram.iter().enumerate() {
            seen += *count;
            if seen >= rank {
                return (index as u64 * HISTOGRAM_BUCKET_US) as f64;
            }
        }

        self.max_us as f64
    }

    fn tail_low_fps(&self, fraction: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let target = ((self.count as f64 * fraction).ceil() as u64).max(1);
        let mut remaining = target;
        let mut selected = 0_u64;
        let mut sum_us = 0_f64;

        if self.overflow_count > 0 {
            let take = remaining.min(self.overflow_count);
            let average_overflow = self.overflow_sum_us as f64 / self.overflow_count as f64;
            sum_us += average_overflow * take as f64;
            selected += take;
            remaining -= take;
        }

        if remaining > 0 {
            for (index, count) in self.histogram.iter().enumerate().rev() {
                if *count == 0 {
                    continue;
                }

                let take = remaining.min(*count);
                let bucket_us = index as u64 * HISTOGRAM_BUCKET_US;
                sum_us += bucket_us as f64 * take as f64;
                selected += take;
                remaining -= take;

                if remaining == 0 {
                    break;
                }
            }
        }

        if selected == 0 {
            0.0
        } else {
            fps_from_us(sum_us / selected as f64)
        }
    }
}

fn fps_from_us(frame_us: f64) -> f64 {
    if frame_us <= 0.0 {
        0.0
    } else {
        1_000_000.0 / frame_us
    }
}

#[cfg(windows)]
pub fn qpc_clock_anchor() -> Result<QpcAnchor, String> {
    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn QueryPerformanceCounter(performance_count: *mut i64) -> i32;
        fn QueryPerformanceFrequency(frequency: *mut i64) -> i32;
    }

    let mut frequency = 0_i64;
    let mut counter = 0_i64;

    let frequency_ok = unsafe { QueryPerformanceFrequency(&mut frequency) };
    let counter_ok = unsafe { QueryPerformanceCounter(&mut counter) };

    if frequency_ok == 0 || counter_ok == 0 || frequency <= 0 || counter < 0 {
        return Err("QueryPerformanceCounter/Frequency failed".to_owned());
    }

    Ok(QpcAnchor {
        qpc: counter as u64,
        frequency_hz: frequency as u64,
    })
}

#[cfg(not(windows))]
pub fn qpc_clock_anchor() -> Result<QpcAnchor, String> {
    Err("QPC is only available on Windows".to_owned())
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct QpcAnchor {
    pub qpc: u64,
    pub frequency_hz: u64,
}

#[cfg(test)]
mod tests {
    use super::{CaptureAccumulator, FrameSample};

    fn sample(swapchain: u64, qpc: u64, frame_us: u64) -> FrameSample {
        FrameSample {
            swapchain,
            qpc,
            frame_time_us: frame_us,
            cpu_busy_us: None,
            cpu_wait_us: None,
            gpu_time_us: None,
            gpu_busy_us: None,
            displayed_time_us: Some(frame_us),
        }
    }

    #[test]
    fn ring_evicts_old_frames() {
        let mut capture = CaptureAccumulator::new(1_000, 2);
        capture.observe(sample(1, 1_000, 10_000), None, None);
        capture.observe(sample(1, 2_000, 10_000), None, None);
        capture.observe(sample(1, 4_001, 10_000), None, None);

        let result = capture.finish().expect("capture should contain frames");
        assert_eq!(result.recent_frames.len(), 1);
        assert_eq!(result.recent_frames[0].qpc, 4_001);
    }

    #[test]
    fn dominant_swapchain_is_selected() {
        let mut capture = CaptureAccumulator::new(1_000, 10);

        for qpc in 1..=100 {
            capture.observe(
                sample(10, qpc, 10_000),
                Some("Composed: Flip"),
                Some("DXGI"),
            );
        }
        for qpc in 101..=110 {
            capture.observe(sample(20, qpc, 40_000), None, None);
        }

        let result = capture.finish().expect("capture should contain frames");
        assert_eq!(result.summary.primary_swapchain, "0xA");
        assert_eq!(result.summary.frames, 100);
        assert_eq!(result.summary.stream_count, 2);
        assert!(result.summary.primary_frame_share > 0.90);
        assert!(result.summary.p99_frame_ms >= 9.9);
    }
}
