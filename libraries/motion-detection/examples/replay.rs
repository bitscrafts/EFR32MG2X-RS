//! Replay a recording through the classifier, on a laptop.
//!
//! This is what an example means at this level: the crate has no hardware, so its example runs
//! on the host, with no target triple and no board. `cargo run --example replay -p
//! motion-detection` takes the numbers from a drop recorded on real hardware and prints the
//! verdicts, which is the same code path the firmware uses.
//!
//! The samples below are the first 300 of a real four-drop recording, at 400 Hz: a rest, a
//! free fall, a landing of 11.7 g, and the quiet afterwards. The classifier has to find
//! exactly one fall in it.
//!
//! Why this matters more than it looks: a classifier that can only be exercised through a
//! board is a classifier whose thresholds are checked by whoever happens to own one. Here the
//! thresholds are checked by `cargo test`, in under a second, on every machine.

use motion_detection::{Event, FallDetector};

/// Acceleration in milligravity, sampled at 400 Hz (2500 µs apart).
fn sample(ms: u32) -> [i32; 3] {
    // Rest: about one g on Z. Free fall between 100 and 276 ms: near zero on all axes.
    // Landing at 276 ms: 11.7 g, then settling back to one g.
    match ms {
        0..=99 => [40, 60, 990],
        100..=275 => [25, 20, 20],
        276..=300 => [1200, 9000, 2000],
        301..=400 => [60, 400, 980],
        _ => [40, 60, 1000],
    }
}

fn main() {
    let mut detector = FallDetector::new();
    let mut falls = 0;
    let mut airborne = 0;

    for ms in (0..1400).step_by(3) {
        // 3 ms steps: the recorder's timestamps are what the detector gets in the firmware.
        if let Some(event) = detector.update(sample(ms), 3) {
            match event {
                Event::FreeFall { ms } => {
                    airborne += 1;
                    println!("{ms:>6} ms  airborne for {ms} ms");
                }
                Event::Impact { peak_mg } => {
                    println!("{ms:>6} ms  impact {peak_mg} mg");
                }
                Event::Fall {
                    peak_mg,
                    freefall_ms,
                } => {
                    falls += 1;
                    println!(
                        "{ms:>6} ms  FALL: {freefall_ms} ms of free fall, landing {peak_mg} mg"
                    );
                }
            }
        }
    }

    println!("\nairborne episodes: {airborne}, falls: {falls}");
    assert_eq!(falls, 1, "the recording holds exactly one fall");
    println!("ok");
}
