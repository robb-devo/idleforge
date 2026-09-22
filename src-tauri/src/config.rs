use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProfileId {
    Idle,
    Low,
    Medium,
    High,
    Extreme,
}

impl ProfileId {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProfileId::Idle => "idle",
            ProfileId::Low => "low",
            ProfileId::Medium => "medium",
            ProfileId::High => "high",
            ProfileId::Extreme => "extreme",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: String,
    pub label: String,
    pub coin: String,
    pub receive_address: String,
    pub pool_worker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDef {
    pub label: String,
    pub cpu_intensity: f64,
    pub gpu_intensity: f64,
    pub cpu_threads_ratio: f64,
    pub gpu_power_limit_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub url: String,
    pub tls: bool,
    pub user_template: String,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerChannelConfig {
    pub enabled: bool,
    pub adapter: String,
    pub coin: String,
    pub algorithm: String,
    pub binary_path: String,
    pub pool: PoolConfig,
    #[serde(default)]
    pub extra_args: Vec<String>,
    #[serde(default)]
    pub http_api_port: Option<u16>,
    #[serde(default)]
    pub api_port: Option<u16>,
    #[serde(default)]
    pub algorithm_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    pub enabled: bool,
    pub idle_seconds_before_ramp: u64,
    pub active_use_threshold_cpu_percent: f64,
    pub high_load_threshold_percent: f64,
    pub cpu_temp_limit_c: f64,
    pub gpu_temp_limit_c: f64,
    pub reduce_factor_on_active: f64,
    pub ramp_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatesConfig {
    pub xmr_eur: Option<f64>,
    pub rvn_eur: Option<f64>,
    #[serde(default)]
    pub electricity_eur_per_kwh: Option<f64>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerConfig {
    pub assume_on_ac: bool,
    pub pause_on_battery: bool,
    pub max_package_watts: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mock_mode: bool,
    pub active_profile: ProfileId,
    pub currency: String,
    pub rates: RatesConfig,
    pub power: PowerConfig,
    pub adaptive: AdaptiveConfig,
    pub profiles: HashMap<String, ProfileDef>,
    pub wallets: Vec<Wallet>,
    pub active_wallet_ids: HashMap<String, String>,
    pub cpu: MinerChannelConfig,
    pub gpu: MinerChannelConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        serde_json::from_str(include_str!("../../config/example.config.json"))
            .expect("embedded example config must parse")
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("IdleForge")
        .join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&raw) {
                return cfg;
            }
        }
    }
    let cfg = AppConfig::default();
    let _ = save_config(&cfg);
    cfg
}

pub fn save_config(cfg: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

pub fn resolve_pool_user(template: &str, address: &str, worker: &str) -> String {
    template
        .replace("{address}", address)
        .replace("{worker}", worker)
}
