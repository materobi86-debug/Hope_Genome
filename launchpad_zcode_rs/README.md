# HOPE CODE Multi-App Launchpad Integration & Full Microscope Memory 🦀🎛️🗣️🎙️🤖📱⚡🚀

High performance, low-latency **Rust background daemon**, **Mobile PWA Control Panel GUI with Live Voice-to-Voice Call, Jules Agent Chat & Full Cloned Microscope Memory (`bincode` binary format)**, **Web Speech Hungarian Voice Recognition & Edge-TTS Noémi Voice Reader**, **Top Launchpad Button Hardware View Switcher**, **Multi-App Switcher (HOPE CODE, Claude Code, OpenAI Codex)**, **Live CPU/RAM Meter & Pomodoro Timer**, **Scrolling Text Banner ("HOPE CODE" / "STOP!!")**, **Windows 11 System Tray App**, and CLI notifier connecting the **Novation Launchpad Mini MK3** controller with **HOPE CODE, Claude Code, and OpenAI Codex AI Coding Environments**.

Features **Cloned Full Microscope Memory Repository Integration (`bincode` binary format)**, **iPhone Web Push Notifications**, **Camera Flash/Torch Light Signal Control**, **Haptic Vibration Feedback (`navigator.vibrate`)**, **Autonomous Background Jules Agent Execution**, **Jules Agent Interactive Chat**, **Mobile PWA Installation on Android/iOS/Windows**, **Web Speech API Hungarian Voice Commands**, **Top Launchpad Control Buttons for Direct Hardware Switching**, **Multi-App view switching with side LED indicators**, **Live CPU & RAM Performance Meter**, **Focus Pomodoro Clock Ring**, **Scrolling Text Banner on 8x8 LED matrix**, **Edge-TTS Noémi voice speech reading synchronized with real-time 8-bar audio spectrum visualizer**, **20 visual animations**, **4 transition effects**, **Interactive Virtual MIDI Grid**, **Windows Startup Config**, and an **AI Prompt Guide for generating custom animations**!

---

## 🌟 Features

- 🔬 **Full Microscope Memory Integration (`microscope_memory_repo`)**:
  - Cloned full [Microscope Memory](https://github.com/silentnoisehun/microscope-memory) repository into `launchpad_zcode_rs/microscope_memory_repo`.
  - Minden beszélgetés, emlék, preferenciák és feladatelőzmények a bináris `bincode` formátumba mentődnek a `microscope_memory.bin` tárolóban.
- 🔔 **iOS Web Push Értesítések iPhone-ra**:
  - Amikor nem vagy a gépnél, Jules háttérben autonóm feladatokat végez, és az iPhone-odra Web Push értesítést küld, ha elkészült vagy írt neked!
- ⚡ **iPhone Vakufény (Flash/Torch) Signal Control**:
  - Az iPhone PWA felületről közvetlenül kapcsolható a kamera vakujának fénye jelzőfényként!
- 📳 **Haptikus Rezgés Visszajelzés (`navigator.vibrate`)**:
  - Minden gombnyomás, mátrix érintés és esemény finom haptikus rezgést ad a telefonodon!
- 🤖 **Autonóm Jules Agent & Microscope Memory (`bincode` binary persistence)**:
  - Jules önállóan is képes feladatokat végezni a háttérben.
- 🎙️ **Live Voice-to-Voice Continuous Orb Call**:
  - Folyamatos élő hanghívás Jules-szal a PWA felületen! Beszélj magyarul, Jules élőben válaszol Noémi hangján, miközben a Launchpad gombmátrixán szinkronban fut a 8-bar audio spectrum equalizer.
- 🎛️ **Top Control Buttons Hardware View Switcher (CC 91..96)**:
  - **Button 1 (CC 91)**: HOPE CODE / Zcode Matrix View
  - **Button 2 (CC 92)**: Claude Code Matrix View
  - **Button 3 (CC 93)**: OpenAI Codex Matrix View
  - **Button 4 (CC 94)**: ÉLŐ CPU & RAM Monitor
  - **Button 5 (CC 95)**: Pomodoro Fókusz Időzítő
  - **Button 6 (CC 96)**: Hangvezérlés Mód
- 📱 **Mobile PWA & Web Speech Hungarian Voice Commands** (`http://127.0.0.1:8080`):
  - Telepíthető mobil PWA alkalmazás Android/iOS/Windows11 eszközökre!
- 🔀 **Multi-App View Selector & Status Switcher** (HOPE CODE, Claude Code, OpenAI Codex):
  - Az 8x8 mátrix nézet automatikusan átvált az éppen kiválasztott AI alkalmazás (HOPE CODE / Zcode, Claude Code, OpenAI Codex) taskjaira.
- 📊 **Jules Extrák - ÉLŐ Rendszer-Monitorok & Fókusz Események**:
  - `cpu_ram_meter`: ÉLŐ CPU terheltség és RAM használat kijelzése az 8x8 mátrixon!
  - `pomodoro_timer`: Fókusz időzítő köríves számláló az 8x8 mátrix külső gombjain.
  - `git_sentinel`: Cryptographic audit proof és Git commit megerősítő zöld lüktető hullám animáció.
- 🚨 **Scrolling Text Banner Matrix** (`text_banner`):
  - Teljes kijelzős futó felirat piros/színes LED fényekkel az 8x8 mátrixon! Alapértelmezett: **"HOPE CODE"** és **"STOP!!"**.
- 🗣️ **Edge-TTS Noémi Voice Speech Reader & 8-Bar Audio Equalizer Sync** (`zcode-launchpad-speak`):
  - Felolvassa a válaszokat mondatonként a magyar `hu-HU-NoemiNeural` Edge-TTS hangon.
- 🖥️ **Windows 11 System Tray App**:
  - Background tray icon (`zcode-launchpad-tray.exe`) for Windows 11 Home/Pro.
- 🦀 **Rust Core Engine**: Built with `midir`, `microscope-memory`, `bincode`, `warp`, and `tokio` for sub-millisecond response time.

---

## 📦 Building & Executable Creation for Windows 11

### 1. Build Executables via PowerShell
In PowerShell, run:
```powershell
cd launchpad_zcode_rs
.\build_windows_exe.ps1
```

This will produce standalone `.exe` files in `launchpad_zcode_rs/dist/ZcodeLaunchpad-Windows11/`:
1. `zcode-launchpad-panel.exe` (Control Panel, Microscope Memory, PWA, Virtual MIDI & Noémi TTS GUI)
2. `zcode-launchpad-tray.exe` (Windows 11 System Tray Application with icon)
3. `zcode-launchpad-daemon.exe` (Background UDP daemon)
4. `zcode-launchpad-notify.exe` (CLI hook notification tool)

---

## 🧪 Testing

```bash
cd launchpad_zcode_rs
cargo test
python3 -m pytest
```
