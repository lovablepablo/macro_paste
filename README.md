# macro_paste

> **[Deutsch](README.de.md)**

Windows system tray app that sends clipboard text as individual keystrokes.

## Why?

In remote desktop sessions (TeamViewer, pcvisit, AnyDesk, etc.) you cannot use `Ctrl+V` in the **Windows login dialog** of the remote machine. macro_paste solves this: it reads text from the local clipboard and simulates individual key presses via `SendInput`. The remote desktop software then forwards these as real keystrokes to the target machine.

## Features

- **System Tray** – runs quietly in the background
- **Global Hotkey** – default: `Ctrl+Shift+V`, configurable via tray menu
- **Unicode Support** – special characters (umlauts, @, €, etc.) are sent correctly
- **Configurable Delay** – 10 / 20 / 30 / 50 / 100 / 200ms between keystrokes (default: 30ms)
- **Autostart** – optionally launch at Windows startup (registry-based)
- **Portable** – single .exe (~620 KB), no installation required
- **Config File** – `config.json` next to the .exe, created automatically

## Installation

### Option A: Pre-built .exe (recommended)

1. Download `macro_paste.exe` from the [latest release](https://github.com/lovablepablo/macro_paste/releases)
2. Place it in any folder (e.g. `C:\Tools\`)
3. Run it – the tray icon appears in the system tray

### Option B: Build from source

**Prerequisites:**
- [Rust](https://rustup.rs/) (including Cargo)
- [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload

```bash
git clone https://github.com/lovablepablo/macro_paste.git
cd macro_paste
cargo build --release
```

The compiled binary is at `target/release/macro_paste.exe`.

## Usage

1. **Copy text** – e.g. copy a password to the clipboard with `Ctrl+C`
2. **Focus the target field** – e.g. click the password field in the Windows login of the remote session
3. **Press the hotkey** – `Ctrl+Shift+V` (default) – the text is typed character by character

### Tray Menu (right-click the icon)

| Entry | Function |
|-------|----------|
| Paste as Keystrokes | Manual trigger (alternative to the hotkey) |
| Hotkey | Change the key combination (Ctrl+Shift+V/P, Ctrl+Alt+V/P) |
| Delay | Adjust the delay between keystrokes |
| Autostart | Auto-launch the app at Windows startup |
| Beenden | Quit the app |

## Configuration

Settings are stored in `config.json` next to the .exe:

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
2. Download the new `macro_paste.exe` and replace the old one
3. Restart the app – `config.json` is preserved

## Technical Details

- **Language:** Rust
- **Keystroke Method:** `SendInput` with `KEYEVENTF_UNICODE` – sends Unicode characters directly without VirtualKey mapping
- **Event Loop:** winit (Windows Message Pump)
- **Tray:** tray-icon + muda
- **Hotkey:** global-hotkey crate
- **Clipboard:** clipboard-win (Windows API)
- **Autostart:** Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)

## License

MIT
