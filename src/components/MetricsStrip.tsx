import { motion } from "framer-motion";
import type { DashboardSnapshot, EarningsEstimate } from "../lib/types";
import {
  formatHashrate,
  formatPercent,
  formatPower,
  formatTemp,
  formatUptime,
} from "../lib/format";

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
  const utilParts = [
    snapshot.cpu.utilization_percent,
    snapshot.gpu.utilization_percent,
  ].filter((u): u is number => u != null);
  const avgUtil = utilParts.length
    ? utilParts.reduce((a, b) => a + b, 0) / utilParts.length
    : null;

  const combinedHash = (() => {
    const parts: string[] = [];
    if (snapshot.cpu.hashrate_hs > 0) parts.push(formatHashrate(snapshot.cpu.hashrate_hs));
    if (snapshot.gpu.hashrate_hs > 0) parts.push(formatHashrate(snapshot.gpu.hashrate_hs, "MH/s"));
    return parts.length ? parts.join(" · ") : "—";
  })();

  const cost = snapshot.power_cost;
  const costLabel = cost.placeholder
    ? "Strompreis n/v"
    : cost.eur_per_day != null
      ? `≈ ${cost.eur_per_day.toFixed(2)} €/Tag`
      : "—";

  const cards = [
    {
      label: "Hashrate",
      value: combinedHash,
      hint: "CPU + GPU",
    },
    {
      label: "Temperatur",
      value: formatTemp(maxTemp),
      hint: "Max. über Kanäle",
    },
    {
      label: "Auslastung",
      value: formatPercent(avgUtil),
      hint: "Mittel CPU/GPU",
    },
    {
      label: "Leistung",
      value: formatPower(power && power > 0 ? power : null),
      hint: "Best-effort",
    },
    {
      label: "Laufzeit",
      value: formatUptime(totalUptime),
      hint: "Längster Kanal",
    },
    {
      label: "Ertrag / Kosten",
      value: snapshot.earnings.map((e) => `${e.coin} ${earningsLabel(e)}`).join(" · ") || "—",
      hint: `Strom ${costLabel} · keine erfundenen Kurse`,
      compact: true,
    },
  ];

  return (
    <div className="metrics-row fade-in">
      {cards.map((c, i) => (
        <motion.div
          className="metric"
          key={c.label}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: i * 0.05, duration: 0.4, ease: [0.22, 1, 0.36, 1] }}
        >
          <div className="metric-label">{c.label}</div>
          <div className="metric-value" style={c.compact ? { fontSize: "1.0rem" } : undefined}>
            {c.value}
          </div>
          <div className="metric-hint">{c.hint}</div>
        </motion.div>
      ))}
    </div>
  );
}
