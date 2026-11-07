use crate::{
    game_data::BatteryCatalog,
    settings::Settings,
    telemetry::{
        CapacitySource, Connection, ErsCapacity, ParseError, Snapshot, TelemetryState, demo, parse,
    },
};
use std::{
    io,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, RwLock, mpsc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub type SharedSnapshot = Arc<RwLock<Snapshot>>;

enum Command {
    Source {
        endpoint: SocketAddr,
        demo: bool,
        epoch: u64,
    },
    Stop,
}

pub struct Receiver {
    pub shared: SharedSnapshot,
    commands: mpsc::Sender<Command>,
    thread: Option<JoinHandle<()>>,
}

impl Receiver {
    pub fn start(
        settings: &Settings,
        demo: bool,
        wake: impl Fn() + Send + 'static,
    ) -> Result<Self, String> {
        let endpoint = settings.endpoint()?;
        let shared = Arc::new(RwLock::new(Snapshot::default()));
        let copy = shared.clone();
        let (commands, inbox) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("f1-udp".into())
            .spawn(move || receive_loop(endpoint, demo, copy, inbox, wake))
            .map_err(|e| e.to_string())?;
        Ok(Self {
            shared,
            commands,
            thread: Some(thread),
        })
    }

    pub fn source(&self, settings: &Settings, demo: bool) -> Result<(), String> {
        let endpoint = settings.endpoint()?;
        // Clear synchronously; a queued publication from the previous source must
        // never restore its ERS value or Straight Mode alert after this returns.
        let mut current = self.shared.write().unwrap_or_else(|e| e.into_inner());
        let epoch = current.source_epoch.wrapping_add(1);
        self.commands
            .send(Command::Source {
                endpoint,
                demo,
                epoch,
            })
            .map_err(|e| e.to_string())?;
        *current = Snapshot {
            source_epoch: epoch,
            ..Default::default()
        };
        Ok(())
    }
}

impl Drop for Receiver {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn publish(shared: &SharedSnapshot, snapshot: Snapshot, wake: &impl Fn()) {
    // The render thread uses try_read and never waits for this short copy.
    let mut current = shared.write().unwrap_or_else(|e| e.into_inner());
    if current.source_epoch != snapshot.source_epoch {
        return;
    }
    *current = snapshot;
    drop(current);
    wake();
}

fn receive_loop(
    mut endpoint: SocketAddr,
    mut is_demo: bool,
    shared: SharedSnapshot,
    commands: mpsc::Receiver<Command>,
    wake: impl Fn(),
) {
    let mut state = TelemetryState::default();
    let mut socket: Option<UdpSocket> = None;
    let mut buffer = [0u8; 65535];
    let mut next_bind = Instant::now();
    let mut demo_start = Instant::now();
    let mut demo_frame = 0u32;
    let pacer = Pacer::new();
    let mut next_demo = Instant::now();
    let mut catalog = None;
    let mut catalog_session = None;
    loop {
        let wait = if is_demo {
            Duration::ZERO
        } else if socket.is_none() {
            Duration::from_millis(100)
        } else {
            Duration::ZERO
        };
        let command = if wait.is_zero() {
            commands.try_recv().ok()
        } else {
            commands.recv_timeout(wait).ok()
        };
        match command {
            Some(Command::Stop) => break,
            Some(Command::Source {
                endpoint: address,
                demo,
                epoch,
            }) => {
                endpoint = address;
                is_demo = demo;
                socket = None;
                state = TelemetryState::default();
                state.snapshot.source_epoch = epoch;
                catalog = None;
                catalog_session = None;
                next_bind = Instant::now();
                demo_start = Instant::now();
                demo_frame = 0;
                next_demo = demo_start;
                publish(&shared, state.snapshot, &wake);
                continue;
            }
            None => (),
        }
        if is_demo {
            pacer.wait_until(next_demo);
            let now = Instant::now();
            next_demo += Duration::from_nanos(16_666_667);
            if now.saturating_duration_since(next_demo) > Duration::from_millis(100) {
                next_demo = now + Duration::from_nanos(16_666_667);
            }
            demo_frame = demo_frame.wrapping_add(1);
            for packet in demo::sample(demo_start.elapsed().as_secs_f32(), demo_frame) {
                if packet.header().packet_id != 10 || demo_frame % 6 == 1 {
                    state.apply(packet, now);
                }
            }
            state.snapshot.source = Connection::Demo;
            state.snapshot.ers_capacity = Some(ErsCapacity {
                joules: demo::DEMO_ERS_CAPACITY,
                year: 2026,
                source: CapacitySource::Demo,
            });
            publish(&shared, state.snapshot, &wake);
            continue;
        }
        let now = Instant::now();
        if socket.is_none() {
            if now < next_bind {
                continue;
            }
            match UdpSocket::bind(endpoint).and_then(|s| {
                s.set_read_timeout(Some(Duration::from_millis(50)))?;
                Ok(s)
            }) {
                Ok(s) => {
                    socket = Some(s);
                    state.snapshot.source = Connection::Listening;
                    state.snapshot.socket_error = 0;
                }
                Err(e) => {
                    state.snapshot.source = Connection::BindError;
                    state.snapshot.socket_error = e.raw_os_error().unwrap_or(-1);
                    next_bind = now + Duration::from_secs(1);
                }
            }
            publish(&shared, state.snapshot, &wake);
            continue;
        }
        match socket.as_ref().unwrap().recv_from(&mut buffer) {
            Ok((size, _sender)) => {
                let received = Instant::now();
                state.snapshot.counters.received += 1;
                let changed = match parse(&buffer[..size]) {
                    Ok(packet) => state.apply(packet, received),
                    Err(ParseError::Format(_) | ParseError::Version(_)) => {
                        state.snapshot.counters.unsupported += 1;
                        state.snapshot.source = Connection::Unsupported;
                        true
                    }
                    Err(_) => {
                        state.snapshot.counters.malformed += 1;
                        true
                    }
                };
                if changed {
                    let snapshot = &mut state.snapshot;
                    let class_known = snapshot.formula.is_some()
                        || snapshot.aero.is_some_and(|a| a.value.regulations_2026);
                    if class_known {
                        let session = (
                            snapshot.header.map(|h| h.session_uid),
                            snapshot.counters.restarts,
                        );
                        if catalog_session != Some(session) {
                            // Asset discovery/decompression runs once per session on this
                            // worker, never on the render thread or for each packet.
                            catalog = Some(BatteryCatalog::discover());
                            catalog_session = Some(session);
                        }
                    }
                    snapshot.ers_capacity = catalog.as_ref().and_then(|c| c.resolve(snapshot));
                    publish(&shared, state.snapshot, &wake);
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(e) => {
                state.snapshot.source = Connection::BindError;
                state.snapshot.socket_error = e.raw_os_error().unwrap_or(-1);
                socket = None;
                next_bind = Instant::now() + Duration::from_secs(1);
                publish(&shared, state.snapshot, &wake);
            }
        }
    }
}

/// Cached high-resolution waitable timer on Windows; std::thread::sleep elsewhere.
/// Absolute deadlines avoid accumulating wakeup jitter in demo and test traffic.
pub struct Pacer {
    #[cfg(windows)]
    timer: Option<windows::Win32::Foundation::HANDLE>,
}

impl Default for Pacer {
    fn default() -> Self {
        Self::new()
    }
}
impl Pacer {
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            timer: unsafe {
                use windows::Win32::System::Threading::*;
                CreateWaitableTimerExW(
                    None,
                    None,
                    CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                    TIMER_ALL_ACCESS.0,
                )
                .ok()
            },
        }
    }
    pub fn wait_until(&self, deadline: Instant) {
        let delay = deadline.saturating_duration_since(Instant::now());
        if delay.is_zero() {
            return;
        }
        #[cfg(windows)]
        if let Some(timer) = self.timer {
            use windows::Win32::System::Threading::*;
            let due = -((delay.as_nanos() / 100).clamp(1, i64::MAX as u128) as i64);
            unsafe {
                if SetWaitableTimer(timer, &due, 0, None, None, false).is_ok() {
                    WaitForSingleObject(timer, INFINITE);
                    return;
                }
            }
        }
        thread::sleep(delay);
    }
}

#[cfg(windows)]
impl Drop for Pacer {
    fn drop(&mut self) {
        if let Some(timer) = self.timer {
            unsafe {
                let _ = windows::Win32::Foundation::CloseHandle(timer);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_previous_source_cannot_publish_after_a_synchronous_reset() {
        let shared = Arc::new(RwLock::new(Snapshot {
            source_epoch: 2,
            ..Default::default()
        }));
        publish(
            &shared,
            Snapshot {
                source_epoch: 1,
                source: Connection::Demo,
                ..Default::default()
            },
            &|| panic!("old source woke renderer"),
        );
        assert_eq!(shared.read().unwrap().source, Connection::Waiting);
        publish(
            &shared,
            Snapshot {
                source_epoch: 2,
                source: Connection::Listening,
                ..Default::default()
            },
            &|| (),
        );
        assert_eq!(shared.read().unwrap().source, Connection::Listening);
    }
}
