//! Tauri shell. The heavy lifting lives in the `doc-convert` sidecar (bundled
//! via `externalBin`); the frontend invokes it directly through the shell
//! plugin, so this stays minimal — just plugin registration.

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
