import type { AdaptiveStatus } from "../lib/types";

interface Props {
  status: AdaptiveStatus;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
}

const MODE_LABEL: Record<string, string> = {
  idle_ramp: "Leerlauf-Ramp",
  active_reduce: "Aktive Drosselung",
  temp_limit: "Temperaturlimit",
  power_limit: "Leistungslimit",
  battery_pause: "Akku-Pause",
  manual: "Manuell",
  disabled: "Aus",
};

export function AdaptiveBanner({ status, enabled, onToggle }: Props) {
  return (
    <div className="adaptive-banner">
      <div>
        <strong>Adaptiv · {MODE_LABEL[status.mode] ?? status.mode}</strong>
        <span>{status.message}</span>
        {status.system_cpu_percent != null && (
          <span style={{ display: "block", marginTop: 4 }}>
            System-CPU {Math.round(status.system_cpu_percent)}% · Leerlauf {status.idle_seconds}s ·
            Effektives Profil: {status.effective_profile === "pause" ? "Pause" : status.effective_profile}
          </span>
        )}
      </div>
      <button
        className="toggle"
        type="button"
        onClick={() => onToggle(!enabled)}
        aria-pressed={enabled}
      >
        <span className={`toggle-track ${enabled ? "on" : ""}`}>
          <span className="toggle-thumb" />
        </span>
        {enabled ? "Aktiv" : "Aus"}
      </button>
    </div>
  );
}
