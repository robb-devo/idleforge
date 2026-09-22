use crate::adaptive::{AdaptiveController, AdaptiveStatus};
use crate::config::{
    load_config, resolve_pool_user, save_config, AppConfig, ProfileId, Wallet,
};
use crate::miners::{
    LolMinerAdapter, MinerAdapter, MinerKind, MinerRunState, MinerStats, StartRequest, XmrigAdapter,
};
use crate::sensors::{HardwareInfo, SensorHub, SensorReading};
use parking_lot::Mutex;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct ChannelStats {
    pub kind: MinerKind,
    pub state: MinerRunState,
    pub hashrate_hs: f64,
    pub hashrate_unit: String,
    pub utilization_percent: Option<f64>,
    pub temperature_c: Option<f64>,
    pub power_w: Option<f64>,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub uptime_seconds: u64,
    pub intensity: f64,
    pub error: Option<String>,
    pub adapter: String,
    pub coin: String,
    pub algorithm: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EarningsEstimate {
    pub coin: String,
    pub amount_per_day: Option<f64>,
    pub fiat_per_day: Option<f64>,
    pub fiat_currency: String,
    pub placeholder: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PowerCostEstimate {
    pub watts: Option<f64>,
    pub eur_per_day: Option<f64>,
    pub placeholder: bool,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSnapshot {
    pub mock_mode: bool,
    pub hardware: HardwareInfo,
    pub cpu: ChannelStats,
    pub gpu: ChannelStats,
    pub adaptive: AdaptiveStatus,
    pub earnings: Vec<EarningsEstimate>,
    pub power_cost: PowerCostEstimate,
    pub profile: ProfileId,
    pub updated_at: String,
}

struct ChannelRuntime {
    desired_running: bool,
    started_at: Option<Instant>,
    intensity: f64,
    last_stats: MinerStats,
    state: MinerRunState,
    error: Option<String>,
    last_reconfigure: Option<Instant>,
}

impl ChannelRuntime {
    fn new() -> Self {
        Self {
            desired_running: false,
            started_at: None,
            intensity: 0.0,
            last_stats: MinerStats {
                hashrate_hs: 0.0,
                utilization_percent: None,
                temperature_c: None,
                power_w: None,
                accepted_shares: 0,
                rejected_shares: 0,
            },
            state: MinerRunState::Stopped,
            error: None,
            last_reconfigure: None,
        }
    }
}

pub struct AppStateInner {
    pub config: AppConfig,
    sensors: SensorHub,
    adaptive: AdaptiveController,
    cpu: Box<dyn MinerAdapter>,
    gpu: Box<dyn MinerAdapter>,
    cpu_rt: ChannelRuntime,
    gpu_rt: ChannelRuntime,
    last_sensors: SensorReading,
    hardware_cache: Option<HardwareInfo>,
}

impl AppStateInner {
    pub fn new() -> Self {
        let config = load_config();
        Self {
            config,
            sensors: SensorHub::new(),
            adaptive: AdaptiveController::new(),
            cpu: Box::new(XmrigAdapter::new()),
            gpu: Box::new(LolMinerAdapter::new()),
            cpu_rt: ChannelRuntime::new(),
            gpu_rt: ChannelRuntime::new(),
            last_sensors: SensorReading::default(),
            hardware_cache: None,
        }
    }

    pub fn hardware(&mut self) -> HardwareInfo {
        if self.hardware_cache.is_none() {
            self.hardware_cache = Some(self.sensors.hardware());
        }
        let mut hw = self.hardware_cache.clone().unwrap();
        // Names stay cached; live load and power source refresh every snapshot.
        hw.system_cpu_percent = self.last_sensors.system_cpu_percent;
        hw.on_battery = self.last_sensors.on_battery.or(hw.on_battery);
        if hw.note.is_empty() {
            hw.note = "Hardware via Tauri-Sensoren (echte Gerätenamen).".into();
        }
        hw
    }

    fn profile_key(&self) -> String {
        self.config.active_profile.as_str().to_string()
    }

    fn profile_params(&self, kind: MinerKind, effective: &str) -> (f64, f64, f64) {
        if effective == "pause" {
            return (0.0, 0.0, 0.0);
        }
        let key = if self.config.profiles.contains_key(effective) {
            effective.to_string()
        } else {
            self.profile_key()
        };
        let p = self
            .config
            .profiles
            .get(&key)
            .or_else(|| self.config.profiles.get("medium"))
            .expect("profiles must contain medium");
        let (intensity, threads, power) = match kind {
            MinerKind::Cpu => (p.cpu_intensity, p.cpu_threads_ratio, p.gpu_power_limit_percent),
            MinerKind::Gpu => (p.gpu_intensity, p.cpu_threads_ratio, p.gpu_power_limit_percent),
        };
        (
            crate::safety::clamp_ratio(intensity),
            crate::safety::clamp_ratio(threads),
            crate::safety::clamp_power_percent(power),
        )
    }

    fn wallet_for(&self, kind: MinerKind) -> Option<&Wallet> {
        let id_key = match kind {
            MinerKind::Cpu => "cpu",
            MinerKind::Gpu => "gpu",
        };
        let id = self.config.active_wallet_ids.get(id_key)?;
        self.config.wallets.iter().find(|w| &w.id == id)
    }

    fn build_start(
        &self,
        kind: MinerKind,
        intensity: f64,
        threads: f64,
        power_pct: f64,
    ) -> Result<StartRequest, String> {
        let channel = match kind {
            MinerKind::Cpu => &self.config.cpu,
            MinerKind::Gpu => &self.config.gpu,
        };
        let wallet = self
            .wallet_for(kind)
            .ok_or_else(|| "Keine aktive Wallet für diesen Kanal.".to_string())?;
        if wallet.receive_address.starts_with("YOUR_") && !self.config.mock_mode {
            return Err(
                "Receive-Adresse ist noch ein Platzhalter. Bitte in Wallets konfigurieren.".into(),
            );
        }
        let user = resolve_pool_user(
            &channel.pool.user_template,
            &wallet.receive_address,
            &wallet.pool_worker,
        );
        let api_port = match kind {
            MinerKind::Cpu => channel.http_api_port.unwrap_or(18088),
            MinerKind::Gpu => channel.api_port.unwrap_or(18089),
        };
        let mut req = StartRequest {
            binary_path: channel.binary_path.clone(),
            pool_url: channel.pool.url.clone(),
            user,
            pass: channel.pool.pass.clone(),
            algorithm: channel.algorithm.clone(),
            intensity,
            threads_ratio: threads,
            power_limit_percent: power_pct,
            api_port,
            extra_args: channel.extra_args.clone(),
            mock_mode: self.config.mock_mode,
            tls: channel.pool.tls,
        };
        crate::safety::clamp_start(&mut req);
        Ok(req)
    }

    pub fn start(&mut self, kind: MinerKind) -> Result<(), String> {
        let status = self.current_adaptive();
        let factor = AdaptiveController::intensity_factor(&status, &self.config.adaptive).clamp(0.0, 1.0);
        let (base_i, threads, power_pct) = self.profile_params(kind, &status.effective_profile);
        let intensity = crate::safety::clamp_ratio(base_i * factor);
        let req = self.build_start(kind, intensity, threads, power_pct)?;

        match kind {
            MinerKind::Cpu => {
                self.cpu_rt.state = MinerRunState::Starting;
                self.cpu.start(&req)?;
                self.cpu_rt.desired_running = true;
                self.cpu_rt.started_at = Some(Instant::now());
                self.cpu_rt.intensity = intensity;
                self.cpu_rt.state = if intensity <= 0.01 {
                    MinerRunState::PausedAdaptive
                } else {
                    MinerRunState::Running
                };
                self.cpu_rt.error = None;
            }
            MinerKind::Gpu => {
                self.gpu_rt.state = MinerRunState::Starting;
                self.gpu.start(&req)?;
                self.gpu_rt.desired_running = true;
                self.gpu_rt.started_at = Some(Instant::now());
                self.gpu_rt.intensity = intensity;
                self.gpu_rt.state = if intensity <= 0.01 {
                    MinerRunState::PausedAdaptive
                } else {
                    MinerRunState::Running
                };
                self.gpu_rt.error = None;
            }
        }
        Ok(())
    }

    pub fn stop(&mut self, kind: MinerKind) -> Result<(), String> {
        match kind {
            MinerKind::Cpu => {
                self.cpu.stop()?;
                self.cpu_rt = ChannelRuntime::new();
            }
            MinerKind::Gpu => {
                self.gpu.stop()?;
                self.gpu_rt = ChannelRuntime::new();
            }
        }
        Ok(())
    }

    fn current_adaptive(&mut self) -> AdaptiveStatus {
        self.last_sensors = self.sensors.read();
        let cpu_temp = self
            .cpu_rt
            .last_stats
            .temperature_c
            .or(self.last_sensors.cpu_temp_c);
        let gpu_temp = self
            .gpu_rt
            .last_stats
            .temperature_c
            .or(self.last_sensors.gpu_temp_c);
        self.adaptive.evaluate(
            &self.config.adaptive,
            &self.config.power,
            &self.config.active_profile,
            &self.last_sensors,
            cpu_temp,
            gpu_temp,
        )
    }

    fn refresh_channel(&mut self, kind: MinerKind, adaptive: &AdaptiveStatus) {
        let factor =
            AdaptiveController::intensity_factor(adaptive, &self.config.adaptive).clamp(0.0, 1.0);
        let (base_i, threads, power_pct) =
            self.profile_params(kind, &adaptive.effective_profile);
        let target_intensity = crate::safety::clamp_ratio(base_i * factor);

        let desired = match kind {
            MinerKind::Cpu => self.cpu_rt.desired_running,
            MinerKind::Gpu => self.gpu_rt.desired_running,
        };
        if !desired {
            match kind {
                MinerKind::Cpu => self.cpu_rt.state = MinerRunState::Stopped,
                MinerKind::Gpu => self.gpu_rt.state = MinerRunState::Stopped,
            }
            return;
        }

        let current_intensity = match kind {
            MinerKind::Cpu => self.cpu_rt.intensity,
            MinerKind::Gpu => self.gpu_rt.intensity,
        };

        let last_reconfigure = match kind {
            MinerKind::Cpu => self.cpu_rt.last_reconfigure,
            MinerKind::Gpu => self.gpu_rt.last_reconfigure,
        };
        let reconfigure_due = last_reconfigure
            .map(|t| t.elapsed() >= std::time::Duration::from_secs(30))
            .unwrap_or(true);
        if reconfigure_due && (current_intensity - target_intensity).abs() > 0.12 {
            if let Ok(req) = self.build_start(kind, target_intensity, threads, power_pct) {
                let result = match kind {
                    MinerKind::Cpu => self.cpu.start(&req),
                    MinerKind::Gpu => self.gpu.start(&req),
                };
                if result.is_ok() {
                    let now = Instant::now();
                    match kind {
                        MinerKind::Cpu => {
                            self.cpu_rt.intensity = target_intensity;
                            self.cpu_rt.last_reconfigure = Some(now);
                        }
                        MinerKind::Gpu => {
                            self.gpu_rt.intensity = target_intensity;
                            self.gpu_rt.last_reconfigure = Some(now);
                        }
                    }
                }
            }
        }

        let poll = match kind {
            MinerKind::Cpu => self.cpu.poll_stats(),
            MinerKind::Gpu => self.gpu.poll_stats(),
        };
        let last_error = match kind {
            MinerKind::Cpu => self.cpu.last_error(),
            MinerKind::Gpu => self.gpu.last_error(),
        };

        let rt = match kind {
            MinerKind::Cpu => &mut self.cpu_rt,
            MinerKind::Gpu => &mut self.gpu_rt,
        };

        match poll {
            Ok(stats) => {
                rt.last_stats = stats;
                rt.error = last_error;
            }
            Err(e) => {
                rt.error = Some(e);
                rt.state = MinerRunState::Error;
                return;
            }
        }

        if target_intensity <= 0.01 {
            rt.state = MinerRunState::PausedAdaptive;
            rt.intensity = 0.0;
        } else if matches!(adaptive.mode, crate::adaptive::AdaptiveMode::TempLimit) {
            rt.state = MinerRunState::Throttled;
            rt.intensity = target_intensity;
        } else {
            rt.state = MinerRunState::Running;
            rt.intensity = target_intensity;
        }

        if rt.last_stats.temperature_c.is_none() {
            rt.last_stats.temperature_c = match kind {
                MinerKind::Cpu => self.last_sensors.cpu_temp_c,
                MinerKind::Gpu => self.last_sensors.gpu_temp_c,
            };
        }
        if rt.last_stats.power_w.is_none() {
            rt.last_stats.power_w = match kind {
                MinerKind::Cpu => self.last_sensors.cpu_power_w,
                MinerKind::Gpu => self.last_sensors.gpu_power_w,
            };
        }
        if rt.last_stats.utilization_percent.is_none() {
            rt.last_stats.utilization_percent =
                Some((rt.intensity * 100.0).min(crate::safety::MAX_POWER_PERCENT));
        }

        if self.config.mock_mode && target_intensity > 0.01 {
            let t = Instant::now().elapsed().as_secs_f64();
            let jitter = (t * 0.7).sin() * 1.2;
            match kind {
                MinerKind::Cpu => {
                    rt.last_stats.temperature_c = Some(58.0 + target_intensity * 18.0 + jitter);
                    rt.last_stats.power_w = Some(35.0 + target_intensity * 55.0);
                    if rt.last_stats.hashrate_hs <= 0.0 {
                        rt.last_stats.hashrate_hs = 6200.0 * target_intensity;
                    }
                }
                MinerKind::Gpu => {
                    rt.last_stats.temperature_c = Some(62.0 + target_intensity * 16.0 + jitter);
                    rt.last_stats.power_w = Some(40.0 + target_intensity * 90.0);
                    if rt.last_stats.hashrate_hs <= 0.0 {
                        rt.last_stats.hashrate_hs = 28.5e6 * target_intensity;
                    }
                }
            }
        }
    }

    fn channel_stats(&self, kind: MinerKind) -> ChannelStats {
        let (rt, channel) = match kind {
            MinerKind::Cpu => (&self.cpu_rt, &self.config.cpu),
            MinerKind::Gpu => (&self.gpu_rt, &self.config.gpu),
        };
        let uptime = rt.started_at.map(|t| t.elapsed().as_secs()).unwrap_or(0);
        ChannelStats {
            kind,
            state: rt.state.clone(),
            hashrate_hs: rt.last_stats.hashrate_hs,
            hashrate_unit: if matches!(kind, MinerKind::Gpu) {
                "MH/s".into()
            } else {
                "H/s".into()
            },
            utilization_percent: rt.last_stats.utilization_percent,
            temperature_c: rt.last_stats.temperature_c,
            power_w: rt.last_stats.power_w,
            accepted_shares: rt.last_stats.accepted_shares,
            rejected_shares: rt.last_stats.rejected_shares,
            uptime_seconds: uptime,
            intensity: rt.intensity,
            error: rt.error.clone(),
            adapter: channel.adapter.clone(),
            coin: channel.coin.clone(),
            algorithm: channel.algorithm.clone(),
        }
    }

    fn earnings(&self, cpu: &ChannelStats, gpu: &ChannelStats) -> Vec<EarningsEstimate> {
        let rates = &self.config.rates;
        let xmr_ph = rates.xmr_eur.is_none();
        let rvn_ph = rates.rvn_eur.is_none();

        let xmr_amount =
            if matches!(cpu.state, MinerRunState::Running | MinerRunState::Throttled) {
                Some((cpu.hashrate_hs / 1000.0) * 0.00012)
            } else {
                None
            };
        let rvn_amount =
            if matches!(gpu.state, MinerRunState::Running | MinerRunState::Throttled) {
                Some((gpu.hashrate_hs / 1e6) * 0.85)
            } else {
                None
            };

        vec![
            EarningsEstimate {
                coin: "XMR".into(),
                amount_per_day: if xmr_ph { None } else { xmr_amount },
                fiat_per_day: if xmr_ph {
                    None
                } else {
                    xmr_amount.and_then(|a| rates.xmr_eur.map(|r| a * r))
                },
                fiat_currency: self.config.currency.clone(),
                placeholder: xmr_ph,
                note: if xmr_ph {
                    "Kein XMR-Kurs konfiguriert — kein erfundener Preis.".into()
                } else {
                    "Schätzung aus Hashrate × manuellem Kurs.".into()
                },
            },
            EarningsEstimate {
                coin: "RVN".into(),
                amount_per_day: if rvn_ph { None } else { rvn_amount },
                fiat_per_day: if rvn_ph {
                    None
                } else {
                    rvn_amount.and_then(|a| rates.rvn_eur.map(|r| a * r))
                },
                fiat_currency: self.config.currency.clone(),
                placeholder: rvn_ph,
                note: if rvn_ph {
                    "Kein RVN-Kurs konfiguriert — kein erfundener Preis.".into()
                } else {
                    "Schätzung aus Hashrate × manuellem Kurs.".into()
                },
            },
        ]
    }

    fn power_cost(&self, cpu: &ChannelStats, gpu: &ChannelStats) -> PowerCostEstimate {
        let watts_sum = cpu.power_w.unwrap_or(0.0) + gpu.power_w.unwrap_or(0.0);
        let watts = if watts_sum > 0.0 { Some(watts_sum) } else { None };
        let rate = self.config.rates.electricity_eur_per_kwh;
        let placeholder = rate.is_none();
        PowerCostEstimate {
            watts,
            eur_per_day: if placeholder {
                None
            } else {
                watts.and_then(|w| rate.map(|r| (w / 1000.0) * 24.0 * r))
            },
            placeholder,
            note: if placeholder {
                "Kein Strompreis (€/kWh) konfiguriert — kein erfundener Preis.".into()
            } else {
                "Schätzung: Leistung × €/kWh × 24h.".into()
            },
        }
    }

    pub fn snapshot(&mut self) -> DashboardSnapshot {
        let adaptive = self.current_adaptive();
        self.refresh_channel(MinerKind::Cpu, &adaptive);
        self.refresh_channel(MinerKind::Gpu, &adaptive);
        let hardware = self.hardware();
        let cpu = self.channel_stats(MinerKind::Cpu);
        let gpu = self.channel_stats(MinerKind::Gpu);
        let earnings = self.earnings(&cpu, &gpu);
        let power_cost = self.power_cost(&cpu, &gpu);
        DashboardSnapshot {
            mock_mode: self.config.mock_mode,
            hardware,
            cpu,
            gpu,
            adaptive,
            earnings,
            power_cost,
            profile: self.config.active_profile.clone(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn set_profile(&mut self, profile: ProfileId) -> Result<(), String> {
        self.config.active_profile = profile;
        save_config(&self.config)
    }

    pub fn set_adaptive_enabled(&mut self, enabled: bool) -> Result<(), String> {
        self.config.adaptive.enabled = enabled;
        save_config(&self.config)
    }

    pub fn add_wallet(&mut self, wallet: Wallet) -> Result<(), String> {
        crate::safety::validate_receive_address(&wallet.receive_address)?;
        self.config.wallets.push(wallet);
        save_config(&self.config)
    }

    pub fn switch_wallet(&mut self, kind: MinerKind, wallet_id: String) -> Result<(), String> {
        let wallet = self
            .config
            .wallets
            .iter()
            .find(|w| w.id == wallet_id)
            .ok_or_else(|| "Wallet nicht gefunden".to_string())?
            .clone();
        let expected = match kind {
            MinerKind::Cpu => self.config.cpu.coin.clone(),
            MinerKind::Gpu => self.config.gpu.coin.clone(),
        };
        if wallet.coin != expected {
            return Err("Wallet-Coin passt nicht zum Miner-Kanal.".into());
        }
        let key = match kind {
            MinerKind::Cpu => "cpu",
            MinerKind::Gpu => "gpu",
        };
        self.config
            .active_wallet_ids
            .insert(key.into(), wallet_id);
        save_config(&self.config)
    }

    pub fn report_activity(&mut self, active: bool) {
        self.adaptive.report_activity(active);
    }

    pub fn save(&mut self, mut config: AppConfig) -> Result<(), String> {
        for wallet in &config.wallets {
            crate::safety::validate_receive_address(&wallet.receive_address)?;
        }
        crate::safety::enforce_config_caps(&mut config);
        self.config = config;
        save_config(&self.config)
    }
}

pub struct AppState(pub Mutex<AppStateInner>);
