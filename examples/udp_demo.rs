use f1_halo_telemetry::telemetry::demo;
use std::{
    net::UdpSocket,
    time::{Duration, Instant},
};

fn main() -> std::io::Result<()> {
    let destination = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:20777".into());
    let frequency = std::env::args()
        .nth(2)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(60)
        .clamp(1, 360);
    let seconds = std::env::args().nth(3).and_then(|v| v.parse::<f64>().ok());
    let socket = UdpSocket::bind("127.0.0.1:0")?;
    let start = Instant::now();
    let mut buffer = [0u8; 1448];
    let pacer = f1_halo_telemetry::udp::Pacer::new();
    println!("SIMULATORE UDP ESPLICITO: {destination}, {frequency} Hz; Ctrl+C per terminare.");
    for frame in 1u32.. {
        if seconds.is_some_and(|s| start.elapsed().as_secs_f64() >= s) {
            break;
        }
        let due = start + Duration::from_secs_f64(frame as f64 / frequency as f64);
        for packet in demo::sample(start.elapsed().as_secs_f32(), frame) {
            if packet.header().packet_id == 10 && frame % (frequency / 10).max(1) != 0 {
                continue;
            }
            let length = demo::encode(packet, &mut buffer);
            socket.send_to(&buffer[..length], &destination)?;
        }
        pacer.wait_until(due);
    }
    Ok(())
}
