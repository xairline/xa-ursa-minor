use hidapi::HidApi;

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
        Ok(Some(sn)) => {
            println!("Serial number: {}", sn);
            sn
        }
        _ => "".to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_sn])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
