import { motion } from "framer-motion";
import type { ChannelStats } from "../lib/types";
import {
  formatHashrate,
  formatPercent,
  formatPower,
  formatTemp,
  formatUptime,
  stateLabel,
} from "../lib/format";

interface Props {
  stats: ChannelStats;
  title: string;
  subtitle: string;
  accent: "cpu" | "gpu";
  onStart: () => void;
  onStop: () => void;
  busy?: boolean;
}

export function MinerPanel({
  stats,
  title,
  subtitle,
  accent,
  onStart,
  onStop,
  busy,
}: Props) {
  const running =
    stats.state === "running" ||
    stats.state === "throttled" ||
    stats.state === "paused_adaptive" ||
    stats.state === "starting";

  const displayHash =
    stats.hashrate_unit === "MH/s"
      ? formatHashrate(stats.hashrate_hs, "MH/s")
      : formatHashrate(stats.hashrate_hs);

  return (
    <motion.section
      className="panel"
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.45, ease: [0.22, 1, 0.36, 1] }}
    >
      <div className="panel-header">
        <div>
          <div className="panel-kicker" style={{ color: accent === "cpu" ? "var(--cpu)" : "var(--gpu)" }}>
            {accent === "cpu" ? "CPU-Kanal" : "GPU-Kanal"}
          </div>
          <h2 className="panel-title">{title}</h2>
          <div className="panel-meta">{subtitle}</div>
        </div>
        <div className={`status-pill ${stats.state}`}>{stateLabel(stats.state)}</div>
      </div>

      {stats.state === "stopped" ? (
        <div className="empty-state">
          <strong>Bereit zum Start</strong>
          <span>
            Externer Miner ({stats.adapter}) · {stats.coin} / {stats.algorithm}
          </span>
        </div>
      ) : (
        <>
          <div>
            <div className="panel-kicker">Hashrate</div>
            <div className="hash-hero">{displayHash}</div>
          </div>

          <div className="stat-grid">
            <div className="stat">
              <div className="stat-label">Auslastung</div>
              <div className="stat-value">{formatPercent(stats.utilization_percent)}</div>
            </div>
            <div className="stat">
              <div className="stat-label">Temperatur</div>
              <div className="stat-value">{formatTemp(stats.temperature_c)}</div>
            </div>
            <div className="stat">
              <div className="stat-label">Leistung</div>
              <div className="stat-value">{formatPower(stats.power_w)}</div>
            </div>
            <div className="stat">
              <div className="stat-label">Laufzeit</div>
              <div className="stat-value">{formatUptime(stats.uptime_seconds)}</div>
            </div>
            <div className="stat">
              <div className="stat-label">Shares</div>
              <div className="stat-value">
                {stats.accepted_shares}
                <span style={{ color: "var(--text-dim)" }}> / {stats.rejected_shares}</span>
              </div>
            </div>
            <div className="stat">
              <div className="stat-label">Intensität</div>
              <div className="stat-value">{Math.round(stats.intensity * 100)}%</div>
            </div>
          </div>

          <div>
            <div className="panel-kicker" style={{ marginBottom: 8 }}>
              Effektive Intensität
            </div>
            <div className="intensity-track">
              <div
                className="intensity-fill"
                style={{
                  width: `${Math.max(2, stats.intensity * 100)}%`,
                  background:
                    accent === "cpu"
                      ? "linear-gradient(90deg, rgba(126,182,255,0.25), var(--cpu))"
                      : undefined,
                }}
              />
            </div>
          </div>
        </>
      )}

      {stats.error && (
        <div style={{ color: "var(--danger)", fontSize: "0.88rem" }}>{stats.error}</div>
      )}

      <div className="panel-actions">
        {!running ? (
          <button className="btn btn-primary" onClick={onStart} disabled={busy}>
            Starten
          </button>
        ) : (
          <button className="btn btn-danger" onClick={onStop} disabled={busy}>
            Stoppen
          </button>
        )}
        <button className="btn btn-ghost btn-sm" disabled title="Konfiguration über Einstellungen">
          {stats.adapter} · {stats.algorithm}
        </button>
      </div>
    </motion.section>
  );
}
