import { memo } from "react";
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

function earningsFiatSum(earnings: EarningsEstimate[]): number | null {
  const vals = earnings
    .map((e) => e.fiat_per_day)
    .filter((v): v is number => v != null);
  if (!vals.length) return null;
  return vals.reduce((a, b) => a + b, 0);
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

export const MetricsStrip = memo(function MetricsStrip({ snapshot }: Props) {
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
  const earnSum = earningsFiatSum(snapshot.earnings);
  const anyEarnPlaceholder = snapshot.earnings.some((e) => e.placeholder);

  let economyValue: string;
  let economyHint: string;
  if (anyEarnPlaceholder && cost.placeholder) {
    economyValue = "Kurse n/v";
    economyHint = "Ertrag & Strompreis nicht gesetzt — keine erfundenen Preise";
  } else if (earnSum != null && cost.eur_per_day != null) {
    const net = earnSum - cost.eur_per_day;
    economyValue = `Netto ≈ ${net.toFixed(2)} €/Tag`;
    economyHint = `Ertrag ≈ ${earnSum.toFixed(2)} € · Strom ≈ ${cost.eur_per_day.toFixed(2)} €`;
  } else {
    economyValue = snapshot.earnings.map((e) => `${e.coin} ${earningsLabel(e)}`).join(" · ");
    economyHint = cost.placeholder
      ? "Strompreis n/v"
      : cost.eur_per_day != null
        ? `Strom ≈ ${cost.eur_per_day.toFixed(2)} €/Tag`
        : "Kosten —";
  }

  const cards = [
    { label: "Hashrate", value: combinedHash, hint: "CPU + GPU" },
    { label: "Temperatur", value: formatTemp(maxTemp), hint: "Max. · Temp-Schutz aktiv" },
    { label: "Auslastung", value: formatPercent(avgUtil), hint: "Mittel CPU/GPU" },
    {
      label: "Leistung",
      value: formatPower(power && power > 0 ? power : null),
      hint: "Best-effort",
    },
    { label: "Laufzeit", value: formatUptime(totalUptime), hint: "Längster Kanal" },
    { label: "Ertrag / Kosten", value: economyValue, hint: economyHint, compact: true },
  ];

  return (
    <div className="metrics-row">
      {cards.map((c) => (
        <div className="metric" key={c.label}>
          <div className="metric-label">{c.label}</div>
          <div className="metric-value" style={c.compact ? { fontSize: "1.0rem" } : undefined}>
            {c.value}
          </div>
          <div className="metric-hint">{c.hint}</div>
        </div>
      ))}
    </div>
  );
});
