use crate::plugin_debugln;
use std::sync::mpsc::Sender;
use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, ReadOnly};
use xplm::flight_loop::FlightLoopCallback;
pub struct FlightLoopHandler {
    pub(crate) g_force_y: DataRef<f32, ReadOnly>,
    pub(crate) g_force_x: DataRef<f32, ReadOnly>,
    pub(crate) g_force_z: DataRef<f32, ReadOnly>,
    pub(crate) last_g_force_y: f32,
    pub(crate) last_g_force_x: f32,
    pub(crate) last_g_force_z: f32,
    pub(crate) tx: Sender<(f32, f32, f32)>,
}
impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, _state: &mut xplm::flight_loop::LoopState) {
        let cur_y = self.g_force_y.get();
        let cur_x = self.g_force_x.get();
        let cur_z = self.g_force_z.get();

        // Example of acceleration deltas:
        let diff_y = cur_y - self.last_g_force_y;
        let diff_x = cur_x - self.last_g_force_x;
        let diff_z = cur_z - self.last_g_force_z;

        // Send deltas (or raw forces) to worker thread
        // For raw, you'd just do (cur_x, cur_y, cur_z).
        if let Err(e) = self.tx.send((diff_x, diff_y, diff_z)) {
            plugin_debugln!("Failed to send g-force data: {}", e);
        }

        self.last_g_force_y = self.g_force_y.get();
        self.last_g_force_x = self.g_force_x.get();
        self.last_g_force_z = self.g_force_z.get();
    }
}
