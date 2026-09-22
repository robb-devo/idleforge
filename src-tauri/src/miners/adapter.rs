use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MinerKind {
    Cpu,
    Gpu,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MinerRunState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
    Throttled,
    PausedAdaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStats {
    pub hashrate_hs: f64,
    pub utilization_percent: Option<f64>,
    pub temperature_c: Option<f64>,
    pub power_w: Option<f64>,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
}

#[derive(Debug, Clone)]
pub struct StartRequest {
    pub binary_path: String,
    pub pool_url: String,
    pub user: String,
    pub pass: String,
    pub algorithm: String,
    pub intensity: f64,
    pub threads_ratio: f64,
    pub power_limit_percent: f64,
    pub api_port: u16,
    pub extra_args: Vec<String>,
    pub mock_mode: bool,
    /// From `pool.tls`. Adapters also enable TLS when the URL uses port 443 or contains `ssl`.
    pub tls: bool,
}

/// TLS is required for SupportXMR (`:443`) and pools that set `pool.tls` or embed `ssl` in the URL.
pub fn pool_wants_tls(tls_flag: bool, url: &str) -> bool {
    if tls_flag {
        return true;
    }
    let url = url.trim().to_ascii_lowercase();
    url.contains("ssl") || url.ends_with(":443") || url.contains(":443/") || url.contains(":443?")
}

pub trait MinerAdapter: Send {
    #[allow(dead_code)]
    fn id(&self) -> &'static str;
    #[allow(dead_code)]
    fn kind(&self) -> MinerKind;
    fn start(&mut self, req: &StartRequest) -> Result<(), String>;
    fn stop(&mut self) -> Result<(), String>;
    #[allow(dead_code)]
    fn is_running(&self) -> bool;
    fn poll_stats(&mut self) -> Result<MinerStats, String>;
    fn last_error(&self) -> Option<String>;
}

#[cfg(test)]
mod tests {
    use super::pool_wants_tls;

    #[test]
    fn tls_for_supportxmr_and_explicit_flag() {
        assert!(pool_wants_tls(true, "pool.supportxmr.com:443"));
        assert!(pool_wants_tls(false, "pool.supportxmr.com:443"));
        assert!(pool_wants_tls(false, "stratum+ssl://pool.example.com:3333"));
        assert!(pool_wants_tls(true, "gulf.moneroocean.stream:20128"));
        assert!(!pool_wants_tls(false, "pool.example.com:3333"));
    }
}
