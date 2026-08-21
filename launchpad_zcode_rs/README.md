# Zcode Novation Launchpad Mini MK3 Integration (Rust Edition) 🦀🎛️🚀

High performance, low-latency **Rust background daemon**, **Windows 11 System Tray App**, and CLI notifier connecting the **Novation Launchpad Mini MK3** controller with the **Zcode AI Coding Environment**.

Features **13 visual animations**, **4 transition effects**, and **Windows 11 System Tray executable integration**!

---

## 🌟 Features

- 🟢 **8x8 Main Task Grid Visualizer**:
  - ⚪ **Grey (Dim)**: Pending task
  - 🟡 **Yellow (Pulsing)**: Active / Thinking task
  - 🟢 **Green (Bright)**: Successfully completed task
  - 🔴 **Red (Bright)**: Failed / Error task
- ⚡ **13 Visual Animations**:
  - `spinner`: Rotating cyan spinner for AI thinking / code generation
  - `scan`: White scanning line for running commands / tests
  - `success_ripple`: Green expanding ripple on operation completion
  - `error_flash`: Red flashing grid on error
  - `rainbow_wave`: Diagonal shifting rainbow wave across the matrix
  - `matrix_rain`: Green digital rain effect with leading bright drops
  - `fireworks`: Random colorful bursting fireworks
  - `galaxy_spiral`: Rotating galaxy spiral pattern
  - `plasma_wave`: Dynamic sine plasma wave simulation
  - `equalizer_bars`: 8-channel audio equalizer bars
  - `strobe_pulse`: Alternating strobe pulse effect
- ✨ **4 Transition Effects**:
  - `dissolve`: Random pad dissolve transition
  - `wipe_right`: Horizontal curtain wipe transition
  - `wipe_down`: Vertical curtain wipe transition
  - `zoom_iris`: Circular expanding/contracting iris transition
- 🖥️ **Windows 11 System Tray App**:
  - Background tray icon (`zcode-launchpad-tray.exe`) for Windows 11 Home/Pro.
- 🦀 **Rust Core Engine**: Built with `midir` and `tokio` for sub-millisecond response time.

---

## 📦 Building & Executable Creation for Windows 11

### 1. Build Executables via PowerShell
In PowerShell, run:
```powershell
cd launchpad_zcode_rs
.\build_windows_exe.ps1
```

This will produce standalone `.exe` files in `launchpad_zcode_rs/dist/ZcodeLaunchpad-Windows11/`:
1. `zcode-launchpad-tray.exe` (Windows 11 System Tray Application with icon)
2. `zcode-launchpad-daemon.exe` (Background UDP daemon)
3. `zcode-launchpad-notify.exe` (CLI hook notification tool)

---

## 🚀 Usage

### 1. Run System Tray App on Windows 11
Double-click `zcode-launchpad-tray.exe`. It will sit in your Windows 11 Taskbar notification tray.

### 2. Send Zcode Event Notifications
From Zcode hooks / terminal:
```bash
# Galaxy spiral animation with iris transition on task start
zcode-launchpad-notify task_start --task-id 0 --anim galaxy_spiral --trans zoom_iris

# Plasma wave animation with curtain wipe transition
zcode-launchpad-notify command --anim plasma_wave --trans wipe_right

# Fireworks transition on success
zcode-launchpad-notify task_success --task-id 0 --anim fireworks --trans dissolve
```

---

## 🧪 Testing

```bash
cd launchpad_zcode_rs
cargo test
```
