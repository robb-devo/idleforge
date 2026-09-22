use crate::config::{AdaptiveConfig, PowerConfig, ProfileId};
use crate::sensors::SensorReading;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum AdaptiveMode {
    IdleRamp,
    ActiveReduce,
    TempLimit,
    PowerLimit,
    BatteryPause,
    Manual,
    Disabled,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdaptiveStatus {
    pub mode: AdaptiveMode,
    pub effective_profile: String,
    pub message: String,
    pub idle_seconds: u64,
    pub system_cpu_percent: Option<f64>,
}

pub struct AdaptiveController {
    last_activity: Instant,
    user_active: bool,
}

impl AdaptiveController {
    pub fn new() -> Self {
        Self {
            last_activity: Instant::now()
                .checked_sub(std::time::Duration::from_secs(120))
                .unwrap_or_else(Instant::now),
            user_active: false,
        }
    }

    pub fn report_activity(&mut self, active: bool) {
        self.user_active = active;
        if active {
            self.last_activity = Instant::now();
        }
    }

    pub fn evaluate(
        &self,
        adaptive: &AdaptiveConfig,
        power: &PowerConfig,
        target: &ProfileId,
        sensors: &SensorReading,
        cpu_temp: Option<f64>,
        gpu_temp: Option<f64>,
    ) -> AdaptiveStatus {
        let idle_seconds = self.last_activity.elapsed().as_secs();
        let sys_cpu = sensors.system_cpu_percent;
        let on_battery = sensors.on_battery.unwrap_or(false);

        if !adaptive.enabled {
            return AdaptiveStatus {
                mode: AdaptiveMode::Disabled,
                effective_profile: target.as_str().into(),
                message: "Adaptiv deaktiviert — manuelles Profil aktiv.".into(),
                idle_seconds,
                system_cpu_percent: sys_cpu,
            };
        }

        if power.pause_on_battery && on_battery {
            return AdaptiveStatus {
                mode: AdaptiveMode::BatteryPause,
                effective_profile: "pause".into(),
                message: "Akku erkannt — Mining pausiert.".into(),
                idle_seconds,
                system_cpu_percent: sys_cpu,
            };
        }

        if let Some(t) = cpu_temp.or(sensors.cpu_temp_c) {
            if t >= adaptive.cpu_temp_limit_c {
                return AdaptiveStatus {
                    mode: AdaptiveMode::TempLimit,
                    effective_profile: "low".into(),
                    message: format!("CPU-Temperatur {t:.0}°C — Limit erreicht."),
                    idle_seconds,
                    system_cpu_percent: sys_cpu,
                };
            }
        }
        if let Some(t) = gpu_temp.or(sensors.gpu_temp_c) {
            if t >= adaptive.gpu_temp_limit_c {
                return AdaptiveStatus {
                    mode: AdaptiveMode::TempLimit,
                    effective_profile: "low".into(),
                    message: format!("GPU-Temperatur {t:.0}°C — Limit erreicht."),
                    idle_seconds,
                    system_cpu_percent: sys_cpu,
                };
            }
        }

        let package_w =
            sensors.cpu_power_w.unwrap_or(0.0) + sensors.gpu_power_w.unwrap_or(0.0);
        if (sensors.cpu_power_w.is_some() || sensors.gpu_power_w.is_some())
            && package_w >= power.max_package_watts
        {
            return AdaptiveStatus {
                mode: AdaptiveMode::PowerLimit,
                effective_profile: "low".into(),
                message: format!("Leistung {package_w:.0} W — Paketlimit."),
                idle_seconds,
                system_cpu_percent: sys_cpu,
            };
        }

        if let Some(cpu) = sys_cpu {
            if cpu >= adaptive.high_load_threshold_percent
                || cpu >= adaptive.active_use_threshold_cpu_percent
                || self.user_active
            {
                return AdaptiveStatus {
                    mode: AdaptiveMode::ActiveReduce,
                    effective_profile: "low".into(),
                    message: "Aktive Nutzung / Last erkannt — Intensität reduziert.".into(),
                    idle_seconds,
                    system_cpu_percent: sys_cpu,
                };
            }
        } else if self.user_active {
            return AdaptiveStatus {
                mode: AdaptiveMode::ActiveReduce,
                effective_profile: "low".into(),
                message: "Aktive Nutzung erkannt — Intensität reduziert.".into(),
                idle_seconds,
                system_cpu_percent: sys_cpu,
            };
        }

        if idle_seconds < adaptive.idle_seconds_before_ramp {
            return AdaptiveStatus {
                mode: AdaptiveMode::IdleRamp,
                effective_profile: "low".into(),
                message: format!("Leerlauf {idle_seconds}s — Ramp läuft an…"),
                idle_seconds,
                system_cpu_percent: sys_cpu,
            };
        }

        AdaptiveStatus {
            mode: AdaptiveMode::IdleRamp,
            effective_profile: target.as_str().into(),
            message: "Leerlauf stabil — Zielprofil erreicht.".into(),
            idle_seconds,
            system_cpu_percent: sys_cpu,
        }
    }

    pub fn intensity_factor(status: &AdaptiveStatus, adaptive: &AdaptiveConfig) -> f64 {
        match status.mode {
            AdaptiveMode::BatteryPause => 0.0,
            AdaptiveMode::ActiveReduce | AdaptiveMode::TempLimit | AdaptiveMode::PowerLimit => {
                adaptive.reduce_factor_on_active
            }
            _ => 1.0,
        }
    }
}
