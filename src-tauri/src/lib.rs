use std::{thread, time};
use xa_ursa_minor_hid::hid::HIDWrapper;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_sn() -> String {
    // Attempt to create our HID wrapper
    let Some(hid_wrapper) = HIDWrapper::new() else {
        return "".to_string();
    };

    // Return the serial number if it exists, else an empty string
    hid_wrapper
        .get_serial_number()
        .unwrap_or_else(|| "".to_string())
}

#[tauri::command]
fn restart_ursa_minor() -> String {
    // The restart command data
    let data = [
        0x02, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // Attempt to create our HID wrapper
    let Some(hid_wrapper) = HIDWrapper::new() else {
        return "".to_string();
    };

    // Write the data
    match hid_wrapper.write_data(&data) {
        Ok(_) => "Success".to_string(),
        Err(_) => "Failed".to_string(),
    }
}

#[tauri::command]
fn test_ursa_minor() -> String {
    // Attempt to create our HID wrapper
    let Some(hid_wrapper) = HIDWrapper::new() else {
        return "".to_string();
    };

    let start = time::Instant::now();
    let mut counter = 0;

    // Loop for 2 seconds
    while start.elapsed() < time::Duration::from_secs(2) {
        // Write the data to the HID device
        if let Err(e) = hid_wrapper.write_vibration(counter % 255) {
            eprintln!("Failed to send command: {}", e);
            return "Failed".to_string();
        }
        counter += 1;
        println!("Command sent successfully. Count: {}", counter);

        // Sleep a bit to avoid spamming
        thread::sleep(time::Duration::from_millis(10));
    }

    // Final write
    if let Err(e) = hid_wrapper.write_vibration(0) {
        eprintln!("Failed to send final command: {}", e);
        return "Failed".to_string();
    }
    counter += 1;
    println!("Command sent successfully. Count: {}", counter);

    // Final short delay (optional)
    thread::sleep(time::Duration::from_millis(100));
    println!("Total commands sent: {}", counter);

    "Success".to_string()
}

#[tauri::command]
fn lights_off() -> String {
    // Attempt to create our HID wrapper
    let Some(hid_wrapper) = HIDWrapper::new() else {
        return "".to_string();
    };

    // Write the data
    match hid_wrapper.write_backlight(0) {
        Ok(_) => "Success".to_string(),
        Err(_) => "Failed".to_string(),
    }
}

#[tauri::command]
fn lights_on() -> String {
    // Attempt to create our HID wrapper
    let Some(hid_wrapper) = HIDWrapper::new() else {
        return "".to_string();
    };

    // Write the data
    match hid_wrapper.write_backlight(255) {
        Ok(_) => "Success".to_string(),
        Err(_) => "Failed".to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_sn,
            restart_ursa_minor,
            test_ursa_minor,
            lights_off,
            lights_on,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
