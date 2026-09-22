import type { DashboardSnapshot, EarningsEstimate } from "../lib/types";
import { formatHashrate, formatPower, formatTemp, formatUptime } from "../lib/format";

interface Props {
  snapshot: DashboardSnapshot;
}

function earningsLabel(e: EarningsEstimate): string {
  if (e.placeholder) return "Kurs n/v";
  if (e.fiat_per_day != null) {
    return `≈ ${e.fiat_per_day.toFixed(2)} ${e.fiat_currency}/Tag`;
  }
  if (e.amount_per_day != null) {
    return `≈ ${e.amount_per_day.toFixed(4)} ${e.coin}/Tag`;
  }
  return "—";
}

export function MetricsStrip({ snapshot }: Props) {
  const totalUptime = Math.max(snapshot.cpu.uptime_seconds, snapshot.gpu.uptime_seconds);
  const power =
    (snapshot.cpu.power_w ?? 0) + (snapshot.gpu.power_w ?? 0) || null;
  const tempParts = [snapshot.cpu.temperature_c, snapshot.gpu.temperature_c].filter(
    (t): t is number => t != null,
  );
  const maxTemp = tempParts.length ? Math.max(...tempParts) : null;

  const combinedHash = (() => {
    const parts: string[] = [];
    if (snapshot.cpu.hashrate_hs > 0) parts.push(formatHashrate(snapshot.cpu.hashrate_hs));
    if (snapshot.gpu.hashrate_hs > 0) parts.push(formatHashrate(snapshot.gpu.hashrate_hs, "MH/s"));
    return parts.length ? parts.join(" · ") : "—";
  })();

  return (
    <div className="metrics-row fade-in">
      <div className="metric">
        <div className="metric-label">Gesamt-Hashrate</div>
        <div className="metric-value">{combinedHash}</div>
        <div className="metric-hint">CPU + GPU Kanäle</div>
      </div>
      <div className="metric">
        <div className="metric-label">Temperatur</div>
        <div className="metric-value">{formatTemp(maxTemp)}</div>
        <div className="metric-hint">Max. über Kanäle · best-effort</div>
      </div>
      <div className="metric">
        <div className="metric-label">Leistung</div>
        <div className="metric-value">{formatPower(power && power > 0 ? power : null)}</div>
        <div className="metric-hint">Geschätzt · best-effort</div>
      </div>
      <div className="metric">
        <div className="metric-label">Ertrag (geschätzt)</div>
        <div className="metric-value" style={{ fontSize: "1.05rem" }}>
          {snapshot.earnings.map((e) => `${e.coin} ${earningsLabel(e)}`).join(" · ") || "—"}
        </div>
        <div className="metric-hint">
          Laufzeit {formatUptime(totalUptime)} · keine erfundenen Live-Preise
        </div>
      </div>
    </div>
  );
}
