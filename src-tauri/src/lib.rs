pub mod commands;
pub mod models;
pub mod services;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::system_info::exit_app,
            commands::system_info::get_app_version,
            commands::system_info::get_system_info,
            commands::virtualization::get_virtualization_status,
            commands::disable::execute_disable,
            commands::disable::request_reboot,
            commands::export::export_csv,
            commands::export::export_csv_auto,
            commands::export::export_inspection_csvs_auto,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
