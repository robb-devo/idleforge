import { motion } from "framer-motion";
import type { AdaptiveStatus } from "../lib/types";

interface Props {
  status: AdaptiveStatus;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  onBattery: boolean | null;
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

export function AdaptiveBanner({ status, enabled, onToggle, onBattery }: Props) {
  const powerLabel =
    onBattery == null ? "Netzteil n/v" : onBattery ? "Akku" : "Netzteil";

  return (
    <motion.div
      className="adaptive-banner"
      layout
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.35, ease: [0.22, 1, 0.36, 1] }}
    >
      <div>
        <strong>Adaptiv · {MODE_LABEL[status.mode] ?? status.mode}</strong>
        <span>{status.message}</span>
        <span style={{ display: "block", marginTop: 4 }}>
          {status.system_cpu_percent != null && (
            <>System-CPU {Math.round(status.system_cpu_percent)}% · </>
          )}
          Leerlauf {status.idle_seconds}s · Effektives Profil:{" "}
          {status.effective_profile === "pause" ? "Pause" : status.effective_profile}
          {" · "}
          <span className={`power-chip ${onBattery ? "battery" : "ac"}`}>{powerLabel}</span>
        </span>
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
    </motion.div>
  );
}
