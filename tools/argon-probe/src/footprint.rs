use serde::Serialize;
use sysinfo::{Pid, System};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ProcessResourceSnapshot {
    pub cpu_time_ms: u64,
    pub resident_memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub total_read_bytes: u64,
    pub total_written_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProbeFootprint {
    pub measurement_scope: &'static str,
    pub cpu_time_ms: u64,
    pub resident_memory_start_bytes: u64,
    pub resident_memory_end_bytes: u64,
    pub resident_memory_sampled_high_water_bytes: u64,
    pub virtual_memory_start_bytes: u64,
    pub virtual_memory_end_bytes: u64,
    pub virtual_memory_sampled_high_water_bytes: u64,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

pub struct ProbeFootprintTracker {
    start: ProcessResourceSnapshot,
}

impl ProbeFootprintTracker {
    pub fn start() -> Result<Self, String> {
        Ok(Self {
            start: snapshot_current_process()?,
        })
    }

    pub fn finish(self) -> Result<ProbeFootprint, String> {
        let end = snapshot_current_process()?;
        Ok(ProbeFootprint::between(self.start, end))
    }
}

impl ProbeFootprint {
    fn between(start: ProcessResourceSnapshot, end: ProcessResourceSnapshot) -> Self {
        Self {
            measurement_scope: "presentmon_capture_window",
            cpu_time_ms: end.cpu_time_ms.saturating_sub(start.cpu_time_ms),
            resident_memory_start_bytes: start.resident_memory_bytes,
            resident_memory_end_bytes: end.resident_memory_bytes,
            resident_memory_sampled_high_water_bytes: start
                .resident_memory_bytes
                .max(end.resident_memory_bytes),
            virtual_memory_start_bytes: start.virtual_memory_bytes,
            virtual_memory_end_bytes: end.virtual_memory_bytes,
            virtual_memory_sampled_high_water_bytes: start
                .virtual_memory_bytes
                .max(end.virtual_memory_bytes),
            read_bytes: end.total_read_bytes.saturating_sub(start.total_read_bytes),
            written_bytes: end
                .total_written_bytes
                .saturating_sub(start.total_written_bytes),
        }
    }
}

fn snapshot_current_process() -> Result<ProcessResourceSnapshot, String> {
    let system = System::new_all();
    let pid = Pid::from_u32(std::process::id());
    let process = system
        .process(pid)
        .ok_or_else(|| format!("Argon Probe process {} is not visible to sysinfo", pid.as_u32()))?;
    let disk = process.disk_usage();

    Ok(ProcessResourceSnapshot {
        cpu_time_ms: process.accumulated_cpu_time(),
        resident_memory_bytes: process.memory(),
        virtual_memory_bytes: process.virtual_memory(),
        total_read_bytes: disk.total_read_bytes,
        total_written_bytes: disk.total_written_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::{ProbeFootprint, ProcessResourceSnapshot};

    #[test]
    fn footprint_uses_saturating_deltas_and_explicit_sampled_high_water() {
        let start = ProcessResourceSnapshot {
            cpu_time_ms: 100,
            resident_memory_bytes: 10_000,
            virtual_memory_bytes: 40_000,
            total_read_bytes: 1_000,
            total_written_bytes: 5_000,
        };
        let end = ProcessResourceSnapshot {
            cpu_time_ms: 145,
            resident_memory_bytes: 12_000,
            virtual_memory_bytes: 38_000,
            total_read_bytes: 1_600,
            total_written_bytes: 4_500,
        };

        let footprint = ProbeFootprint::between(start, end);

        assert_eq!(footprint.cpu_time_ms, 45);
        assert_eq!(footprint.resident_memory_sampled_high_water_bytes, 12_000);
        assert_eq!(footprint.virtual_memory_sampled_high_water_bytes, 40_000);
        assert_eq!(footprint.read_bytes, 600);
        assert_eq!(footprint.written_bytes, 0);
        assert_eq!(footprint.measurement_scope, "presentmon_capture_window");
    }
}
