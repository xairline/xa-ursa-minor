use hidapi::HidApi;
use std::{thread, time};
static VID: u16 = 0x4098;
static PID: u16 = 0xBC27;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_sn() -> String {
    // Attempt to create the HID API; if it fails, return ""
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(_) => return "".to_string(),
    };

    // Attempt to open the device; if it fails, return ""
    let device = match api.open(VID, PID) {
        Ok(device) => device,
        Err(_) => return "".to_string(),
    };

    // Attempt to get the serial number; if anything goes wrong, return ""
    match device.get_serial_number_string() {
        Ok(Some(sn)) => sn,
        _ => "".to_string(),
    }
}

#[tauri::command]
fn restart_ursa_minor() -> String {
    // 02 01 00 00 00 01 04 00 00 00 00 00 00 00

    // Attempt to create the HID API; if it fails, return ""
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(_) => return "".to_string(),
    };

    // Attempt to open the device; if it fails, return ""
    let device = match api.open(VID, PID) {
        Ok(device) => device,
        Err(_) => return "".to_string(),
    };

    // send the restart command
    let data = [
        0x02, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    match device.write(&data) {
        Ok(_) => "Success".to_string(),
        Err(_) => "Failed".to_string(),
    }
}

#[tauri::command]
fn test_ursa_minor() -> String {
    // 02 01 00 00 00 01 04 00 00 00 00 00 00 00

    // Attempt to create the HID API; if it fails, return ""
    let api = match HidApi::new() {
        Ok(api) => api,
        Err(_) => return "".to_string(),
    };

    // Attempt to open the device; if it fails, return ""
    let device = match api.open(VID, PID) {
        Ok(device) => device,
        Err(_) => return "".to_string(),
    };

    // Data to be sent
    let mut data = [0x02, 7, 191, 0, 0, 3, 0x49, 0, 0, 0, 0, 0, 0, 0];

    // Start the timer
    let start = time::Instant::now();
    let mut counter = 0;

    while start.elapsed() < time::Duration::from_secs(2) {
        data[8] = counter;
        match device.write(&data) {
            Ok(_) => {
                counter += 1; // Increment the counter
                println!("Command sent successfully. Count: {}", counter);
            }
            Err(e) => {
                println!("Failed to send command: {}", e);
                return "Failed".to_string();
            }
        }
        // Small delay to avoid spamming (optional, adjust as needed)
        thread::sleep(time::Duration::from_millis(100));
    }
    data[8] = 0;
    match device.write(&data) {
        Ok(_) => {
            counter += 1; // Increment the counter
            println!("Command sent successfully. Count: {}", counter);
        }
        Err(e) => {
            println!("Failed to send command: {}", e);
            return "Failed".to_string();
        }
    }
    // Small delay to avoid spamming (optional, adjust as needed)
    thread::sleep(time::Duration::from_millis(100));
    println!("Total commands sent: {}", counter);
    "Success".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_sn,
            restart_ursa_minor,
            test_ursa_minor
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
