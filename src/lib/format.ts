export function formatHashrate(hs: number, unitHint?: string): string {
  if (!hs || hs <= 0) return "—";
  if (unitHint === "MH/s" || hs >= 1e6) {
    return `${(hs / 1e6).toFixed(2)} MH/s`;
  }
  if (hs >= 1e3) return `${(hs / 1e3).toFixed(2)} kH/s`;
  return `${hs.toFixed(0)} H/s`;
}

export function formatUptime(seconds: number): string {
  if (seconds <= 0) return "0s";
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  if (h > 0) return `${h}h ${m.toString().padStart(2, "0")}m`;
  if (m > 0) return `${m}m ${s.toString().padStart(2, "0")}s`;
  return `${s}s`;
}

export function formatTemp(c: number | null): string {
  if (c == null) return "n/v";
  return `${c.toFixed(0)}°C`;
}

export function formatPower(w: number | null): string {
  if (w == null) return "n/v";
  return `${w.toFixed(0)} W`;
}

export function formatPercent(p: number | null): string {
  if (p == null) return "n/v";
  return `${Math.round(p)}%`;
}

export function shortenAddress(addr: string, size = 6): string {
  if (!addr || addr.startsWith("YOUR_")) return "Nicht konfiguriert";
  if (addr.length <= size * 2 + 1) return addr;
  return `${addr.slice(0, size)}…${addr.slice(-size)}`;
}

export function stateLabel(state: string): string {
  switch (state) {
    case "stopped":
      return "Gestoppt";
    case "starting":
      return "Startet…";
    case "running":
      return "Läuft";
    case "stopping":
      return "Stoppt…";
    case "error":
      return "Fehler";
    case "throttled":
      return "Gedrosselt";
    case "paused_adaptive":
      return "Adaptiv pausiert";
    default:
      return state;
  }
}
