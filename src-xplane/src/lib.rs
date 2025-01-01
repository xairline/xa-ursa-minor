// src/lib.rs
extern crate xplm;
mod logger;
mod misc;
use xplm::plugin::{Plugin, PluginInfo};
use xplm::{xplane_plugin};

struct MinimalPlugin;

impl Plugin for MinimalPlugin {
    type Error = std::convert::Infallible;

    fn start() -> Result<Self, Self::Error> {
        // The following message should be visible in the developer console and the Log.txt file
        plugin_debugln!("Hello, World! From the Minimal Rust Plugin");
        Ok(MinimalPlugin)
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("XA URSA Minor Driver"),
            signature: String::from("org.xairline.ursa-minor"),
            description: String::from("A plugin written in Rust"),
        }
    }
}

xplane_plugin!(MinimalPlugin);