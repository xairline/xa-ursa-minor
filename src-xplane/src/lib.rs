// src/lib.rs
extern crate xplm;

use xplm::xplane_plugin;

mod flight_loop;
mod logger;
mod misc;
mod plugin;
mod vibration;

xplane_plugin!(plugin::UrsaMinorPlugin);
