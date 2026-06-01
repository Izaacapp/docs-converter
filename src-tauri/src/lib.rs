//! Tauri shell. Conversion happens on the converter server; the frontend POSTs
//! PDFs to it over HTTP and saves the result — so this is just plugin
//! registration (http for the API, fs to save the file, dialog to pick a path).

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
