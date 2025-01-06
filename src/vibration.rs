use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use crate::plugin_debugln;
use crate::hid::HIDWrapper;
/// How often the worker processes new buffer items (e.g. ~50 Hz).
pub static mut PROCESS_INTERVAL: Duration = Duration::from_millis(20);
/// Duration of each new wave in seconds.
pub static mut WAVE_DURATION: f32 = 0.2;
/// Map magnitude to [0..255]. Adjust `MAX_MAG` to fit your typical input range.
pub static mut MAX_MAG: f32 = 1.5;
/// Only bother writing intensities above this threshold (like your existing 5).
pub static mut MIN_MOTOR_INTENSITY: u8 = 3;
pub static mut HIGH_PASS_ALPHA: f32 = 0.9;
pub static mut BASE_FREQ: f32 = 1.0; // Starting frequency
pub static mut FREQ_SENSITIVITY: f32 = 2.0; // Scale factor for delta -> frequency
pub static mut BASE_SHARPNESS: f32 = 1.0; // Starting sharpness
pub static mut SHARPNESS_SENSITIVITY: f32 = 2.0; // Scale factor for delta -> sharpness (raising sine wave)
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
    wave_frequency: f32, // per-wave frequency
    wave_sharpness: f32, // per-wave shaping exponent
}

impl WaveEvent {
    /// Return the intensity of this wave at the given `now` instant.
    /// If the wave has expired, return `None`.
    unsafe fn current_intensity(&self, now: Instant) -> Option<u8> {
        let elapsed = now.duration_since(self.start_time).as_secs_f32();
        if elapsed > WAVE_DURATION {
            // Wave is fully expired
            return None;
        }

        // 0..1 progress through the wave
        let progress = elapsed / WAVE_DURATION;

        // Example: full sine wave from 0..(2π * wave_frequency)
        let raw_sine = (progress * std::f32::consts::TAU * self.wave_frequency).sin();

        // We only want positive arcs. If you prefer a half-sine from 0..π, do:
        //   let raw_sine = (progress * std::f32::consts::PI).sin();
        // (It starts at 0, up to 1, back to 0, without going negative.)

        let shaped = if raw_sine < 0.0 {
            0.0
        } else {
            // Raise sine to wave_sharpness for steeper rise/fall
            raw_sine.powf(self.wave_sharpness)
        };

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

    previous_mag: f32,
}

impl VibrationManager {
    /// Create a new manager with no active waves.
    pub unsafe fn new(hid_wrapper: HIDWrapper) -> Self {
        Self {
            waves: Vec::new(),
            hid_wrapper,
            last_intensity: 0,
            hp_filter: HighPassFilter3D::new(HIGH_PASS_ALPHA),
            previous_mag: 0.0,
        }
    }

    /// Convert (ax, ay, az) -> magnitude -> wave with a certain peak intensity, then store it.
    pub unsafe fn spawn_wave_for_input(&mut self, ax: f32, ay: f32, az: f32) {
        let (fx, fy, fz) = self.hp_filter.filter((ax, ay, az));

        // Example magnitude is sqrt(fx^2 + fy^2 + fz^2).
        let mag = (fx.powi(2) + fy.powi(2) + fz.powi(2)).sqrt();

        // Convert magnitude to [0..255]
        let scaled = (mag / MAX_MAG) * 255.0;
        let target_intensity = scaled.clamp(0.0, 255.0) as u8;

        // If the scaled intensity is trivial (like 0), skip spawning wave
        if target_intensity == 0 {
            return;
        }

        // ------------------- NEW: compute delta and map it to frequency/sharpness -------------------
        let delta_mag = (mag - self.previous_mag).abs();
        self.previous_mag = mag; // update for next call

        // Dynamic frequency: e.g., base + some sensitivity * delta
        let wave_frequency = BASE_FREQ + FREQ_SENSITIVITY * delta_mag;

        // Dynamic sharpness: e.g., base + some sensitivity * delta
        // clamp or limit if you like (to avoid going too high)
        let wave_sharpness = (BASE_SHARPNESS + SHARPNESS_SENSITIVITY * delta_mag).clamp(1.0, 5.0);

        // Create a new wave event
        let wave = WaveEvent {
            start_time: Instant::now(),
            target_intensity,
            wave_frequency,
            wave_sharpness,
        };

        // Insert wave into the list
        self.waves.push(wave);
    }

    /// Called regularly (e.g. every 20ms) to update waves and send motor commands.
    pub unsafe fn update(&mut self) {
        let now = Instant::now();

        // Compute the maximum intensity across all active waves
        let mut max_intensity = 0u8;
        self.waves.retain(|wave| unsafe {
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
    thread::spawn(move || unsafe {
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
