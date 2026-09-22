# IdleForge — Architecture

Windows-only hobby desktop app that **orchestrates external miners**. No mining algorithms, binaries, or secrets live in this repository.

## Goals

| Goal | Approach |
|------|----------|
| Premium finished feel | Tauri 2 + React dark UI, German labels, motion |
| Safe by default | Receive addresses only; mock mode without miners |
| Extensible | `MinerAdapter` trait — plug in new coins/miners later |
| Device-agnostic | Detect hardware; user supplies binaries, pools, wallets |

## UI structure

```
┌──────── Sidebar ────────┬────────── Main ──────────────────────────┐
│ IdleForge               │ Header (view title)                      │
│ · Übersicht             │ Adaptive status banner                   │
│ · Wallets               │ Metric cards: Hashrate · Temp · Util ·   │
│ · Einstellungen         │   Power · Uptime · Ertrag/Kosten         │
│                         │ CPU card  │  GPU card                    │
│ [Mock/Live badge]       │ Profiles: Idle→Extreme                   │
│                         │ Start All / Stop All                     │
└─────────────────────────┴──────────────────────────────────────────┘
```

**Views**

1. **Übersicht** — live control surface (primary)
2. **Wallets** — multi receive-address + pool worker switcher
3. **Einstellungen** — paths, rates, €/kWh, adaptive limits, hardware

**State model (frontend)**

- `DashboardSnapshot` polled ~1 Hz (Tauri invoke or `MockEngine`)
- Local `AppConfig` mirror for wallets/profiles/settings
- User activity → `report_user_activity` for adaptive reduce/ramp

## Backend modules (`src-tauri`)

```
commands.rs     Tauri IPC surface
state.rs        Runtime: miners + adaptive + sensors + config
config.rs       Load/save %APPDATA%/IdleForge/config.json
adaptive/       Idle ramp, active reduce, temp/power/battery rules
sensors/        sysinfo + nvidia-smi + battery (graceful degradation)
miners/
  adapter.rs    MinerAdapter trait
  xmrig.rs      CPU · XMR · RandomX (Phase 1)
  lolminer.rs   GPU · KawPow scaffold (replaceable plugin)
```

### MinerAdapter

```text
start(StartRequest) → spawn external process (or mock)
stop()              → kill process
poll_stats()        → HTTP API / stdout → hashrate, shares, …
```

Intensity comes from **profile × adaptive factor**. CPU and GPU channels are independent.

### Profiles (5)

`idle` → `low` → `medium` → `high` → `extreme`

Adaptive may force a lower effective profile or `pause` (battery / hard limits).

### Economics (honest placeholders)

- Optional fiat rates for coins (`xmr_eur`, …)
- Optional `electricity_eur_per_kwh`
- UI shows **estimated earnings** and **power cost** only when rates are set; otherwise clear “Kurs n/v” / “Strompreis n/v” — never invent live market prices.

## Mock mode

`mock_mode: true` (default in example config + browser Vite session):

- No binary required
- Synthetic hashrate / temp / power / shares
- Full UI walkthrough on any OS via `npm run dev`

## Phase roadmap

| Phase | Scope |
|-------|--------|
| **1 (this MVP)** | XMR/XMRig CPU live path + GPU adapter/UI scaffold + mock |
| **2** | Harden GPU plugin of choice; richer Windows sensors (NVAPI/LHM) |
| **3** | More adapters via plugin registry keyed by `config.*.adapter` |

## Non-goals

- Implementing RandomX / KawPow / any PoW in-process
- Storing seed phrases or private keys
- Shipping miner executables
