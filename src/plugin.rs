use crate::flight_loop::FlightLoopHandler;
use crate::hid::HIDWrapper;
use crate::plugin_debugln;
use crate::vibration::start_vibration_thread;
use xplm::data::borrowed::DataRef;
use xplm::flight_loop::FlightLoop;
use xplm::plugin::{Plugin, PluginInfo};

pub struct UrsaMinorPlugin {
    flight_loop: FlightLoop,
}

impl Plugin for UrsaMinorPlugin {
    type Error = std::convert::Infallible;

    fn start() -> Result<Self, Self::Error> {
        // The following message should be visible in the developer console and the Log.txt file
        plugin_debugln!("Hello, World! From the Minimal Rust Plugin");

        let (tx, r_) = std::sync::mpsc::channel();
        let plugin = Self {
            flight_loop: FlightLoop::new(FlightLoopHandler {
                g_force_y: DataRef::find("sim/flightmodel2/misc/gforce_axil").unwrap(),
                g_force_x: DataRef::find("sim/flightmodel2/misc/gforce_side").unwrap(),
                g_force_z: DataRef::find("sim/flightmodel2/misc/gforce_normal").unwrap(),
                last_g_force_x: 0.0,
                last_g_force_y: 0.0,
                last_g_force_z: 0.0,
                tx: tx,
            }),
        };

        Ok(plugin)
    }

    fn enable(&mut self) -> Result<(), Self::Error> {
        // The following message should be visible in the developer console and the Log.txt file
        plugin_debugln!("Plugin enabled");
        let (tx, rx) = std::sync::mpsc::channel();
        start_vibration_thread(rx);
        self.flight_loop = FlightLoop::new(FlightLoopHandler {
            g_force_y: DataRef::find("sim/flightmodel2/misc/gforce_axil").unwrap(),
            g_force_x: DataRef::find("sim/flightmodel2/misc/gforce_side").unwrap(),
            g_force_z: DataRef::find("sim/flightmodel2/misc/gforce_normal").unwrap(),
            last_g_force_x: 0.0,
            last_g_force_y: 0.0,
            last_g_force_z: 0.0,
            tx: tx,
        });
        self.flight_loop.schedule_immediate();
        Ok(())
    }

    fn disable(&mut self) {
        self.flight_loop.deactivate();
        HIDWrapper::new().unwrap().write_vibration(0).unwrap();
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("XA URSA Minor Driver"),
            signature: String::from("org.xairline.ursa-minor"),
            description: String::from("A plugin written in Rust"),
        }
    }
}
