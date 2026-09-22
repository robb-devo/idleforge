import { useState } from "react";
import type { AppConfig, MinerKind, Wallet } from "../lib/types";
import { shortenAddress } from "../lib/format";

interface Props {
  config: AppConfig;
  onSwitch: (kind: MinerKind, walletId: string) => void;
  onAdd: (wallet: Wallet) => void;
}

export function WalletManager({ config, onSwitch, onAdd }: Props) {
  const [label, setLabel] = useState("");
  const [coin, setCoin] = useState("XMR");
  const [address, setAddress] = useState("");
  const [worker, setWorker] = useState("");

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    const receive = address.trim();
    if (!label.trim() || !receive) return;
    if (receive.split(/\s+/).length !== 1) {
      window.alert("Nur eine öffentliche Empfangsadresse — keine Seed-Phrase oder privater Schlüssel.");
      return;
    }
    onAdd({
      id: `${coin.toLowerCase()}-${Date.now()}`,
      label: label.trim(),
      coin,
      receive_address: receive,
      pool_worker: worker.trim() || "idleforge",
    });
    setLabel("");
    setAddress("");
    setWorker("");
  };

  return (
    <div>
      <div className="wallet-list">
        {config.wallets.map((w) => {
          const isCpu = config.active_wallet_ids.cpu === w.id;
          const isGpu = config.active_wallet_ids.gpu === w.id;
          return (
            <div className="wallet-row" key={w.id}>
              <div>
                <strong>{w.label}</strong>
                <div style={{ color: "var(--text-muted)", fontSize: "0.82rem", marginTop: 2 }}>
                  Worker: {w.pool_worker}
                </div>
              </div>
              <div>{w.coin}</div>
              <div className="mono">{shortenAddress(w.receive_address, 8)}</div>
              <div style={{ display: "flex", gap: 6, justifyContent: "flex-end" }}>
                {w.coin === config.cpu.coin && (
                  <button
                    className={`btn btn-sm ${isCpu ? "btn-primary" : ""}`}
                    onClick={() => onSwitch("cpu", w.id)}
                    type="button"
                  >
                    {isCpu ? "CPU aktiv" : "CPU"}
                  </button>
                )}
                {w.coin === config.gpu.coin && (
                  <button
                    className={`btn btn-sm ${isGpu ? "btn-primary" : ""}`}
                    onClick={() => onSwitch("gpu", w.id)}
                    type="button"
                  >
                    {isGpu ? "GPU aktiv" : "GPU"}
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      <form className="form-grid" onSubmit={submit}>
        <div className="field">
          <label>Bezeichnung</label>
          <input value={label} onChange={(e) => setLabel(e.target.value)} placeholder="z. B. XMR Reserve" />
        </div>
        <div className="field">
          <label>Coin</label>
          <select value={coin} onChange={(e) => setCoin(e.target.value)}>
            <option value="XMR">XMR</option>
            <option value="RVN">RVN</option>
          </select>
        </div>
        <div className="field">
          <label>Empfangsadresse</label>
          <input
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="Nur Receive-Adresse — niemals Seed/Private Key"
          />
        </div>
        <div className="field">
          <label>Pool-Worker</label>
          <input value={worker} onChange={(e) => setWorker(e.target.value)} placeholder="idleforge-worker" />
        </div>
        <div style={{ gridColumn: "1 / -1", display: "flex", gap: 10, alignItems: "center" }}>
          <button className="btn btn-primary" type="submit">
            Wallet hinzufügen
          </button>
          <span style={{ color: "var(--text-dim)", fontSize: "0.82rem" }}>
            Keine Seed-Phrasen oder privaten Schlüssel — nur Empfangsadressen.
          </span>
        </div>
      </form>
    </div>
  );
}
