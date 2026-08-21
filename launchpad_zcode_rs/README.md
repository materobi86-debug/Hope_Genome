# HOPE CODE Multi-App Novation Launchpad Mini MK3 Integration (Rust Edition) 🦀🎛️🗣️📱🚀

High performance, low-latency **Rust background daemon**, **Mobile PWA Control Panel GUI with Web Speech Hungarian Voice Recognition & Edge-TTS Noémi Voice Reader**, **Top Launchpad Button Hardware View Switcher**, **Multi-App Switcher (HOPE CODE, Claude Code, OpenAI Codex)**, **Live CPU/RAM Meter & Pomodoro Timer**, **Scrolling Text Banner ("HOPE CODE" / "STOP!!")**, **Windows 11 System Tray App**, and CLI notifier connecting the **Novation Launchpad Mini MK3** controller with **HOPE CODE, Claude Code, and OpenAI Codex AI Coding Environments**.

Features **Mobile PWA Installation on Android/iOS/Windows**, **Web Speech API Hungarian Voice Commands**, **Top Launchpad Control Buttons for Direct Hardware Switching**, **Multi-App view switching with side LED indicators**, **Live CPU & RAM Performance Meter**, **Focus Pomodoro Clock Ring**, **Scrolling Text Banner on 8x8 LED matrix**, **Edge-TTS Noémi voice speech reading synchronized with real-time 8-bar audio spectrum visualizer**, **20 visual animations**, **4 transition effects**, **Interactive Virtual MIDI Grid**, **Windows Startup Config**, and an **AI Prompt Guide for generating custom animations**!

---

## 🌟 Features

- 🎛️ **Top Control Buttons Hardware View Switcher (CC 91..96)**:
  - **Button 1 (CC 91)**: HOPE CODE / Zcode Matrix View
  - **Button 2 (CC 92)**: Claude Code Matrix View
  - **Button 3 (CC 93)**: OpenAI Codex Matrix View
  - **Button 4 (CC 94)**: ÉLŐ CPU & RAM Monitor
  - **Button 5 (CC 95)**: Pomodoro Fókusz Időzítő
  - **Button 6 (CC 96)**: Hangvezérlés Mód
- 📱 **Mobile PWA & Web Speech Hungarian Voice Commands** (`http://127.0.0.1:8080`):
  - Telepíthető mobil PWA alkalmazás Android/iOS/Windows11 eszközökre!
  - **Magyar Hangvezérlés**: Beszélj magyarul az alkalmazások vezérléséhez (pl. *"Futtass tesztet"*, *"Válts Claude-ra"*, *"Állítsd le"*, *"Olvasd fel a választ"*).
- 🔀 **Multi-App View Selector & Status Switcher** (HOPE CODE, Claude Code, OpenAI Codex):
  - Az 8x8 mátrix nézet automatikusan átvált az éppen kiválasztott AI alkalmazás (HOPE CODE / Zcode, Claude Code, OpenAI Codex) taskjaira.
  - Az oldalsó kör LED gombok (Note 89, 79, 69) mutatják az alkalmazások állapotát: Zöld = Aktív nézet, Villogó Sárga = Esemény a háttérben futó appban, Cián = Háttérben futó app.
- 📊 **Jules Extrák - ÉLŐ Rendszer-Monitorok & Fókusz Események**:
  - `cpu_ram_meter`: ÉLŐ CPU terheltség (fentről lefelé zöld-sárga-piros sávok) és RAM használat (alsó sávok) kijelzése az 8x8 mátrixon!
  - `pomodoro_timer`: Fókusz időzítő köríves számláló az 8x8 mátrix külső gombjain.
  - `git_sentinel`: Cryptographic audit proof és Git commit megerősítő zöld lüktető hullám animáció.
- 🚨 **Scrolling Text Banner Matrix** (`text_banner`):
  - Teljes kijelzős futó felirat piros/színes LED fényekkel az 8x8 mátrixon! Alapértelmezett: **"HOPE CODE"** és **"STOP!!"**.
- 🗣️ **Edge-TTS Noémi Voice Speech Reader & 8-Bar Audio Equalizer Sync** (`zcode-launchpad-speak`):
  - Felolvassa a válaszokat mondatonként a magyar `hu-HU-NoemiNeural` Edge-TTS hangon.
  - A felolvasás alatt élőben 8-bar audio spektrum equalizer jelenik meg a Launchpad Mini MK3 kijelzőjén és a virtuális MIDI felületen!
- 🖥️ **Windows 11 System Tray App**:
  - Background tray icon (`zcode-launchpad-tray.exe`) for Windows 11 Home/Pro.
- 🦀 **Rust Core Engine**: Built with `midir`, `warp`, and `tokio` for sub-millisecond response time.

---

## 📦 Building & Executable Creation for Windows 11

### 1. Build Executables via PowerShell
In PowerShell, run:
```powershell
cd launchpad_zcode_rs
.\build_windows_exe.ps1
```

This will produce standalone `.exe` files in `launchpad_zcode_rs/dist/ZcodeLaunchpad-Windows11/`:
1. `zcode-launchpad-panel.exe` (Control Panel, Multi-App Dashboard, PWA, Virtual MIDI & Noémi TTS GUI)
2. `zcode-launchpad-tray.exe` (Windows 11 System Tray Application with icon)
3. `zcode-launchpad-daemon.exe` (Background UDP daemon)
4. `zcode-launchpad-notify.exe` (CLI hook notification tool)

---

## 🚀 Usage

### 1. Multi-App Switching via CLI or Top Buttons
```bash
# Switch active Launchpad view to Claude Code
zcode-launchpad-notify switch_app --app claude_code

# Send task event for OpenAI Codex in background
zcode-launchpad-notify task_start --app codex --task-id 3 --anim matrix_rain
```

### 2. Live CPU/RAM Meter & Scrolling Text
```bash
# Display Live CPU & RAM Performance Meter on Launchpad
zcode-launchpad-notify command --anim cpu_ram_meter

# Display Scrolling Text Banner
zcode-launchpad-notify command --anim text_banner --text "HOPE CODE"
```

### 3. Speak Text with Noémi Voice & Real-Time Launchpad EQ Sync
```bash
zcode-launchpad-speak "Szia! Ez a HOPE CODE AI válasza, amit most olvasok fel Noémi hangján."
```

### 4. Launch Mobile PWA & Voice Control
Run `zcode-launchpad-panel.exe` and open `http://127.0.0.1:8080` in your browser or install as PWA on mobile.

---

## 🧪 Testing

```bash
cd launchpad_zcode_rs
cargo test
python3 -m pytest
```
