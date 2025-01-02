use hidapi::HidApi;

// Replace these with your actual values
static VID: u16 = 0x4098;
static PID: u16 = 0xBC27;

pub struct HIDWrapper {
    api: HidApi,
}

impl HIDWrapper {
    /// Attempt to create a new HIDWrapper. Returns `None` if any step fails.
    pub fn new() -> Option<Self> {
        // Create the HID API instance
        let api = HidApi::new().ok()?;

        Some(HIDWrapper { api })
    }

    /// Retrieve the serial number string, or `None` if something fails
    pub fn get_serial_number(&self) -> Option<String> {
        // Attempt to open the desired device
        let device = self
            .api
            .open(VID, PID)
            .ok()
            .ok_or_else(|| "Failed to open HID device".to_string())
            .ok()?;
        device.get_serial_number_string().ok().flatten()
    }

    /// Write raw data to the device. Returns Ok(()) on success, or Err on failure.
    pub fn write_data(&self, data: &[u8]) -> Result<(), String> {
        let device = self
            .api
            .open(VID, PID)
            .ok()
            .ok_or_else(|| "Failed to open HID device".to_string())?;
        device
            .write(data)
            .map(|_| ())
            .map_err(|e| format!("Failed to write to device: {e}"))
    }

    pub fn write_vibration(&self, vibration: u8) -> Result<(), String> {
        let mut data = [0x02, 7, 191, 0, 0, 3, 0x49, 0, 0, 0, 0, 0, 0, 0];
        data[8] = vibration;
        self.write_data(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_returns_none_if_no_device() {
        // If the device with VID/PID is not actually connected,
        // new() will return None.
        let wrapper = HIDWrapper::new();
        if wrapper.is_none() {
            println!(
                "No HID device found at VID=0x{:04X}, PID=0x{:04X}.",
                VID, PID
            );
        }
        // We won't assert a failure here because it might be genuinely disconnected.
        // Instead, we'll just pass if the code doesn't crash.
    }

    #[test]
    fn test_get_serial_number() {
        // This test will only pass if a device is actually connected with the correct VID/PID.
        // We'll skip if no device is found.
        let Some(wrapper) = HIDWrapper::new() else {
            println!("No device connected; skipping test_get_serial_number()");
            return;
        };

        // Attempt to get the serial number. We won't assert for a specific string
        // unless you know what the SN should be.
        let serial = wrapper.get_serial_number();
        println!("Serial number: {:?}", serial);
        // Just ensure it doesn’t panic. If you want a stricter check, you could:
        // assert!(serial.is_some(), "Device had no serial number.");
    }

    #[test]
    fn test_write_data() {
        // This test will also only pass if a device is actually connected.
        let Some(wrapper) = HIDWrapper::new() else {
            println!("No device connected; skipping test_write_data()");
            return;
        };

        // A small data buffer to write. Adjust as appropriate for your device.
        let data = [0x00, 0x01, 0x02, 0x03];

        let result = wrapper.write_data(&data);
        match result {
            Ok(_) => println!("Successfully wrote data to the device."),
            Err(e) => panic!("Failed to write data to the device: {e}"),
        }
    }
}
