import { useState } from "react";
import type { AppConfig } from "../lib/types";

interface Props {
  config: AppConfig;
  onSave: (next: AppConfig) => void;
}

function parseOptional(raw: string): number | null {
  const t = raw.trim();
  if (!t) return null;
  const n = Number(t.replace(",", "."));
  return Number.isFinite(n) ? n : null;
}

export function RatesEditor({ config, onSave }: Props) {
  const [xmr, setXmr] = useState(config.rates.xmr_eur?.toString() ?? "");
  const [rvn, setRvn] = useState(config.rates.rvn_eur?.toString() ?? "");
  const [kwh, setKwh] = useState(config.rates.electricity_eur_per_kwh?.toString() ?? "");

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    onSave({
      ...config,
      rates: {
        ...config.rates,
        xmr_eur: parseOptional(xmr),
        rvn_eur: parseOptional(rvn),
        electricity_eur_per_kwh: parseOptional(kwh),
        note: "Manuelle Kurse / Strompreis — keine Live-Feeds, keine erfundenen Preise.",
      },
    });
  };

  return (
    <form className="form-grid" onSubmit={submit}>
      <div className="field">
        <label>XMR / {config.currency}</label>
        <input
          value={xmr}
          onChange={(e) => setXmr(e.target.value)}
          placeholder="leer = Platzhalter"
          inputMode="decimal"
        />
      </div>
      <div className="field">
        <label>RVN / {config.currency} (optional)</label>
        <input
          value={rvn}
          onChange={(e) => setRvn(e.target.value)}
          placeholder="leer = Platzhalter"
          inputMode="decimal"
        />
      </div>
      <div className="field">
        <label>Strompreis €/kWh</label>
        <input
          value={kwh}
          onChange={(e) => setKwh(e.target.value)}
          placeholder="z. B. 0.32"
          inputMode="decimal"
          data-testid="input-electricity"
        />
      </div>
      <div className="field" style={{ justifyContent: "flex-end" }}>
        <label>&nbsp;</label>
        <button className="btn btn-primary" type="submit" data-testid="save-rates">
          Kurse speichern
        </button>
      </div>
      <p style={{ gridColumn: "1 / -1", margin: 0, color: "var(--text-dim)", fontSize: "0.82rem" }}>
        Leer lassen für Platzhalter. IdleForge holt keine Live-Kurse und erfindet keine Preise.
      </p>
    </form>
  );
}
