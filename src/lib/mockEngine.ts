import {
  AdaptiveStatus,
  AppConfig,
  ChannelStats,
  DashboardSnapshot,
  DEFAULT_CONFIG,
  EarningsEstimate,
  HardwareInfo,
  MinerKind,
  MinerRunState,
  ProfileId,
  Wallet,
} from "./types";

function cloneConfig(): AppConfig {
  return structuredClone(DEFAULT_CONFIG);
}

function nowIso() {
  return new Date().toISOString();
}

function baseChannel(
  kind: MinerKind,
  cfg: AppConfig,
  state: MinerRunState = "stopped",
): ChannelStats {
  const channel = kind === "cpu" ? cfg.cpu : cfg.gpu;
  return {
    kind,
    state,
    hashrate_hs: 0,
    hashrate_unit: kind === "cpu" ? "H/s" : "MH/s",
    utilization_percent: null,
    temperature_c: null,
    power_w: null,
    accepted_shares: 0,
    rejected_shares: 0,
    uptime_seconds: 0,
    intensity: 0,
    error: null,
    adapter: channel.adapter,
    coin: channel.coin,
    algorithm: channel.algorithm,
  };
}

export class MockEngine {
  config: AppConfig = cloneConfig();
  private cpuRunning = false;
  private gpuRunning = false;
  private cpuStartedAt: number | null = null;
  private gpuStartedAt: number | null = null;
  private cpuShares = { accepted: 0, rejected: 0 };
  private gpuShares = { accepted: 0, rejected: 0 };
  private idleSeconds = 120;
  private tick = 0;
  private userActive = false;

  getHardware(): HardwareInfo {
    return {
      cpu_name: "AMD Ryzen 9 (erkannt · Demo)",
      gpu_name: "NVIDIA GeForce RTX (erkannt · Demo)",
      cpu_cores: 16,
      ram_gb: 32,
      on_battery: false,
      sensors_available: true,
      note: "Mock-Modus: Sensoren und Miner werden simuliert. Referenzhardware nur als Beispiel.",
    };
  }

  setProfile(profile: ProfileId) {
    this.config.active_profile = profile;
  }

  setMockMode(enabled: boolean) {
    this.config.mock_mode = enabled;
  }

  setUserActive(active: boolean) {
    this.userActive = active;
    if (active) this.idleSeconds = 0;
  }

  start(kind: MinerKind) {
    if (kind === "cpu") {
      this.cpuRunning = true;
      this.cpuStartedAt = Date.now();
      this.cpuShares = { accepted: 0, rejected: 0 };
    } else {
      this.gpuRunning = true;
      this.gpuStartedAt = Date.now();
      this.gpuShares = { accepted: 0, rejected: 0 };
    }
  }

  stop(kind: MinerKind) {
    if (kind === "cpu") {
      this.cpuRunning = false;
      this.cpuStartedAt = null;
    } else {
      this.gpuRunning = false;
      this.gpuStartedAt = null;
    }
  }

  addWallet(wallet: Wallet) {
    this.config.wallets.push(wallet);
  }

  switchWallet(kind: MinerKind, walletId: string) {
    const wallet = this.config.wallets.find((w) => w.id === walletId);
    if (!wallet) throw new Error("Wallet nicht gefunden");
    if (kind === "cpu" && wallet.coin !== this.config.cpu.coin) {
      throw new Error("Wallet-Coin passt nicht zum CPU-Miner");
    }
    if (kind === "gpu" && wallet.coin !== this.config.gpu.coin) {
      throw new Error("Wallet-Coin passt nicht zum GPU-Miner");
    }
    this.config.active_wallet_ids[kind] = walletId;
  }

  updateConfig(partial: Partial<AppConfig>) {
    this.config = { ...this.config, ...partial };
  }

  private adaptive(): AdaptiveStatus {
    const a = this.config.adaptive;
    if (!a.enabled) {
      return {
        mode: "disabled",
        effective_profile: this.config.active_profile,
        message: "Adaptiv deaktiviert — manuelles Profil aktiv.",
        idle_seconds: this.idleSeconds,
        system_cpu_percent: 18,
      };
    }

    if (this.config.power.pause_on_battery && this.getHardware().on_battery) {
      return {
        mode: "battery_pause",
        effective_profile: "pause",
        message: "Akku erkannt — Mining pausiert.",
        idle_seconds: this.idleSeconds,
        system_cpu_percent: 12,
      };
    }

    const systemCpu = this.userActive ? 48 + Math.sin(this.tick / 8) * 8 : 14 + Math.sin(this.tick / 11) * 4;

    if (systemCpu >= a.high_load_threshold_percent) {
      return {
        mode: "active_reduce",
        effective_profile: "low",
        message: "Hohe Systemlast — Intensität reduziert.",
        idle_seconds: this.idleSeconds,
        system_cpu_percent: systemCpu,
      };
    }

    if (this.userActive || systemCpu >= a.active_use_threshold_cpu_percent) {
      return {
        mode: "active_reduce",
        effective_profile: "low",
        message: "Aktive Nutzung erkannt — Drosselung aktiv.",
        idle_seconds: this.idleSeconds,
        system_cpu_percent: systemCpu,
      };
    }

    if (this.idleSeconds < a.idle_seconds_before_ramp) {
      return {
        mode: "idle_ramp",
        effective_profile: "low",
        message: `Leerlauf ${this.idleSeconds}s — Ramp läuft an…`,
        idle_seconds: this.idleSeconds,
        system_cpu_percent: systemCpu,
      };
    }

    return {
      mode: "idle_ramp",
      effective_profile: this.config.active_profile,
      message: "Leerlauf stabil — Zielprofil erreicht.",
      idle_seconds: this.idleSeconds,
      system_cpu_percent: systemCpu,
    };
  }

  private intensityFor(kind: MinerKind, adaptive: AdaptiveStatus): number {
    if (adaptive.effective_profile === "pause") return 0;
    const profile = this.config.profiles[adaptive.effective_profile];
    const base = kind === "cpu" ? profile.cpu_intensity : profile.gpu_intensity;
    if (adaptive.mode === "active_reduce") {
      return base * this.config.adaptive.reduce_factor_on_active;
    }
    return base;
  }

  private channel(kind: MinerKind, adaptive: AdaptiveStatus): ChannelStats {
    const running = kind === "cpu" ? this.cpuRunning : this.gpuRunning;
    const startedAt = kind === "cpu" ? this.cpuStartedAt : this.gpuStartedAt;
    const shares = kind === "cpu" ? this.cpuShares : this.gpuShares;
    const intensity = this.intensityFor(kind, adaptive);
    const ch = baseChannel(kind, this.config);

    if (!running) {
      return { ...ch, state: "stopped", intensity: 0 };
    }

    if (intensity <= 0.01) {
      return {
        ...ch,
        state: "paused_adaptive",
        intensity: 0,
        uptime_seconds: startedAt ? Math.floor((Date.now() - startedAt) / 1000) : 0,
        temperature_c: kind === "cpu" ? 48 : 42,
        utilization_percent: 2,
        power_w: kind === "cpu" ? 18 : 12,
      };
    }

    const jitter = 0.92 + Math.sin(this.tick / 5 + (kind === "cpu" ? 0 : 1.7)) * 0.08;
    const cpuHash = 6200 * intensity * jitter;
    const gpuHashMh = 28.5 * intensity * jitter;

    if (this.tick % 7 === 0) shares.accepted += 1;
    if (this.tick % 53 === 0) shares.rejected += 1;

    const tempBase = kind === "cpu" ? 58 : 62;
    const temp = tempBase + intensity * 18 + Math.sin(this.tick / 9) * 1.5;

    let state: MinerRunState = "running";
    if (
      (kind === "cpu" && temp >= this.config.adaptive.cpu_temp_limit_c - 2) ||
      (kind === "gpu" && temp >= this.config.adaptive.gpu_temp_limit_c - 2)
    ) {
      state = "throttled";
    }

    return {
      ...ch,
      state,
      hashrate_hs: kind === "cpu" ? cpuHash : gpuHashMh * 1e6,
      hashrate_unit: kind === "cpu" ? "H/s" : "MH/s",
      utilization_percent: Math.round(intensity * 100 * jitter),
      temperature_c: Math.round(temp * 10) / 10,
      power_w: Math.round((kind === "cpu" ? 35 + intensity * 55 : 40 + intensity * 90) * 10) / 10,
      accepted_shares: shares.accepted,
      rejected_shares: shares.rejected,
      uptime_seconds: startedAt ? Math.floor((Date.now() - startedAt) / 1000) : 0,
      intensity,
    };
  }

  private earnings(cpu: ChannelStats, gpu: ChannelStats): EarningsEstimate[] {
    const rates = this.config.rates;
    const xmrPlaceholder = rates.xmr_eur == null;
    const rvnPlaceholder = rates.rvn_eur == null;

    // Rough demo-only heuristic when rates are set; otherwise clear placeholders.
    const xmrPerDay =
      cpu.state === "running" || cpu.state === "throttled"
        ? (cpu.hashrate_hs / 1000) * 0.00012
        : null;
    const rvnPerDay =
      gpu.state === "running" || gpu.state === "throttled"
        ? (gpu.hashrate_hs / 1e6) * 0.85
        : null;

    return [
      {
        coin: "XMR",
        amount_per_day: xmrPlaceholder ? null : xmrPerDay,
        fiat_per_day:
          xmrPlaceholder || xmrPerDay == null || rates.xmr_eur == null
            ? null
            : xmrPerDay * rates.xmr_eur,
        fiat_currency: this.config.currency,
        placeholder: xmrPlaceholder,
        note: xmrPlaceholder
          ? "Kein XMR-Kurs konfiguriert — kein erfundener Preis."
          : "Schätzung aus Hashrate × manuellem Kurs (nicht live).",
      },
      {
        coin: "RVN",
        amount_per_day: rvnPlaceholder ? null : rvnPerDay,
        fiat_per_day:
          rvnPlaceholder || rvnPerDay == null || rates.rvn_eur == null
            ? null
            : rvnPerDay * rates.rvn_eur,
        fiat_currency: this.config.currency,
        placeholder: rvnPlaceholder,
        note: rvnPlaceholder
          ? "Kein RVN-Kurs konfiguriert — kein erfundener Preis."
          : "Schätzung aus Hashrate × manuellem Kurs (nicht live).",
      },
    ];
  }

  snapshot(): DashboardSnapshot {
    this.tick += 1;
    if (!this.userActive) this.idleSeconds += 1;
    else this.idleSeconds = 0;

    const adaptive = this.adaptive();
    const cpu = this.channel("cpu", adaptive);
    const gpu = this.channel("gpu", adaptive);

    return {
      mock_mode: this.config.mock_mode,
      hardware: this.getHardware(),
      cpu,
      gpu,
      adaptive,
      earnings: this.earnings(cpu, gpu),
      profile: this.config.active_profile,
      updated_at: nowIso(),
    };
  }
}

export const mockEngine = new MockEngine();
