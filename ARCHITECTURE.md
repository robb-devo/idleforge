# IdleForge — Architecture & UI Plan

Windows-only hobby desktop app that **orchestrates external miners**.  
No mining algorithms, binaries, seed phrases, or private keys in this repository.

This document is the design source of truth for the MVP. Implementation follows it.

---

## Product feel

Premium dark gaming/tech control surface — calm, finished, German labels.  
Clear **cards** for metrics and miner channels. Fluid motion for state changes.  
Not a debug console, not RGB clutter.

## UI structure

```
┌──────── Sidebar ────────┬────────── Main ──────────────────────────┐
│ IdleForge               │ Header + Alles starten / Alles stoppen   │
│ · Übersicht             │ Adaptive banner + Netzteil/Akku chip     │
│ · Wallets               │ Metric cards (6):                        │
│ · Einstellungen         │   Hashrate · Temp · Util · Power ·       │
│                         │   Laufzeit · Ertrag/Kosten               │
│ [Mock/Live]             │ CPU card          │ GPU card             │
│                         │ Profiles: Idle → Extreme (5)             │
└─────────────────────────┴──────────────────────────────────────────┘
```

| View | Purpose |
|------|---------|
| **Übersicht** | Live control: metrics, CPU/GPU, profiles, adaptive |
| **Wallets** | Multi receive-address + pool worker; assign to CPU/GPU |
| **Einstellungen** | Hardware, miner paths, editable €/kWh & rates, adaptive limits |

### Cards (Übersicht)

1. **Hashrate** — combined CPU + GPU  
2. **Temperatur** — max across channels (temp protection feeds adaptive)  
3. **Auslastung** — mean utilization  
4. **Leistung** — watts best-effort  
5. **Laufzeit** — longest channel uptime  
6. **Ertrag / Kosten** — fiat earnings vs Stromkosten; Netto when both known  

Plus dedicated **CPU** and **GPU** channel cards (independent Start/Stop).

### Frontend state

- `DashboardSnapshot` polled ~1 Hz (`MockEngine` or Tauri invoke)
- `AppConfig` for profiles, wallets, rates, adaptive, paths
- Pointer/keyboard activity → `report_user_activity` → adaptive reduce/ramp

---

## Controls (behavior)

| Control | Behavior |
|---------|----------|
| Profiles | `idle` · `low` · `medium` · `high` · `extreme` — **max 95%** |
| Start/Stop | Per channel + **Alles starten / Alles stoppen** |
| Temp protection | Adaptive → `temp_limit` → effective low / throttle |
| Netzteil / Akku | Sensors; `pause_on_battery` → pause mining |
| Active use / load | Reduce intensity (`reduce_factor_on_active`) |
| Idle | After `idle_seconds_before_ramp`, ramp to target profile |
| Economics | Manual `xmr_eur` / `rvn_eur` / `electricity_eur_per_kwh` — never invent live prices |

---

## Backend modules (`src-tauri`)

```
commands.rs     Tauri IPC
state.rs        Runtime orchestration
config.rs       %APPDATA%/IdleForge/config.json
adaptive/       Idle ramp, active reduce, temp/power/battery
safety.rs       Hard 95% cap (intensity, threads, GPU power)
sensors/        sysinfo + nvidia-smi / WMI GPU + battery
miners/
  adapter.rs    MinerAdapter trait (extensibility seam)
  xmrig.rs      Phase 1 — CPU XMR RandomX (external process)
  lolminer.rs   GPU — default ETCHASH via MoneroOcean (XMR payout); algo/pool swappable
```

### MinerAdapter

```text
start(StartRequest)  → spawn external binary (or mock)
stop()               → kill child
poll_stats()         → HTTP API / stdout → hashrate, shares, …
```

Intensity = **profile × adaptive factor**, then clamped by `safety.rs` to **≤ 95%**. Channels are independent. Adaptive factors cannot amplify past 1.0, and thread counts always leave at least one logical CPU free. XMRig is started below normal priority.

Adding a miner later = new adapter module + config `adapter` id — no algorithm code. Call `clamp_start` before spawn so new adapters inherit the cap.

### Hardware under Tauri

`SensorHub` reports the real CPU brand and a live system CPU%. GPU name comes from `nvidia-smi`, or on Windows from `Win32_VideoController`. Browser `npm run dev` uses labeled demo names. `mock_mode` simulates miner hashrate only; it does not replace Tauri hardware identity.

### Mining scope

| Phase | Scope |
|-------|--------|
| **1 (MVP)** | XMRig/XMR CPU live path; GPU fully wired in UI/state/adapter (scaffold OK) |
| **2** | Harden chosen GPU plugin; richer Windows sensors |
| **3** | Adapter registry by `config.*.adapter` |

---

## Mock mode

Default for demo: `npm run dev` runs the finished UI without Tauri or miner binaries.  
Synthetic metrics prove cards, controls, adaptive, and economics placeholders.

---

## Non-goals

- Implementing any PoW algorithm in-process  
- Committing miner executables or secrets  
- Storing seed phrases / private keys (receive addresses only)
