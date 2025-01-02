use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use crate::plugin_debugln;
use xa_ursa_minor_hid::hid::HIDWrapper;

/// How often the worker processes new buffer items (e.g. ~50 Hz).
const PROCESS_INTERVAL: Duration = Duration::from_millis(20);

/// Duration of each new wave in seconds.
const WAVE_DURATION: f32 = 0.2;

/// If you want to control how many sine cycles occur within `WAVE_DURATION`.
const WAVE_FREQUENCY: f32 = 1.0; // 1 cycle from 0..WAVE_DURATION

/// “Sharpness” or shaping exponent. Adjust to taste; >1 means “punchier” wave.
const SHARPNESS: f32 = 1.0;

/// Map magnitude to [0..255]. Adjust `MAX_MAG` to fit your typical input range.
const MAX_MAG: f32 = 5.0;

/// Only bother writing intensities above this threshold (like your existing 5).
const MIN_MOTOR_INTENSITY: u8 = 5;

const HIGH_PASS_ALPHA: f32 = 0.9;

/// Simple 3D high-pass filter using the one-pole method.
struct HighPassFilter3D {
    alpha: f32,
    prev_input: (f32, f32, f32),
    prev_output: (f32, f32, f32),
}

impl HighPassFilter3D {
    /// Create a new high-pass filter with a given alpha (0..1).
    /// Larger alpha => stronger high-pass (keeps more high-frequency).
    /// Smaller alpha => more smoothing, less “buzz.”
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha,
            // Start with zeros; or you could initialize differently if desired
            prev_input: (0.0, 0.0, 0.0),
            prev_output: (0.0, 0.0, 0.0),
        }
    }

    /// Filter the given (x, y, z) and return the high-passed (x, y, z).
    pub fn filter(&mut self, current_input: (f32, f32, f32)) -> (f32, f32, f32) {
        let (cx, cy, cz) = current_input;
        let (px, py, pz) = self.prev_input;
        let (ox, oy, oz) = self.prev_output;

        // Apply high-pass for each axis
        let out_x = self.alpha * (ox + cx - px);
        let out_y = self.alpha * (oy + cy - py);
        let out_z = self.alpha * (oz + cz - pz);

        // Update state
        self.prev_input = current_input;
        self.prev_output = (out_x, out_y, out_z);

        (out_x, out_y, out_z)
    }
}

/// A single “wave event” that starts at `start_time`, has a peak amplitude
/// (`target_intensity`), and lasts for `WAVE_DURATION`.
struct WaveEvent {
    start_time: Instant,
    target_intensity: u8,
}

impl WaveEvent {
    /// Return the intensity of this wave at the given `now` instant.
    /// If the wave has expired, return `None`.
    fn current_intensity(&self, now: Instant) -> Option<u8> {
        let elapsed = now.duration_since(self.start_time).as_secs_f32();

        if elapsed > WAVE_DURATION {
            // Wave is fully expired
            return None;
        }

        // 0..1 progress through the wave
        let progress = elapsed / WAVE_DURATION;

        // Example sine wave approach:
        //   raw_sine goes from 0..(2π * WAVE_FREQUENCY)
        //   We clamp it to [0..1] after the shaping
        let raw_sine = (progress * std::f32::consts::TAU * WAVE_FREQUENCY).sin();

        // You might prefer a half-sine from 0..π so it starts at 0, up to 1, back to 0.
        // For that, do something like:
        //    let raw_sine = (progress * std::f32::consts::PI).sin();
        // Then it goes from 0..1..0. Use whichever shape you like.

        // “Raise to SHARPNESS” for more abrupt ramp up/down:
        let shaped = if raw_sine < 0.0 { 0.0 } else { raw_sine }.powf(SHARPNESS);

        let intensity_f = shaped * (self.target_intensity as f32);
        let intensity = intensity_f.round().clamp(0.0, 255.0) as u8;

        Some(intensity)
    }
}

/// VibrationManager now spawns wave events and merges them by taking a pointwise max.
pub struct VibrationManager {
    /// The list of active waves. We add a new wave whenever we get new input.
    /// We remove waves once they’re expired.
    waves: Vec<WaveEvent>,

    /// HID device wrapper.
    hid_wrapper: HIDWrapper,

    /// Track the last written intensity so we can avoid spamming the same value.
    last_intensity: u8,

    hp_filter: HighPassFilter3D,
}

impl VibrationManager {
    /// Create a new manager with no active waves.
    pub fn new(hid_wrapper: HIDWrapper) -> Self {
        Self {
            waves: Vec::new(),
            hid_wrapper,
            last_intensity: 0,
            hp_filter: HighPassFilter3D::new(HIGH_PASS_ALPHA),
        }
    }

    /// Convert (ax, ay, az) -> magnitude -> wave with a certain peak intensity, then store it.
    pub fn spawn_wave_for_input(&mut self, ax: f32, ay: f32, az: f32) {
        let (fx, fy, fz) = self.hp_filter.filter((ax, ay, az));

        // For example, magnitude is sqrt(ax^2 + ay^2 + az^2).
        let mag = (fx.powi(2) + fy.powi(2) + fz.powi(2)).sqrt();

        // Convert magnitude to [0..255]
        let scaled = (mag / MAX_MAG) * 255.0;
        let target_intensity = scaled.clamp(0.0, 255.0) as u8;

        // If the scaled intensity is trivial (like 0), skip spawning wave
        if target_intensity == 0 {
            return;
        }

        // Create a new wave event
        let wave = WaveEvent {
            start_time: Instant::now(),
            target_intensity,
        };

        // Insert wave into the list
        self.waves.push(wave);
    }

    /// Called regularly (e.g. every 20ms) to update waves and send motor commands.
    pub fn update(&mut self) {
        let now = Instant::now();

        // Compute the maximum intensity across all active waves
        let mut max_intensity = 0u8;
        self.waves.retain(|wave| {
            if let Some(current) = wave.current_intensity(now) {
                if current > max_intensity {
                    max_intensity = current;
                }
                true // wave is still active
            } else {
                false // wave has expired
            }
        });

        // Write to motor only if:
        //  - the max intensity is above threshold, or
        //  - it’s zero but the last intensity was non-zero
        if max_intensity >= MIN_MOTOR_INTENSITY {
            if max_intensity != self.last_intensity {
                plugin_debugln!("Vibration Intensity -> {}", max_intensity);
                if let Err(e) = self.hid_wrapper.write_vibration(max_intensity) {
                    plugin_debugln!("Failed to write vibration to device: {}", e);
                }
                self.last_intensity = max_intensity;
            }
        } else {
            // If the new max is 0 but we previously had something non-zero, then set to 0
            if max_intensity == 0 && self.last_intensity != 0 {
                plugin_debugln!("Vibration Intensity -> 0");
                if let Err(e) = self.hid_wrapper.write_vibration(0) {
                    plugin_debugln!("Failed to write vibration to device: {}", e);
                }
                self.last_intensity = 0;
            }
        }
    }
}

/// Worker thread:
///   1. Receives (x, y, z) from flight loop or other source.
///   2. Spawns a wave on each new input.
///   3. Updates/merges waves every `PROCESS_INTERVAL`.
pub fn start_vibration_thread(rx: Receiver<(f32, f32, f32)>) {
    thread::spawn(move || {
        // Try to open the HID device once. If that fails, bail out.
        let hid_wrapper = match HIDWrapper::new() {
            Some(h) => h,
            None => {
                plugin_debugln!("Could not open HID device! Vibration will be disabled.");
                return;
            }
        };

        let mut vib_manager = VibrationManager::new(hid_wrapper);

        loop {
            // Pull in all available data from the channel (non-blocking).
            while let Ok((ax, ay, az)) = rx.try_recv() {
                // For each new triple, spawn a wave.
                vib_manager.spawn_wave_for_input(ax, ay, az);
            }

            // Update waves & write to motor
            vib_manager.update();

            thread::sleep(PROCESS_INTERVAL);
        }
    });
}
