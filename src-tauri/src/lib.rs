mod adaptive;
mod commands;
mod config;
mod miners;
mod sensors;
mod state;

use state::{AppState, AppStateInner};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(parking_lot::Mutex::new(AppStateInner::new())))
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::get_config,
            commands::get_hardware,
            commands::start_miner,
            commands::stop_miner,
            commands::set_profile,
            commands::set_adaptive_enabled,
            commands::add_wallet,
            commands::switch_wallet,
            commands::report_user_activity,
            commands::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running IdleForge");
}
