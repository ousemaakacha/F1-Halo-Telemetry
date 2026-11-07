use f1_halo_telemetry::telemetry::{demo, parse};
use std::{hint::black_box, time::Instant};

fn main() {
    let mut buffer = [0u8; 1448];
    let length = demo::encode(demo::sample(7.0, 420)[0], &mut buffer);
    let iterations = 2_000_000u32;
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(parse(black_box(&buffer[..length])).unwrap());
    }
    let elapsed = start.elapsed();
    println!(
        "Car Telemetry, {iterations} parses, {:.2} ns/packet, {:.0} packets/s",
        elapsed.as_nanos() as f64 / iterations as f64,
        iterations as f64 / elapsed.as_secs_f64()
    );
}
