import { useCallback, useEffect, useState } from "react";
import {
  addWallet,
  getConfig,
  getSnapshot,
  reportUserActivity,
  runningInTauri,
  saveConfig,
  setAdaptiveEnabled,
  setProfile,
  startAllMiners,
  startMiner,
  stopAllMiners,
  stopMiner,
  switchWallet,
} from "./lib/api";
import type { AppConfig, DashboardSnapshot, MinerKind, ProfileId, Wallet } from "./lib/types";
import { AdaptiveBanner } from "./components/AdaptiveBanner";
import { MetricsStrip } from "./components/MetricsStrip";
import { MinerPanel } from "./components/MinerPanel";
import { ProfileSelector } from "./components/ProfileSelector";
import { RatesEditor } from "./components/RatesEditor";
import { WalletManager } from "./components/WalletManager";

type View = "dashboard" | "wallets" | "settings";

export default function App() {
  const [view, setView] = useState<View>("dashboard");
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const tauri = runningInTauri();

  const refresh = useCallback(async () => {
    try {
      const [snap, cfg] = await Promise.all([getSnapshot(), getConfig()]);
      setSnapshot(snap);
      setConfig(cfg);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), 1000);
    return () => window.clearInterval(id);
  }, [refresh]);

  useEffect(() => {
    let idleTimer: number | undefined;
    const markActive = () => {
      void reportUserActivity(true);
      window.clearTimeout(idleTimer);
      idleTimer = window.setTimeout(() => void reportUserActivity(false), 4000);
    };
    const events: Array<keyof WindowEventMap> = ["mousemove", "keydown", "pointerdown"];
    events.forEach((ev) => window.addEventListener(ev, markActive));
    idleTimer = window.setTimeout(() => void reportUserActivity(false), 2000);
    return () => {
      events.forEach((ev) => window.removeEventListener(ev, markActive));
      window.clearTimeout(idleTimer);
    };
  }, []);

  const withBusy = async (fn: () => Promise<void>) => {
    setBusy(true);
    try {
      await fn();
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  if (!snapshot || !config) {
    return (
      <div className="loading-shell">
        <div>
          <div className="loader" />
          <div className="loader-caption">IdleForge wird geladen…</div>
        </div>
      </div>
    );
  }

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">IdleForge</div>
          <div className="brand-sub">Mining Control</div>
        </div>

        <nav className="nav">
          <button
            className={`nav-item ${view === "dashboard" ? "active" : ""}`}
            onClick={() => setView("dashboard")}
            type="button"
            data-testid="nav-dashboard"
          >
            Übersicht
          </button>
          <button
            className={`nav-item ${view === "wallets" ? "active" : ""}`}
            onClick={() => setView("wallets")}
            type="button"
            data-testid="nav-wallets"
          >
            Wallets
          </button>
          <button
            className={`nav-item ${view === "settings" ? "active" : ""}`}
            onClick={() => setView("settings")}
            type="button"
            data-testid="nav-settings"
          >
            Einstellungen
          </button>
        </nav>

        <div className="sidebar-footer">
          <div className="badge">
            <span className={`badge-dot ${snapshot.mock_mode ? "mock" : ""}`} />
            {snapshot.mock_mode ? "Mock-Modus" : "Live"}
          </div>
          <div style={{ fontSize: "0.78rem", color: "var(--text-dim)", lineHeight: 1.45 }}>
            {tauri ? "Tauri-Backend verbunden" : "Browser-Demo ohne native Sensoren"}
          </div>
        </div>
      </aside>

      <main className="main">
        <header className="topbar">
          <div>
            <h1>
              {view === "dashboard" && "Übersicht"}
              {view === "wallets" && "Wallets"}
              {view === "settings" && "Einstellungen"}
            </h1>
            <p>
              {view === "dashboard" &&
                "Ruhige Steuerung für externe Miner — CPU Phase‑1 (XMRig), GPU-Adapter bereit, adaptiv und ohne Algorithmen im Repo."}
              {view === "wallets" &&
                "Empfangsadressen und Pool-Worker verwalten. Niemals Seed-Phrasen oder private Schlüssel."}
              {view === "settings" &&
                "Profile, Pfade, Strompreis und adaptive Regeln. Siehe ARCHITECTURE.md und config/example.config.json."}
            </p>
          </div>
          {view === "dashboard" && (
            <div className="topbar-actions">
              <button
                className="btn btn-primary"
                type="button"
                data-testid="start-all"
                disabled={busy}
                onClick={() => void withBusy(() => startAllMiners())}
              >
                Alles starten
              </button>
              <button
                className="btn btn-danger"
                type="button"
                data-testid="stop-all"
                disabled={busy}
                onClick={() => void withBusy(() => stopAllMiners())}
              >
                Alles stoppen
              </button>
            </div>
          )}
        </header>

        {error && (
          <div
            style={{
              marginBottom: 16,
              padding: "12px 14px",
              borderRadius: 10,
              background: "var(--danger-soft)",
              border: "1px solid rgba(224,108,117,0.3)",
              color: "#f0a8ae",
            }}
          >
            {error}
          </div>
        )}

        {view === "dashboard" && (
          <>
            <AdaptiveBanner
              status={snapshot.adaptive}
              enabled={config.adaptive.enabled}
              onBattery={snapshot.hardware.on_battery}
              onToggle={(enabled) => void withBusy(() => setAdaptiveEnabled(enabled))}
            />

            <MetricsStrip snapshot={snapshot} />

            <div className="panel-grid">
              <MinerPanel
                stats={snapshot.cpu}
                title="XMRig · Monero"
                subtitle={`${snapshot.hardware.cpu_name} · RandomX`}
                accent="cpu"
                busy={busy}
                onStart={() => void withBusy(() => startMiner("cpu"))}
                onStop={() => void withBusy(() => stopMiner("cpu"))}
              />
              <MinerPanel
                stats={snapshot.gpu}
                title="GPU-Adapter · Scaffold"
                subtitle={`${snapshot.hardware.gpu_name} · KawPow-fähig`}
                accent="gpu"
                busy={busy}
                onStart={() => void withBusy(() => startMiner("gpu"))}
                onStop={() => void withBusy(() => stopMiner("gpu"))}
              />
            </div>

            <section className="section">
              <div className="section-header">
                <div>
                  <h2>Leistungsprofile</h2>
                  <p>Idle bis Extrem — Adaptive Regeln können unter das Zielprofil drosseln.</p>
                </div>
              </div>
              <ProfileSelector
                profiles={config.profiles}
                active={config.active_profile}
                onChange={(id: ProfileId) => void withBusy(() => setProfile(id))}
              />
            </section>
          </>
        )}

        {view === "wallets" && (
          <section className="section fade-in">
            <div className="section-header">
              <div>
                <h2>Multi-Wallet</h2>
                <p>Adressen hinzufügen und CPU/GPU-Kanälen zuweisen.</p>
              </div>
            </div>
            <WalletManager
              config={config}
              onSwitch={(kind: MinerKind, walletId: string) =>
                void withBusy(() => switchWallet(kind, walletId))
              }
              onAdd={(wallet: Wallet) => void withBusy(() => addWallet(wallet))}
            />
          </section>
        )}

        {view === "settings" && (
          <div className="fade-in" style={{ display: "grid", gap: 16 }}>
            <section className="section">
              <div className="section-header">
                <div>
                  <h2>Hardware</h2>
                  <p>{snapshot.hardware.note}</p>
                </div>
              </div>
              <div className="stat-grid">
                <div className="stat">
                  <div className="stat-label">CPU</div>
                  <div className="stat-value" style={{ fontSize: "0.92rem" }}>
                    {snapshot.hardware.cpu_name}
                  </div>
                </div>
                <div className="stat">
                  <div className="stat-label">GPU</div>
                  <div className="stat-value" style={{ fontSize: "0.92rem" }}>
                    {snapshot.hardware.gpu_name}
                  </div>
                </div>
                <div className="stat">
                  <div className="stat-label">Netzteil / Akku</div>
                  <div className="stat-value">
                    {snapshot.hardware.on_battery == null
                      ? "n/v"
                      : snapshot.hardware.on_battery
                        ? "Akku"
                        : "Netzteil"}
                  </div>
                </div>
                <div className="stat">
                  <div className="stat-label">Sensoren</div>
                  <div className="stat-value">
                    {snapshot.hardware.sensors_available ? "Verfügbar" : "Degradiert"}
                  </div>
                </div>
              </div>
            </section>

            <section className="section">
              <div className="section-header">
                <div>
                  <h2>Miner-Pfade</h2>
                  <p>Externe Binaries — keine Miner im Repository.</p>
                </div>
              </div>
              <div className="stat-grid">
                <div className="stat">
                  <div className="stat-label">CPU (XMRig)</div>
                  <div className="stat-value" style={{ fontSize: "0.82rem", wordBreak: "break-all" }}>
                    {config.cpu.binary_path}
                  </div>
                </div>
                <div className="stat">
                  <div className="stat-label">GPU (Adapter)</div>
                  <div className="stat-value" style={{ fontSize: "0.82rem", wordBreak: "break-all" }}>
                    {config.gpu.binary_path}
                  </div>
                </div>
              </div>
            </section>

            <section className="section">
              <div className="section-header">
                <div>
                  <h2>Kurse & Strompreis</h2>
                  <p>Manuell konfigurierbar — Ertrag und Stromkosten werden daraus geschätzt.</p>
                </div>
              </div>
              <RatesEditor
                config={config}
                onSave={(next) => void withBusy(() => saveConfig(next))}
              />
              <div className="stat-grid" style={{ marginTop: 14 }}>
                <div className="stat">
                  <div className="stat-label">Kosten (geschätzt)</div>
                  <div className="stat-value" style={{ fontSize: "0.9rem" }}>
                    {snapshot.power_cost.placeholder
                      ? "Strompreis n/v"
                      : snapshot.power_cost.eur_per_day != null
                        ? `≈ ${snapshot.power_cost.eur_per_day.toFixed(2)} €/Tag`
                        : "—"}
                  </div>
                </div>
                <div className="stat">
                  <div className="stat-label">Hinweis</div>
                  <div className="stat-value" style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>
                    {snapshot.power_cost.note}
                  </div>
                </div>
              </div>
            </section>

            <section className="section">
              <div className="section-header">
                <div>
                  <h2>Adaptive Regeln</h2>
                  <p>Temp-, Last- und Netzteil-Schutz — siehe ARCHITECTURE.md.</p>
                </div>
              </div>
              <div className="stat-grid">
                <div className="stat">
                  <div className="stat-label">Leerlauf bis Ramp</div>
                  <div className="stat-value">{config.adaptive.idle_seconds_before_ramp}s</div>
                </div>
                <div className="stat">
                  <div className="stat-label">Aktiv-Schwelle</div>
                  <div className="stat-value">{config.adaptive.active_use_threshold_cpu_percent}%</div>
                </div>
                <div className="stat">
                  <div className="stat-label">CPU-Temp-Limit</div>
                  <div className="stat-value">{config.adaptive.cpu_temp_limit_c}°C</div>
                </div>
                <div className="stat">
                  <div className="stat-label">GPU-Temp-Limit</div>
                  <div className="stat-value">{config.adaptive.gpu_temp_limit_c}°C</div>
                </div>
                <div className="stat">
                  <div className="stat-label">Akku-Pause</div>
                  <div className="stat-value">{config.power.pause_on_battery ? "Ja" : "Nein"}</div>
                </div>
                <div className="stat">
                  <div className="stat-label">Drossel-Faktor</div>
                  <div className="stat-value">{config.adaptive.reduce_factor_on_active}</div>
                </div>
              </div>
            </section>
          </div>
        )}
      </main>
    </div>
  );
}
