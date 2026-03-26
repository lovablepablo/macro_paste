# macro_paste

> **[Deutsch](README.de.md)**

Cross-platform system tray app that sends clipboard text as individual keystrokes. Works on **Windows** and **macOS**.

## Why?

In remote desktop sessions (TeamViewer, pcvisit, AnyDesk, etc.) you cannot use `Ctrl+V` in the **Windows login dialog** of the remote machine. macro_paste solves this: it reads text from the local clipboard and simulates individual key presses. The remote desktop software then forwards these as real keystrokes to the target machine.

## Features

- **System Tray** – runs quietly in the background (Windows tray / macOS menu bar)
- **Global Hotkey** – default: `Ctrl+Shift+V`, configurable via tray menu
- **Unicode Support** – special characters (umlauts, @, €, etc.) are sent correctly
- **Configurable Delay** – 10 / 20 / 30 / 50 / 100 / 200ms between keystrokes (default: 30ms)
- **Autostart** – optionally launch at system startup
- **Single Instance** – prevents duplicate tray icons
- **Portable** – single binary, no installation required
- **Config File** – `config.json` next to the binary, created automatically

## Installation

### Windows

**Option A: Pre-built .exe (recommended)**

1. Download `macro_paste.exe` from the [latest release](https://github.com/lovablepablo/macro_paste/releases)
2. Place it in any folder (e.g. `C:\Tools\`)
3. Run it – the tray icon appears in the system tray

**Option B: Build from source**

Prerequisites: [Rust](https://rustup.rs/), [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload

```bash
git clone https://github.com/lovablepablo/macro_paste.git
cd macro_paste
cargo build --release
```

Binary: `target/release/macro_paste.exe`

### macOS

Build from source (pre-built binary not yet available):

Prerequisites: [Rust](https://rustup.rs/), Xcode Command Line Tools (`xcode-select --install`)

```bash
git clone https://github.com/lovablepablo/macro_paste.git
cd macro_paste
cargo build --release
```

Binary: `target/release/macro_paste`

**Important:** On first launch, grant Accessibility permission under **System Settings > Privacy & Security > Accessibility**. Without this, keystroke simulation will not work.

## Usage

1. **Copy text** – e.g. copy a password to the clipboard with `Ctrl+C` / `Cmd+C`
2. **Focus the target field** – e.g. click the password field in the remote session
3. **Press the hotkey** – `Ctrl+Shift+V` (default) – the text is typed character by character

### Tray Menu (right-click / click the icon)

| Entry | Function |
|-------|----------|
| Paste as Keystrokes | Manual trigger (alternative to the hotkey) |
| Hotkey | Change the key combination (Ctrl+Shift+V/P, Ctrl+Alt+V/P) |
| Delay | Adjust the delay between keystrokes |
| Autostart | Auto-launch the app at system startup |
| Beenden | Quit the app |

## Configuration

Settings are stored in `config.json` next to the binary:

```json
{
  "hotkey": "Ctrl+Shift+V",
  "delay_ms": 30,
  "autostart": false
}
```

The file is created automatically with default values on first launch. Changes made via the tray menu are saved immediately.

## Update

1. Quit the app via the tray menu
2. Replace the binary with the new version
3. Restart the app – `config.json` is preserved

## Privacy & Security

- The app **does not store any passwords or clipboard content** – it only reads the clipboard at the moment the hotkey is pressed
- **No network access** – the app works entirely offline, no data is sent anywhere
- **No telemetry or analytics** – what you paste stays on your machine
- The only file written to disk is `config.json` (hotkey, delay, autostart preference)

## Technical Details

- **Language:** Rust
- **Keystroke Simulation:**
  - Windows: `SendInput` with `KEYEVENTF_UNICODE`
  - macOS: `CGEvent` with `CGEventKeyboardSetUnicodeString`
- **Event Loop:** winit (cross-platform)
- **Tray:** tray-icon + muda
- **Hotkey:** global-hotkey crate
- **Clipboard:** arboard (cross-platform)
- **Autostart:**
  - Windows: Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
  - macOS: Launch Agent (`~/Library/LaunchAgents/`)
- **Single Instance:**
  - Windows: Named Mutex
  - macOS: File lock (`~/.macropaste/instance.lock`)

## License

MIT
