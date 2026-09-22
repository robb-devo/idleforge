use crate::config::{AppConfig, ProfileId, Wallet};
use crate::miners::MinerKind;
use crate::sensors::HardwareInfo;
use crate::state::{AppState, DashboardSnapshot};
use tauri::State;

#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>) -> DashboardSnapshot {
    state.0.lock().snapshot()
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> AppConfig {
    state.0.lock().config.clone()
}

#[tauri::command]
pub fn get_hardware(state: State<'_, AppState>) -> HardwareInfo {
    state.0.lock().hardware()
}

#[tauri::command]
pub fn start_miner(state: State<'_, AppState>, kind: MinerKind) -> Result<(), String> {
    state.0.lock().start(kind)
}

#[tauri::command]
pub fn stop_miner(state: State<'_, AppState>, kind: MinerKind) -> Result<(), String> {
    state.0.lock().stop(kind)
}

#[tauri::command]
pub fn set_profile(state: State<'_, AppState>, profile: ProfileId) -> Result<(), String> {
    state.0.lock().set_profile(profile)
}

#[tauri::command]
pub fn set_adaptive_enabled(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    state.0.lock().set_adaptive_enabled(enabled)
}

#[tauri::command]
pub fn add_wallet(state: State<'_, AppState>, wallet: Wallet) -> Result<(), String> {
    state.0.lock().add_wallet(wallet)
}

#[tauri::command]
pub fn switch_wallet(
    state: State<'_, AppState>,
    kind: MinerKind,
    wallet_id: String,
) -> Result<(), String> {
    state.0.lock().switch_wallet(kind, wallet_id)
}

#[tauri::command]
pub fn report_user_activity(state: State<'_, AppState>, active: bool) {
    state.0.lock().report_activity(active)
}

#[tauri::command]
pub fn save_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    state.0.lock().save(config)
}
