//! Hard stability cap: IdleForge never asks a miner for 100% of the machine.
//! Extreme and any adaptive/throttle path are clamped to 95%. At least one
//! logical CPU is always left free so Windows stays responsive.

use crate::config::{AppConfig, ProfileDef};
use crate::miners::StartRequest;

/// Maximum fraction of CPU threads or miner intensity (95%).
pub const MAX_SYSTEM_LOAD: f64 = 0.95;

/// Maximum GPU power-limit percent passed to an external miner.
pub const MAX_POWER_PERCENT: f64 = 95.0;

pub fn clamp_ratio(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    v.clamp(0.0, MAX_SYSTEM_LOAD)
}

pub fn clamp_power_percent(v: f64) -> f64 {
    if !v.is_finite() {
        return 0.0;
    }
    v.clamp(0.0, MAX_POWER_PERCENT)
}

/// Thread count for XMRig. Never uses every logical processor.
pub fn capped_thread_count(logical: usize, ratio: f64) -> u32 {
    if logical <= 1 {
        return 1;
    }
    let ratio = clamp_ratio(ratio);
    let by_ratio = ((logical as f64) * ratio).floor() as usize;
    let hard_cap = ((logical as f64) * MAX_SYSTEM_LOAD).floor() as usize;
    let max_threads = hard_cap.max(1).min(logical - 1);
    by_ratio.max(1).min(max_threads) as u32
}

pub fn clamp_profile(profile: &mut ProfileDef) {
    profile.cpu_intensity = clamp_ratio(profile.cpu_intensity);
    profile.gpu_intensity = clamp_ratio(profile.gpu_intensity);
    profile.cpu_threads_ratio = clamp_ratio(profile.cpu_threads_ratio);
    profile.gpu_power_limit_percent = clamp_power_percent(profile.gpu_power_limit_percent);
}

pub fn enforce_config_caps(cfg: &mut AppConfig) {
    for profile in cfg.profiles.values_mut() {
        clamp_profile(profile);
    }
    if !cfg.adaptive.reduce_factor_on_active.is_finite() {
        cfg.adaptive.reduce_factor_on_active = 0.35;
    }
    cfg.adaptive.reduce_factor_on_active = cfg.adaptive.reduce_factor_on_active.clamp(0.0, 1.0);
}

pub fn clamp_start(req: &mut StartRequest) {
    req.intensity = clamp_ratio(req.intensity);
    req.threads_ratio = clamp_ratio(req.threads_ratio);
    req.power_limit_percent = clamp_power_percent(req.power_limit_percent);
}

pub fn validate_receive_address(address: &str) -> Result<(), String> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        return Err("Empfangsadresse fehlt.".into());
    }
    if trimmed.split_whitespace().count() != 1 {
        return Err(
            "Nur eine öffentliche Empfangsadresse — keine Seed-Phrase und kein privater Schlüssel."
                .into(),
        );
    }
    let lower = trimmed.to_lowercase();
    for marker in ["seed", "mnemonic", "private", "xprv", "secret key"] {
        if lower.contains(marker) {
            return Err("Seed-Phrasen und private Schlüssel sind nicht erlaubt.".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extreme_never_takes_every_thread() {
        for cores in [2usize, 4, 8, 12, 16, 24, 32] {
            let n = capped_thread_count(cores, 1.0);
            assert!(n >= 1);
            assert!(n < cores as u32, "{n} threads on {cores} cores");
            assert!(
                (n as f64) / (cores as f64) <= MAX_SYSTEM_LOAD + f64::EPSILON,
                "{n}/{cores} exceeds 95%"
            );
        }
    }

    #[test]
    fn ratios_cannot_exceed_cap() {
        assert_eq!(clamp_ratio(1.0), MAX_SYSTEM_LOAD);
        assert_eq!(clamp_ratio(1.5), MAX_SYSTEM_LOAD);
        assert_eq!(clamp_power_percent(100.0), MAX_POWER_PERCENT);
    }
}
