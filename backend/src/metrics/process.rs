//! Process metrics read from `/proc` at each scrape (Linux only; elsewhere, e.g. on a Windows dev
//! machine, nothing is exported).

use prometheus_client::{collector::Collector, encoding::DescriptorEncoder};

#[derive(Debug)]
pub struct ProcessCollector;

impl Collector for ProcessCollector {
    #[cfg(target_os = "linux")]
    fn encode(&self, mut encoder: DescriptorEncoder) -> Result<(), std::fmt::Error> {
        use prometheus_client::{
            encoding::EncodeMetric,
            metrics::{counter::ConstCounter, gauge::ConstGauge},
        };

        if let Some(seconds) = linux::cpu_seconds() {
            let counter = ConstCounter::new(seconds);
            let metric = encoder.encode_descriptor(
                "process_cpu_seconds",
                "Total user and system CPU time spent, in seconds",
                None,
                counter.metric_type(),
            )?;
            counter.encode(metric)?;
        }
        if let Some(bytes) = linux::resident_memory_bytes() {
            let gauge = ConstGauge::new(bytes);
            let metric = encoder.encode_descriptor(
                "process_resident_memory_bytes",
                "Resident memory size, in bytes",
                None,
                gauge.metric_type(),
            )?;
            gauge.encode(metric)?;
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn encode(&self, _encoder: DescriptorEncoder) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    /// Kernel clock ticks per second (`sysconf(_SC_CLK_TCK)`), 100 on every mainstream kernel.
    const CLOCK_TICKS: f64 = 100.0;

    /// `utime + stime` from `/proc/self/stat`.
    pub fn cpu_seconds() -> Option<f64> {
        let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
        // The command name (2nd field) may contain spaces: count fields after its closing paren.
        let rest = stat.get(stat.rfind(')')? + 2..)?;
        let mut fields = rest.split_whitespace();
        // `rest` starts at field 3 (state); utime and stime are fields 14 and 15.
        let utime: u64 = fields.nth(11)?.parse().ok()?;
        let stime: u64 = fields.next()?.parse().ok()?;
        Some((utime + stime) as f64 / CLOCK_TICKS)
    }

    /// `VmRSS` from `/proc/self/status`.
    pub fn resident_memory_bytes() -> Option<i64> {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let kb: i64 = status
            .lines()
            .find_map(|l| l.strip_prefix("VmRSS:"))?
            .trim()
            .trim_end_matches("kB")
            .trim()
            .parse()
            .ok()?;
        Some(kb * 1024)
    }
}
