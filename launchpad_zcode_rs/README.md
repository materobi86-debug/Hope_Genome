# HOPE CODE Multi-App Novation Launchpad Mini MK3 Integration (Rust Edition) 🦀🎛️🗣️🚀

High performance, low-latency **Rust background daemon**, **Multi-App Control Panel GUI with Virtual MIDI Display & Edge-TTS Noémi Voice Reader**, **Multi-App Switcher (HOPE CODE, Claude Code, OpenAI Codex)**, **Live CPU/RAM Meter & Pomodoro Timer**, **Scrolling Text Banner ("HOPE CODE" / "STOP!!")**, **Windows 11 System Tray App**, and CLI notifier connecting the **Novation Launchpad Mini MK3** controller with **HOPE CODE, Claude Code, and OpenAI Codex AI Coding Environments**.

Features **Multi-App view switching with side LED indicators**, **Live CPU & RAM Performance Meter**, **Focus Pomodoro Clock Ring**, **Scrolling Text Banner on 8x8 LED matrix**, **Edge-TTS Noémi voice speech reading synchronized with real-time 8-bar audio spectrum visualizer**, **20 visual animations**, **4 transition effects**, **Interactive Virtual MIDI Grid**, **Windows Startup Config**, and an **AI Prompt Guide for generating custom animations**!

---

## 🌟 Features

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
- 📱 **Control Panel GUI & Virtual MIDI Display** (`zcode-launchpad-panel.exe`):
  - Interactive 8x8 virtual Launchpad matrix showing live status colors in real-time.
  - Multi-App selector buttons with side LED status preview.
  - Live preview tab for testing all 20 animations and 4 transitions on screen and physical device.
  - Windows 11 startup / system autostart configuration settings.
  - Prompt guide template for creating new custom visual animations using external AI tools (Claude / ChatGPT).
- 🟢 **8x8 Main Task Grid Visualizer**:
  - ⚪ **Grey (Dim)**: Pending task
  - 🟡 **Yellow (Pulsing)**: Active / Thinking task
  - 🟢 **Green (Bright)**: Successfully completed task
  - 🔴 **Red (Bright)**: Failed / Error task
- ⚡ **20 Visual Animations**:
  - `cpu_ram_meter`: Live CPU & RAM Performance Meter
  - `pomodoro_timer`: Focus clock ring timer
  - `git_sentinel`: Cryptographic AI Proof & Git Sentinel
  - `text_banner`: Scrolling text banner (HOPE CODE / STOP!! / Custom text)
  - `vortex_whirl`: Rotating whirlpool vortex
  - `color_comb`: Shifting color comb matrix
  - `hypnotic_rings`: Concentric expanding hypnotic rings
  - `pulsar_burst`: Pulsing central cross burst
  - `equalizer_bars`: 8-channel audio spectrum equalizer synced with TTS speech!
  - `spinner`: Rotating cyan spinner for AI thinking / code generation
  - `scan`: White scanning line for running commands / tests
  - `success_ripple`: Green expanding ripple on operation completion
  - `error_flash`: Red flashing grid on error
  - `rainbow_wave`: Diagonal shifting rainbow wave across the matrix
  - `matrix_rain`: Green digital rain effect with leading bright drops
  - `fireworks`: Random colorful bursting fireworks
  - `galaxy_spiral`: Rotating galaxy spiral pattern
  - `plasma_wave`: Dynamic sine plasma wave simulation
  - `strobe_pulse`: Alternating strobe pulse effect
- ✨ **4 Transition Effects**:
  - `dissolve`: Random pad dissolve transition
  - `wipe_right`: Horizontal curtain wipe transition
  - `wipe_down`: Vertical curtain wipe transition
  - `zoom_iris`: Circular expanding/contracting iris transition
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
1. `zcode-launchpad-panel.exe` (Control Panel, Multi-App Dashboard, Virtual MIDI & Noémi TTS GUI)
2. `zcode-launchpad-tray.exe` (Windows 11 System Tray Application with icon)
3. `zcode-launchpad-daemon.exe` (Background UDP daemon)
4. `zcode-launchpad-notify.exe` (CLI hook notification tool)

---

## 🚀 Usage

### 1. Multi-App Switching via CLI
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

### 4. Launch Control Panel & Virtual MIDI Display
Run `zcode-launchpad-panel.exe` and open `http://127.0.0.1:8080` in your web browser.

---

## 🧪 Testing

```bash
cd launchpad_zcode_rs
cargo test
python3 -m pytest
```
