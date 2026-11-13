use f1_halo_telemetry::{
    settings::Settings,
    telemetry::{CapacitySource, Connection, Packet, demo},
    udp::Receiver,
};
use std::{
    net::UdpSocket,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

#[test]
fn actual_udp_receives_latest_snapshot_and_switches_sources() {
    let probe = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    let settings = Settings {
        port,
        ..Settings::default()
    };
    let wakes = Arc::new(AtomicU64::new(0));
    let worker_wakes = wakes.clone();
    let receiver = Receiver::start(&settings, false, move || {
        worker_wakes.fetch_add(1, Ordering::Relaxed);
    })
    .unwrap();
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    let mut buffer = [0u8; f1_halo_telemetry::telemetry::protocol::HISTORY_SIZE];
    let start = Instant::now();
    let mut received = false;
    let mut frame = 120;
    while start.elapsed() < Duration::from_secs(3) {
        // Exercise UDP lap timing without triggering unrelated game-asset discovery.
        for packet in demo::sample(2.0, frame).into_iter().filter(|packet| {
            matches!(
                packet,
                Packet::Telemetry(..) | Packet::History(..) | Packet::Lap(..)
            )
        }) {
            let length = demo::encode(packet, &mut buffer);
            sender
                .send_to(&buffer[..length], ("127.0.0.1", port))
                .unwrap();
        }
        frame += 1;
        std::thread::sleep(Duration::from_millis(20));
        let snapshot = *receiver.shared.read().unwrap();
        if snapshot.lap_delta_ms(Instant::now()) == Some(230)
            && snapshot.gap_behind_ms(Instant::now()) == Some(2_536)
        {
            received = true;
            break;
        }
    }
    assert!(received);
    assert!(wakes.load(Ordering::Relaxed) > 0);
    receiver.source(&settings, true).unwrap();
    let start = Instant::now();
    while receiver.shared.read().unwrap().source != Connection::Demo {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(receiver.shared.read().unwrap().source, Connection::Demo);
    assert_eq!(
        receiver.shared.read().unwrap().ers_capacity.unwrap().source,
        CapacitySource::Demo
    );
    receiver.source(&settings, false).unwrap();
    {
        let cleared = *receiver.shared.read().unwrap();
        assert!(cleared.ers_capacity.is_none());
        assert!(cleared.aero.is_none());
        assert!(cleared.lap_history.is_none());
        assert!(cleared.gap_behind.is_none());
        assert!(cleared.telemetry.is_none());
        assert_eq!(cleared.source_epoch, 2);
    }
    let start = Instant::now();
    while receiver.shared.read().unwrap().source != Connection::Listening {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(receiver.shared.read().unwrap().telemetry.is_none());
}

#[test]
fn busy_port_is_reported_and_retried_automatically() {
    let probe = UdpSocket::bind("127.0.0.1:0").unwrap();
    let settings = Settings {
        port: probe.local_addr().unwrap().port(),
        ..Settings::default()
    };
    let receiver = Receiver::start(&settings, false, || ()).unwrap();
    let start = Instant::now();
    while receiver.shared.read().unwrap().source != Connection::BindError {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_ne!(receiver.shared.read().unwrap().socket_error, 0);
    drop(probe);
    let start = Instant::now();
    while receiver.shared.read().unwrap().source != Connection::Listening {
        assert!(start.elapsed() < Duration::from_secs(3));
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn malformed_and_unsupported_datagrams_do_not_prevent_recovery() {
    let probe = UdpSocket::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    let settings = Settings {
        port,
        ..Settings::default()
    };
    let receiver = Receiver::start(&settings, false, || ()).unwrap();
    let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
    let start = Instant::now();
    while receiver.shared.read().unwrap().source != Connection::Listening {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut buffer = [0u8; 1448];
    let size = demo::encode(demo::sample(1.0, 60)[0], &mut buffer);
    buffer[0..2].copy_from_slice(&2025u16.to_le_bytes());
    sender
        .send_to(&buffer[..size], ("127.0.0.1", port))
        .unwrap();
    let start = Instant::now();
    while receiver.shared.read().unwrap().source != Connection::Unsupported {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(10));
    }
    sender.send_to(&[0; 4], ("127.0.0.1", port)).unwrap();
    let start = Instant::now();
    while receiver.shared.read().unwrap().counters.malformed == 0 {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(10));
    }
    buffer[0..2].copy_from_slice(&2026u16.to_le_bytes());
    sender
        .send_to(&buffer[..size], ("127.0.0.1", port))
        .unwrap();
    let start = Instant::now();
    while receiver.shared.read().unwrap().telemetry.is_none() {
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(10));
    }
    let snapshot = *receiver.shared.read().unwrap();
    assert_eq!(snapshot.connection(Instant::now()), Connection::Live);
    assert_eq!(snapshot.counters.unsupported, 1);
    assert_eq!(snapshot.counters.malformed, 1);
}
