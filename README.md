# IdleForge

**DE** · Premium Windows-Desktop-App zur Steuerung *externer* CPU- und GPU-Kryptominer.  
**EN** · Premium Windows desktop hobby app that manages *external* CPU + GPU cryptocurrency miners.

Kein Mining-Algorithmus im Repo. Keine Miner-Binaries. Keine Seed-Phrasen / Private Keys — nur Empfangsadressen.

> Designziel: Linear / Raycast / Stripe-Dashboard-Niveau — ruhig, dunkel, präzise. Kein RGB-Gamer-Chrome.

---

## Quick start (Mock-UI, jedes OS)

Die UI läuft im **Mock-Modus** ohne Tauri und ohne echte Miner — ideal zum Anschauen und Entwickeln.

```bash
npm install
npm run dev
```

Öffne die angezeigte URL (Standard: `http://localhost:1420`).  
Start/Stop CPU & GPU, Profile, Adaptive-Banner und Wallets funktionieren simuliert.

```bash
npm run build   # Production-Frontend nach dist/
```

---

## Windows · echte Miner (Tauri 2)

Voraussetzungen: [Tauri 2 auf Windows](https://tauri.app/start/prerequisites/) (WebView2, Rust, Node).

1. XMRig und lolMiner **selbst** herunterladen (nicht im Repo).
2. `config/example.config.json` nach `%APPDATA%\IdleForge\config.json` kopieren (wird beim ersten Start auch angelegt).
3. Pfade, Pools und **Receive-Adressen** setzen. `mock_mode: false`.
4. Starten:

```bash
npm install
npm run tauri:dev
# Release-Installer (auf Windows ausführen):
npm run tauri:build
```

Der NSIS-Installer liegt danach unter `src-tauri/target/release/bundle/nsis/` (`IdleForge_*_x64-setup.exe`). Doppelklick installiert für den aktuellen Benutzer (kein Admin nötig) und legt einen Startmenü-Eintrag an. Parallel entsteht ein MSI unter `bundle/msi/`. In der installierten App kommen CPU-/GPU-Namen und die Systemlast von echten Sensoren (`sysinfo`, `nvidia-smi` oder `Win32_VideoController`) — der Mock betrifft nur Miner-Hashrates, solange `mock_mode` an ist.

CPU und GPU lassen sich **unabhängig** starten/stoppen.

| Kanal | Coin | Algorithmus | Adapter | Binary (Beispiel) |
|-------|------|-------------|---------|-------------------|
| CPU | Monero (XMR) | RandomX | `xmrig` | `C:\Miners\xmrig\xmrig.exe` |
| GPU | Ravencoin (RVN) | **KawPow** | `lolminer` | `C:\Miners\lolMiner\lolMiner.exe` |

### Warum KawPow?

Für moderne NVIDIA-GPUs ist **KawPow (Ravencoin)** ein sinnvolles, weiterhin aktives Ziel mit stabiler lolMiner-Unterstützung. Ethash/ETC ist möglich, aber KawPow ist als Default klar und erweiterbar. Weitere Algos kommen über das `MinerAdapter`-Plugin-Interface.

---

## Sicherheitslimit — maximal 95%

IdleForge fordert **niemals 100%** der Maschine an. Extrem ist das Maximum und liegt bei **95%** Intensität, Threads und GPU-Power. Adaptive Regeln und manuelle Config-Werte werden zur Laufzeit auf dieses Limit geklemmt (auch wenn eine Datei `1.0` / `100` enthält). Mindestens ein logischer CPU-Kern bleibt frei; XMRig startet mit `--cpu-priority 1` (unter Normal), damit Windows bedienbar bleibt.

## Profile

| ID | DE | Bedeutung |
|----|----|-----------|
| `idle` | Idle | Minimal — Hintergrund schonen |
| `low` | Niedrig | Schonend |
| `medium` | Mittel | Alltag (Default) |
| `high` | Hoch | Aggressiver |
| `extreme` | Extrem | Hartes Maximum: **95%** CPU und GPU |

Adaptive Regeln können unter das gewählte Profil **drosseln** oder pausieren. Sie können das 95%-Limit nicht überschreiben.

---

## Adaptive Regeln (Scaffold)

Teilweise umgesetzt in Mock + Rust-Backend:

- **Aktive Nutzung / hohe Last** → Intensität × `reduce_factor_on_active` (effektiv oft „low“)
- **Leerlauf** → nach `idle_seconds_before_ramp` Ramp auf Zielprofil
- **Temp-Limits** (`cpu_temp_limit_c` / `gpu_temp_limit_c`) → Drosselung
- **Leistungslimit** (`max_package_watts`) → Drosselung, wenn Sensoren Werte liefern
- **Akku** (`pause_on_battery`) → Pause (Windows best-effort)

UI meldet Maus/Tastatur-Aktivität an das Backend (`report_user_activity`).

---

## Multi-Wallet

In der Config / UI:

- Beliebig viele Einträge mit `label`, `coin`, `receive_address`, `pool_worker`
- CPU/GPU-Kanal jeweils eine aktive Wallet-ID
- **Niemals** Seed-Phrasen oder Private Keys speichern

---

## Kurse / Ertrag / Stromkosten

`rates.xmr_eur`, `rates.rvn_eur`, `rates.electricity_eur_per_kwh` sind optional.  
`null` → klare Platzhalter in der UI. IdleForge **erfindet keine Live-Preise**.

Bei gesetztem Strompreis: geschätzte €/Tag aus gemessener/geschätzter Leistung.

Siehe auch [ARCHITECTURE.md](./ARCHITECTURE.md).

---

## Architektur

```
┌─────────────────────────────────────────────┐
│  React / TypeScript UI (Deutsch)            │
│  Dashboard · Profile · Wallets · Settings   │
└──────────────────┬──────────────────────────┘
                   │ Tauri invoke  /  MockEngine
┌──────────────────▼──────────────────────────┐
│  AppState (Rust)                            │
│  · AdaptiveController                       │
│  · SensorHub (sysinfo + nvidia-smi best-effort) │
│  · Config (%APPDATA%/IdleForge)             │
│  · MinerAdapter[]                           │
│       ├─ XmrigAdapter  (CPU / RandomX)      │
│       └─ LolMinerAdapter (GPU / KawPow)     │
└─────────────────────────────────────────────┘
         │ spawn + HTTP API / stdout
         ▼
   Externe Miner-Prozesse (vom Nutzer gestellt)
```

### Neues Miner-Plugin hinzufügen

1. Trait `MinerAdapter` in `src-tauri/src/miners/adapter.rs` implementieren (`start` / `stop` / `poll_stats`).
2. Modul unter `src-tauri/src/miners/` anlegen (siehe `xmrig.rs` / `lolminer.rs`).
3. In `state.rs` verdrahten oder später per Config `adapter: "dein-name"` auswählen.
4. Frontend-Kanal-Config (`cpu` / `gpu` / künftig weitere) erweitern.
5. **Keine** Algorithmen-Implementierung — nur Prozess-Steuerung und Parsing.

---

## Warum Tauri 2 (nicht Electron)?

- Kleineres Bundle, weniger RAM, native Windows-Feel
- Rust eignet sich natürlich für Prozess-Supervision und Sensorik
- Web-UI bleibt mit React premium und schnell iterierbar

Electron wäre nur Fallback, wenn Tauri-Voraussetzungen blockieren — aktuell nicht nötig.

---

## Sensoren & Degradation

| Quelle | Nutzung |
|--------|---------|
| `sysinfo` | CPU-Name, Kerne, RAM, System-Last |
| `nvidia-smi` (optional) | GPU-Name, Temp, Power |
| Windows Battery (PowerShell) | AC vs. Akku |

Fehlende Tools → Felder `n/v`, App bleibt nutzbar. Mock-Modus simuliert vollständige Metriken.

---

## Sicherheit / Scope

- Windows-Desktop-Hobby-Tool, device-agnostisch (Referenzhardware nur Beispiel)
- Keine committed Binaries, Secrets oder Wallets
- Nur öffentliche Empfangsadressen und Pool-Worker

---

## Scripts

| Script | Zweck |
|--------|-------|
| `npm run dev` | Vite Mock-UI |
| `npm run build` | Frontend-Build |
| `npm run tauri:dev` | Tauri Dev (Windows empfohlen) |
| `npm run tauri:build` | Windows Installer (MSI/NSIS) |

---

## Lizenz / Hinweis

Hobby-Projekt. Mining kann Garantien, Thermik und Stromkosten betreffen — Profile und Limits vernünftig wählen. Halte dich an Pool-/Coin-Nutzungsbedingungen und lokale Gesetze.
