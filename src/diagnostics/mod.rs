use serde::Serialize;
use std::{
    collections::VecDeque,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const EVENT_LOG_LIMIT: u64 = 256 * 1024;

pub struct EventLog {
    pub path: PathBuf,
    pub error: Option<String>,
}

impl EventLog {
    pub fn new(path: PathBuf) -> Self {
        Self { path, error: None }
    }

    pub fn record(&mut self, event: &str, details: serde_json::Value) {
        let mut line = serde_json::to_vec(&serde_json::json!({
            "unix_time_ms": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(),
            "process_id": std::process::id(),
            "event": event,
            "details": details,
        }))
        .expect("JSON values are serializable");
        line.push(b'\n');
        self.error = self.append(&line).err().map(|error| error.to_string());
    }

    fn append(&self, line: &[u8]) -> io::Result<()> {
        if let Some(parent) = self.path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        if file.metadata()?.len() + line.len() as u64 > EVENT_LOG_LIMIT {
            drop(file);
            fs::rename(&self.path, self.path.with_extension("previous.jsonl"))?;
            file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
        }
        file.write_all(line)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Summary {
    pub render_samples: usize,
    pub total_rendered_frames: u64,
    pub unique_telemetry_frames: u64,
    pub draw_cpu_ms_p50: Option<f64>,
    pub draw_cpu_ms_p95: Option<f64>,
    pub frame_interval_ms_p95: Option<f64>,
    pub frame_interval_ms_p50: Option<f64>,
    pub frame_interval_ms_p99: Option<f64>,
    pub effective_render_hz: Option<f64>,
    pub receive_to_present_submit_ms_p95: Option<f64>,
    pub gpu_ms_p95: Option<f64>,
    pub working_set_bytes: u64,
    pub process_cpu_percent: f64,
    pub device_recreations: u64,
    pub presentation_failures: u64,
    pub note: &'static str,
}

pub struct Diagnostics {
    draw_ms: VecDeque<f64>,
    interval_ms: VecDeque<f64>,
    latency_ms: VecDeque<f64>,
    gpu_ms: VecDeque<f64>,
    last_frame: Option<Instant>,
    last_packet: Option<Instant>,
    pub telemetry_frames: u64,
    pub total_frames: u64,
    pub device_recreations: u64,
    pub presentation_failures: u64,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            draw_ms: VecDeque::with_capacity(4096),
            interval_ms: VecDeque::with_capacity(4096),
            latency_ms: VecDeque::with_capacity(4096),
            gpu_ms: VecDeque::with_capacity(4096),
            last_frame: None,
            last_packet: None,
            telemetry_frames: 0,
            total_frames: 0,
            device_recreations: 0,
            presentation_failures: 0,
        }
    }
}

fn push(queue: &mut VecDeque<f64>, value: f64) {
    if queue.len() == 4096 {
        queue.pop_front();
    }
    queue.push_back(value);
}

pub fn percentile(values: &VecDeque<f64>, quantile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted: Vec<f64> = values.iter().copied().collect();
    sorted.sort_unstable_by(f64::total_cmp);
    let index = (quantile.clamp(0.0, 1.0) * (sorted.len() - 1) as f64).ceil() as usize;
    Some(sorted[index])
}

impl Diagnostics {
    pub fn frame(
        &mut self,
        start: Instant,
        draw: Duration,
        packet: Option<Instant>,
        continuously_active: bool,
    ) {
        self.total_frames += 1;
        push(&mut self.draw_ms, draw.as_secs_f64() * 1000.0);
        if let Some(last) = self.last_frame.filter(|_| continuously_active) {
            push(
                &mut self.interval_ms,
                start.saturating_duration_since(last).as_secs_f64() * 1000.0,
            );
        }
        self.last_frame = Some(start);
        if let Some(packet) = packet.filter(|v| self.last_packet != Some(*v)) {
            push(
                &mut self.latency_ms,
                Instant::now()
                    .saturating_duration_since(packet)
                    .as_secs_f64()
                    * 1000.0,
            );
            self.telemetry_frames += 1;
            self.last_packet = Some(packet);
        }
    }

    pub fn gpu_sample(&mut self, milliseconds: f64) {
        if milliseconds.is_finite() && milliseconds >= 0.0 {
            push(&mut self.gpu_ms, milliseconds);
        }
    }

    pub fn summary(&self, memory: u64, cpu: f64) -> Summary {
        Summary {
            render_samples: self.draw_ms.len(),
            total_rendered_frames: self.total_frames,
            unique_telemetry_frames: self.telemetry_frames,
            draw_cpu_ms_p50: percentile(&self.draw_ms, 0.5),
            draw_cpu_ms_p95: percentile(&self.draw_ms, 0.95),
            frame_interval_ms_p95: percentile(&self.interval_ms, 0.95),
            frame_interval_ms_p50: percentile(&self.interval_ms, 0.5),
            frame_interval_ms_p99: percentile(&self.interval_ms, 0.99),
            effective_render_hz: (!self.interval_ms.is_empty()).then(|| {
                1000.0 * self.interval_ms.len() as f64 / self.interval_ms.iter().sum::<f64>()
            }),
            receive_to_present_submit_ms_p95: percentile(&self.latency_ms, 0.95),
            gpu_ms_p95: percentile(&self.gpu_ms, 0.95),
            working_set_bytes: memory,
            process_cpu_percent: cpu,
            device_recreations: self.device_recreations,
            presentation_failures: self.presentation_failures,
            note: "Receive-to-present measures socket receive to Present submission, not optical latency. GPU time covers overlay command execution only. No in-game FPS impact measured.",
        }
    }

    pub fn export(&self, path: &Path, memory: u64, cpu: f64) -> Result<(), String> {
        let json =
            serde_json::to_vec_pretty(&self.summary(memory, cpu)).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_log_appends_timestamped_events_and_rotates_without_losing_the_last_event() {
        let directory = std::env::temp_dir().join(format!(
            "f1-events-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut log = EventLog::new(directory.join("session-events.jsonl"));
        for event in ["startup", "session_visibility"] {
            log.record(event, serde_json::json!({ "visible": true }));
            assert!(log.error.is_none(), "{:?}", log.error);
        }
        let text = fs::read_to_string(&log.path).unwrap();
        let lines: Vec<serde_json::Value> = text
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1]["event"], "session_visibility");
        assert_eq!(lines[1]["details"]["visible"], true);
        assert!(lines[1]["unix_time_ms"].as_u64().unwrap() > 0);
        for event in ["graphics_error", "shutdown"] {
            fs::write(&log.path, vec![b' '; EVENT_LOG_LIMIT as usize]).unwrap();
            log.record(event, serde_json::json!({ "visible": false }));
            assert!(log.error.is_none(), "{:?}", log.error);
            assert_eq!(
                fs::metadata(log.path.with_extension("previous.jsonl"))
                    .unwrap()
                    .len(),
                EVENT_LOG_LIMIT
            );
            let line: serde_json::Value =
                serde_json::from_slice(&fs::read(&log.path).unwrap()).unwrap();
            assert_eq!(line["event"], event);
        }
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn event_log_reports_io_errors_without_aborting_the_app() {
        let mut log = EventLog::new(PathBuf::new());
        log.record("startup", serde_json::json!({}));
        assert!(log.error.is_some());
    }
}
