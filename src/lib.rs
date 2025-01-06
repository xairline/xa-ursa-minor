// src/lib.rs
extern crate xplm;

use xplm::xplane_plugin;

mod flight_loop;
mod logger;
mod misc;
mod plugin;
mod vibration;
mod hid;

xplane_plugin!(plugin::UrsaMinorPlugin);
