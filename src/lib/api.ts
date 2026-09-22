import { mockEngine } from "./mockEngine";
import {
  AppConfig,
  DashboardSnapshot,
  HardwareInfo,
  MinerKind,
  ProfileId,
  Wallet,
} from "./types";

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

export async function getSnapshot(): Promise<DashboardSnapshot> {
  if (!isTauri()) return mockEngine.snapshot();
  return invoke<DashboardSnapshot>("get_snapshot");
}

export async function getConfig(): Promise<AppConfig> {
  if (!isTauri()) return structuredClone(mockEngine.config);
  return invoke<AppConfig>("get_config");
}

export async function getHardware(): Promise<HardwareInfo> {
  if (!isTauri()) return mockEngine.getHardware();
  return invoke<HardwareInfo>("get_hardware");
}

export async function startMiner(kind: MinerKind): Promise<void> {
  if (!isTauri()) {
    mockEngine.start(kind);
    return;
  }
  await invoke("start_miner", { kind });
}

export async function stopMiner(kind: MinerKind): Promise<void> {
  if (!isTauri()) {
    mockEngine.stop(kind);
    return;
  }
  await invoke("stop_miner", { kind });
}

export async function startAllMiners(): Promise<void> {
  await startMiner("cpu");
  await startMiner("gpu");
}

export async function stopAllMiners(): Promise<void> {
  await stopMiner("cpu");
  await stopMiner("gpu");
}

export async function setProfile(profile: ProfileId): Promise<void> {
  if (!isTauri()) {
    mockEngine.setProfile(profile);
    return;
  }
  await invoke("set_profile", { profile });
}

export async function setAdaptiveEnabled(enabled: boolean): Promise<void> {
  if (!isTauri()) {
    mockEngine.config.adaptive.enabled = enabled;
    return;
  }
  await invoke("set_adaptive_enabled", { enabled });
}

export async function addWallet(wallet: Wallet): Promise<void> {
  if (!isTauri()) {
    mockEngine.addWallet(wallet);
    return;
  }
  await invoke("add_wallet", { wallet });
}

export async function switchWallet(kind: MinerKind, walletId: string): Promise<void> {
  if (!isTauri()) {
    mockEngine.switchWallet(kind, walletId);
    return;
  }
  await invoke("switch_wallet", { kind, walletId });
}

export async function reportUserActivity(active: boolean): Promise<void> {
  if (!isTauri()) {
    mockEngine.setUserActive(active);
    return;
  }
  await invoke("report_user_activity", { active });
}

export async function saveConfig(config: AppConfig): Promise<void> {
  if (!isTauri()) {
    mockEngine.updateConfig(config);
    return;
  }
  await invoke("save_config", { config });
}

export function runningInTauri() {
  return isTauri();
}
