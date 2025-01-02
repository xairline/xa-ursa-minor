use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use crate::plugin_debugln;
// This is your HID wrapper crate
use xa_ursa_minor_hid::hid::HIDWrapper;

/// How often the worker processes new buffer items.
const PROCESS_INTERVAL: Duration = Duration::from_millis(20);

/// If you want to store 10 most recent magnitudes to smooth over.
const BUFFER_MAX_SIZE: usize = 10;

/// Exponential smoothing factor (like the `smoothing_alpha` in your Python).
const SMOOTHING_ALPHA: f32 = 0.3;

/// Duration (in seconds) to fade from old intensity to new intensity.
const FADE_DURATION: f32 = 0.2;

/// VibrationManager now stores float magnitudes (not just intensities).
pub struct VibrationManager {
    /// Buffer of recent magnitudes (acceleration changes).
    buffer: Vec<f32>,

    /// HID device wrapper.
    hid_wrapper: HIDWrapper,

    /// The last time we updated our wave/intensity (used for fading).
    last_update_instant: Instant,

    /// The old intensity from which we’re fading out.
    old_intensity: u8,

    /// The new (target) intensity we’re fading into.
    target_intensity: u8,
}

impl VibrationManager {
    /// Create a new manager with an empty buffer.
    pub fn new(hid_wrapper: HIDWrapper) -> Self {
        Self {
            buffer: Vec::new(),
            hid_wrapper,
            last_update_instant: Instant::now(),
            old_intensity: 0,
            target_intensity: 0,
        }
    }

    /// Add a new magnitude to the buffer, discarding the oldest if full.
    pub fn update_buffer(&mut self, new_magnitude: f32) {
        if self.buffer.len() >= BUFFER_MAX_SIZE {
            self.buffer.remove(0);
        }
        self.buffer.push(new_magnitude);
    }

    /// Compute exponential moving average over the buffer, like your Python logic:
    ///
    /// smoothed = sum( buffer[i] * alpha^i ) / sum( alpha^i )
    fn calculate_smoothed_magnitude(&self) -> f32 {
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        // Newest sample has index = 0 => multiply by alpha^0 = 1
        // Next has index = 1 => alpha^1, etc.
        //
        // Alternatively, you can reverse it so the newest is the end.
        // But this approach is fine as long as you’re consistent.
        for (i, &val) in self.buffer.iter().enumerate() {
            let weight = SMOOTHING_ALPHA.powi(i as i32);
            weighted_sum += val * weight;
            weight_total += weight;
        }

        if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            0.0
        }
    }

    /// Given a smoothed magnitude, convert to [0..255].
    /// Adjust `max_mag` for your typical range of deltas.
    fn map_magnitude_to_intensity(&self, mag: f32) -> u8 {
        let max_mag = 5.0; // or something that fits your data
        let scaled = (mag / max_mag) * 255.0;
        scaled.clamp(0.0, 255.0) as u8
    }

    /// Calculate a fade-blended intensity based on how long we’ve been fading.
    fn get_fade_blended_intensity(&self) -> u8 {
        let elapsed = self.last_update_instant.elapsed().as_secs_f32();
        if elapsed >= FADE_DURATION {
            // If we’ve already exceeded the fade duration, we’re at the new target fully.
            return self.target_intensity;
        }

        let progress = elapsed / FADE_DURATION; // 0..1
        let old_f = self.old_intensity as f32;
        let new_f = self.target_intensity as f32;
        let blended = old_f + (new_f - old_f) * progress;
        blended.round().clamp(0.0, 255.0) as u8
    }

    /// The main logic: smooth the buffer -> compute target intensity -> fade -> send to motor.
    pub fn process_buffer(&mut self) {
        // 1) Smooth the buffer
        let smoothed_mag = self.calculate_smoothed_magnitude();

        // 2) Map to [0..255] => new target intensity
        let new_intensity = self.map_magnitude_to_intensity(smoothed_mag);

        // 3) Update old & target intensities, and reset fade timer
        //    if the new intensity differs significantly
        if new_intensity != self.target_intensity {
            self.old_intensity = self.get_fade_blended_intensity();
            self.target_intensity = new_intensity;
            self.last_update_instant = Instant::now();
        }

        // 4) Compute fade-blended “current intensity”
        let current_intensity = self.get_fade_blended_intensity();

        // 5) Send to motor
        self.send_to_motor(current_intensity);
    }

    /// Actually send the intensity to the HID device.
    fn send_to_motor(&self, intensity: u8) {
        if intensity <= 15 {
            plugin_debugln!("Vibration Intensity -> {}", intensity);
        }
        if let Err(e) = self.hid_wrapper.write_vibration(intensity) {
            plugin_debugln!("Failed to write vibration to device: {}", e);
        }
    }
}

/// Worker thread:  
///  1. Receives (x, y, z) from the flight loop.  
///  2. Convert them to magnitudes (deltas or raw).  
///  3. Feed to VibrationManager, then process buffer every 50ms.
pub fn start_vibration_thread(rx: Receiver<(f32, f32, f32)>) {
    thread::spawn(move || {
        // Try to open the HID device once. If that fails, we panic or return.
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
                // Convert (ax, ay, az) to magnitude. For diffs, these might be small.
                let magnitude = (ax.powi(2) + ay.powi(2) + az.powi(2)).sqrt();
                vib_manager.update_buffer(magnitude);
            }

            // Now process & fade
            vib_manager.process_buffer();

            thread::sleep(PROCESS_INTERVAL);
        }
    });
}
