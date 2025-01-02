const BUFFER_MAX_SIZE: usize = 10;
const DECAY_STEP: u8 = 1;
use xa_ursa_minor_hid::hid::HIDWrapper;
/// Manages vibration intensity. Only updates to higher intensities
/// and decays when no new data arrives.
pub struct VibrationManager {
    /// Buffer of pending vibration intensities.
    buffer: Vec<u8>,
    /// Current active intensity sent to the motor.
    current_intensity: u8,
    hid_wrapper: HIDWrapper,
}

impl VibrationManager {
    pub fn new(hidwrapper: HIDWrapper) -> Self {
        Self {
            buffer: Vec::new(),
            current_intensity: 0,
            hid_wrapper: hidwrapper,
        }
    }

    /// Add a new intensity to our buffer, discarding the oldest if full.
    pub fn update_buffer(&mut self, new_intensity: u8) {
        if self.buffer.len() >= BUFFER_MAX_SIZE {
            self.buffer.remove(0);
        }
        self.buffer.push(new_intensity);
    }

    /// Process the next value in the buffer:
    ///  - If it’s greater than the current intensity, adopt it.
    ///  - If not, ignore it and keep current.
    ///  - If empty, decay the current intensity by DECAY_STEP.
    pub fn process_buffer(&mut self) {
        if let Some(&next_intensity) = self.buffer.first() {
            if next_intensity > self.current_intensity {
                self.current_intensity = next_intensity;
                self.send_to_motor();
            }
            // remove the consumed value
            self.buffer.remove(0);
        } else {
            // If buffer empty, decay
            if self.current_intensity > 0 {
                self.current_intensity = self.current_intensity.saturating_sub(DECAY_STEP);
                self.send_to_motor();
            }
        }
    }

    /// Here you'd talk to your actual vibration hardware. For demo, just print.
    fn send_to_motor(&self) {
        plugin_debugln!("Vibration Intensity -> {}", self.current_intensity);
        self.hid_wrapper
            .write_vibration(self.current_intensity)
            .unwrap()
    }
}

use crate::plugin_debugln;
use std::sync::mpsc::Receiver;
use std::thread;

/// How often the worker processes new buffer items.
const PROCESS_INTERVAL: std::time::Duration = std::time::Duration::from_millis(50);

/// Spawn a worker thread that:
/// 1. Receives g-force data from the flight loop.
/// 2. Converts g-forces to a vibration intensity.
/// 3. Feeds the intensity into the VibrationManager.
pub fn start_vibration_thread(rx: Receiver<(f32, f32, f32)>) {
    thread::spawn(move || {
        let mut vib_manager =
            VibrationManager::new(xa_ursa_minor_hid::hid::HIDWrapper::new().unwrap());

        loop {
            // Non-blocking receive: if there's data, process it; otherwise proceed.
            // If you prefer to block until new data, use 'rx.recv()' in a while loop.
            while let Ok((ax, ay, az)) = rx.try_recv() {
                let intensity = calculate_intensity(ax, ay, az);
                vib_manager.update_buffer(intensity);
            }

            // Process at least one item from the buffer or decay if empty
            vib_manager.process_buffer();

            // Sleep a bit before next cycle
            thread::sleep(PROCESS_INTERVAL);
        }
    });
}

/// Convert raw acceleration/g-forces into a 0-255 intensity.
fn calculate_intensity(gx: f32, gy: f32, gz: f32) -> u8 {
    // Example: magnitude-based approach
    let magnitude = (gx.powi(2) + gy.powi(2) + gz.powi(2)).sqrt();

    // Scale magnitude to 0-255. Adjust scaling as needed.
    let max_mag = 5.0; // Suppose 5 G is "max" for our scale
    let scaled = (magnitude / max_mag) * 255.0;
    scaled.clamp(0.0, 255.0) as u8
}
