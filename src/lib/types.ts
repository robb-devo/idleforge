export type ProfileId = "low" | "medium" | "high" | "extreme";

export type MinerKind = "cpu" | "gpu";

export type MinerRunState =
  | "stopped"
  | "starting"
  | "running"
  | "stopping"
  | "error"
  | "throttled"
  | "paused_adaptive";

export type AdaptiveMode =
  | "idle_ramp"
  | "active_reduce"
  | "temp_limit"
  | "power_limit"
  | "battery_pause"
  | "manual"
  | "disabled";

export interface Wallet {
  id: string;
  label: string;
  coin: string;
  receive_address: string;
  pool_worker: string;
}

export interface ProfileDef {
  label: string;
  cpu_intensity: number;
  gpu_intensity: number;
  cpu_threads_ratio: number;
  gpu_power_limit_percent: number;
}

export interface PoolConfig {
  url: string;
  tls: boolean;
  user_template: string;
  pass: string;
}

export interface MinerChannelConfig {
  enabled: boolean;
  adapter: string;
  coin: string;
  algorithm: string;
  binary_path: string;
  pool: PoolConfig;
  extra_args: string[];
  http_api_port?: number;
  api_port?: number;
  algorithm_note?: string;
}

export interface AdaptiveConfig {
  enabled: boolean;
  idle_seconds_before_ramp: number;
  active_use_threshold_cpu_percent: number;
  high_load_threshold_percent: number;
  cpu_temp_limit_c: number;
  gpu_temp_limit_c: number;
  reduce_factor_on_active: number;
  ramp_steps: string[];
}

export interface AppConfig {
  mock_mode: boolean;
  active_profile: ProfileId;
  currency: string;
  rates: {
    xmr_eur: number | null;
    rvn_eur: number | null;
    note?: string;
  };
  power: {
    assume_on_ac: boolean;
    pause_on_battery: boolean;
    max_package_watts: number;
  };
  adaptive: AdaptiveConfig;
  profiles: Record<ProfileId, ProfileDef>;
  wallets: Wallet[];
  active_wallet_ids: {
    cpu: string;
    gpu: string;
  };
  cpu: MinerChannelConfig;
  gpu: MinerChannelConfig;
}

export interface HardwareInfo {
  cpu_name: string;
  gpu_name: string;
  cpu_cores: number;
  ram_gb: number | null;
  on_battery: boolean | null;
  sensors_available: boolean;
  note: string;
}

export interface ChannelStats {
  kind: MinerKind;
  state: MinerRunState;
  hashrate_hs: number;
  hashrate_unit: string;
  utilization_percent: number | null;
  temperature_c: number | null;
  power_w: number | null;
  accepted_shares: number;
  rejected_shares: number;
  uptime_seconds: number;
  intensity: number;
  error: string | null;
  adapter: string;
  coin: string;
  algorithm: string;
}

export interface EarningsEstimate {
  coin: string;
  amount_per_day: number | null;
  fiat_per_day: number | null;
  fiat_currency: string;
  placeholder: boolean;
  note: string;
}

export interface AdaptiveStatus {
  mode: AdaptiveMode;
  effective_profile: ProfileId | "pause";
  message: string;
  idle_seconds: number;
  system_cpu_percent: number | null;
}

export interface DashboardSnapshot {
  mock_mode: boolean;
  hardware: HardwareInfo;
  cpu: ChannelStats;
  gpu: ChannelStats;
  adaptive: AdaptiveStatus;
  earnings: EarningsEstimate[];
  profile: ProfileId;
  updated_at: string;
}

export const DEFAULT_CONFIG: AppConfig = {
  mock_mode: true,
  active_profile: "medium",
  currency: "EUR",
  rates: {
    xmr_eur: null,
    rvn_eur: null,
    note: "Keine Live-Kurse hinterlegt — Platzhalter werden angezeigt.",
  },
  power: {
    assume_on_ac: true,
    pause_on_battery: true,
    max_package_watts: 120,
  },
  adaptive: {
    enabled: true,
    idle_seconds_before_ramp: 90,
    active_use_threshold_cpu_percent: 35,
    high_load_threshold_percent: 75,
    cpu_temp_limit_c: 85,
    gpu_temp_limit_c: 78,
    reduce_factor_on_active: 0.35,
    ramp_steps: ["pause", "low", "medium", "high", "extreme"],
  },
  profiles: {
    low: {
      label: "Niedrig",
      cpu_intensity: 0.35,
      gpu_intensity: 0.3,
      cpu_threads_ratio: 0.4,
      gpu_power_limit_percent: 55,
    },
    medium: {
      label: "Mittel",
      cpu_intensity: 0.6,
      gpu_intensity: 0.55,
      cpu_threads_ratio: 0.65,
      gpu_power_limit_percent: 70,
    },
    high: {
      label: "Hoch",
      cpu_intensity: 0.85,
      gpu_intensity: 0.8,
      cpu_threads_ratio: 0.85,
      gpu_power_limit_percent: 85,
    },
    extreme: {
      label: "Extrem",
      cpu_intensity: 1.0,
      gpu_intensity: 1.0,
      cpu_threads_ratio: 1.0,
      gpu_power_limit_percent: 100,
    },
  },
  wallets: [
    {
      id: "xmr-main",
      label: "XMR Hauptwallet",
      coin: "XMR",
      receive_address: "YOUR_XMR_RECEIVE_ADDRESS_HERE",
      pool_worker: "idleforge-worker-1",
    },
    {
      id: "rvn-main",
      label: "RVN Hauptwallet",
      coin: "RVN",
      receive_address: "YOUR_RVN_RECEIVE_ADDRESS_HERE",
      pool_worker: "idleforge-gpu-1",
    },
  ],
  active_wallet_ids: {
    cpu: "xmr-main",
    gpu: "rvn-main",
  },
  cpu: {
    enabled: true,
    adapter: "xmrig",
    coin: "XMR",
    algorithm: "randomx",
    binary_path: "C:\\Miners\\xmrig\\xmrig.exe",
    http_api_port: 18088,
    pool: {
      url: "pool.supportxmr.com:443",
      tls: true,
      user_template: "{address}.{worker}",
      pass: "x",
    },
    extra_args: [],
  },
  gpu: {
    enabled: true,
    adapter: "lolminer",
    coin: "RVN",
    algorithm: "kawpow",
    algorithm_note:
      "KawPow (Ravencoin) — modernes NVIDIA-Ziel mit guter lolMiner-Unterstützung.",
    binary_path: "C:\\Miners\\lolMiner\\lolMiner.exe",
    api_port: 18089,
    pool: {
      url: "stratum+ssl://rvn.2miners.com:6060",
      tls: true,
      user_template: "{address}.{worker}",
      pass: "x",
    },
    extra_args: [],
  },
};
