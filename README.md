# Zcode Novation Launchpad Mini MK3 Integration 🎛️🚀

A standalone background daemon and event notification CLI tool connecting the **Novation Launchpad Mini MK3** controller with the **Zcode AI Coding Environment**.

It provides real-time LED visual feedback for AI coding tasks and operation animations (thinking spinner, terminal scan, success ripple, and error flash).

---

## 🌟 Features

- 🟢 **8x8 Main Task Grid Visualizer**:
  - ⚪ **Grey (Dim)**: Pending task
  - 🟡 **Yellow (Pulsing)**: Active / Thinking task
  - 🟢 **Green (Bright)**: Successfully completed task
  - 🔴 **Red (Bright)**: Failed / Error task
- ⚡ **Operation Animations**:
  - `spinner`: Rotating cyan spinner for thinking / code generation
  - `scan`: White scanning line for running commands / tests
  - `success_ripple`: Green expanding ripple on operation completion
  - `error_flash`: Red flashing grid on error
- 🔌 **Zero-Latency Daemon**: Runs in background, listens on UDP port `9876`.
- 🛠️ **CLI Hook Notifier**: Fast lightweight CLI (`zcode-launchpad-notify`) for Zcode rule/tool/hook integration.

---

## 📦 Installation & Setup

### 1. Install Dependencies
```bash
pip install -e .
```

or install directly:
```bash
pip install mido python-rtmidi
```

### 2. Connect your Novation Launchpad Mini MK3
Plug your Launchpad Mini MK3 via USB.

### 3. Run the Daemon
Start the Launchpad daemon in background or in a terminal window:
```bash
zcode-launchpad-daemon
```

For testing without physical hardware connected:
```bash
zcode-launchpad-daemon --virtual
```

---

## 🔗 Zcode Hook Integration Guide

In your Zcode project configuration or `.zcode/rules/launchpad-hooks.md` / script hooks, execute `zcode-launchpad-notify`:

### 1. Task Start / AI Thinking
```bash
zcode-launchpad-notify task_start --task-id 0 --anim spinner
```

### 2. Command / Test Execution
```bash
zcode-launchpad-notify command --anim scan
```

### 3. Task Success
```bash
zcode-launchpad-notify task_success --task-id 0 --anim success_ripple
```

### 4. Task Error
```bash
zcode-launchpad-notify task_error --task-id 0 --anim error_flash
```

### 5. Reset Grid
```bash
zcode-launchpad-notify reset
```

---

## 🧪 Running Tests

```bash
python3 -m pytest
```
