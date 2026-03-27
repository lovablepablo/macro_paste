# macro_paste

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
- **Config File** – settings stored automatically on first launch

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

**Option A: Pre-built .app bundle (recommended)**

1. Download `macro_paste-macos-arm64.zip` from the [latest release](https://github.com/lovablepablo/macro_paste/releases)
2. Unzip and move `macro_paste.app` to your Applications folder or any other location
3. Remove the macOS quarantine flag (required for unsigned apps):
   ```bash
   xattr -cr /Applications/macro_paste.app
   ```
4. Open it – the icon appears in the menu bar

> **Note:** macOS may show _"damaged and can't be opened"_ for unsigned apps downloaded from the internet. The `xattr -cr` command above removes the quarantine flag and fixes this.

**Option B: Build from source**

Prerequisites: [Rust](https://rustup.rs/), Xcode Command Line Tools

```bash
xcode-select --install   # if not already present
```

```bash
git clone https://github.com/lovablepablo/macro_paste.git
cd macro_paste
cargo build --release
```

To run it as a proper menu bar app, wrap it in an `.app` bundle:

```bash
mkdir -p target/release/macro_paste.app/Contents/MacOS
mkdir -p target/release/macro_paste.app/Contents/Resources
cp target/release/macro_paste target/release/macro_paste.app/Contents/MacOS/macro_paste
cp assets/Info.plist target/release/macro_paste.app/Contents/Info.plist
open target/release/macro_paste.app
```

**Important – Accessibility permission:**

Keystroke simulation requires Accessibility access. Grant it under:
**System Settings → Privacy & Security → Accessibility**

> **Note:** macOS revokes this permission whenever the binary is replaced (e.g. after a new build). After every update you must re-add the app in the Accessibility settings: remove it with `−` and add it again with `+`.

**Troubleshooting – menu bar icon not visible:**

If the icon does not appear, macOS may be hiding it due to limited menu bar space. Hold `Cmd` and drag other icons to the right to make room, or check whether a menu bar manager (Bartender, Ice, etc.) has hidden it.

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
| Quit | Quit the app |

## Configuration

Settings are stored in a platform-specific config file:

- **macOS:** `~/Library/Application Support/macro_paste/config.json`
- **Windows:** `config.json` next to the executable

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
2. Replace the binary / app bundle with the new version
3. Restart the app – your config is preserved

**macOS only:** After replacing the binary, re-grant Accessibility permission under **System Settings → Privacy & Security → Accessibility** (remove and re-add the app).

## Privacy & Security

- The app **does not store any passwords or clipboard content** – it only reads the clipboard at the moment the hotkey is pressed
- **No network access** – the app works entirely offline, no data is sent anywhere
- **No telemetry or analytics** – what you paste stays on your machine

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
