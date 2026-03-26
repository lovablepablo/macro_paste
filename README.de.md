# macro_paste

> **[English](README.md)**

Plattformübergreifende System-Tray-App, die Clipboard-Text als einzelne Tastaturanschläge sendet. Funktioniert auf **Windows** und **macOS**.

## Wozu?

In Fernwartungssitzungen (TeamViewer, pcvisit, AnyDesk etc.) kann man im **Windows Login-Dialog** der Zielmaschine kein `Ctrl+V` verwenden. macro_paste löst das Problem: Es liest den Text aus der lokalen Zwischenablage und simuliert einzelne Tastatureingaben. Die Fernwartungssoftware leitet diese dann wie echte Tastaturanschläge an die Zielmaschine weiter.

## Features

- **System Tray** – läuft unauffällig im Hintergrund (Windows Tray / macOS Menüleiste)
- **Globaler Hotkey** – Standard: `Ctrl+Shift+V`, änderbar über Tray-Menü
- **Unicode-Support** – Sonderzeichen (äöü, @, €, ß) werden korrekt gesendet
- **Konfigurierbarer Delay** – 10 / 20 / 30 / 50 / 100 / 200ms zwischen Anschlägen (Standard: 30ms)
- **Autostart** – optional beim Systemstart mitlaufen
- **Single Instance** – verhindert doppelte Tray-Icons
- **Portable** – einzelne Binary, keine Installation nötig
- **Config-Datei** – `config.json` neben der Binary, wird automatisch erstellt

## Installation

### Windows

**Option A: Vorkompilierte .exe (empfohlen)**

1. `macro_paste.exe` aus dem [neuesten Release](https://github.com/lovablepablo/macro_paste/releases) herunterladen
2. In einen beliebigen Ordner legen (z.B. `C:\Tools\`)
3. Starten – das Tray-Icon erscheint im System Tray

**Option B: Selbst kompilieren**

Voraussetzungen: [Rust](https://rustup.rs/), [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) mit "Desktop development with C++" Workload

```bash
git clone https://github.com/lovablepablo/macro_paste.git
cd macro_paste
cargo build --release
```

Binary: `target/release/macro_paste.exe`

### macOS

Selbst kompilieren (vorkompilierte Binary noch nicht verfügbar):

Voraussetzungen: [Rust](https://rustup.rs/), Xcode Command Line Tools (`xcode-select --install`)

```bash
git clone https://github.com/lovablepablo/macro_paste.git
cd macro_paste
cargo build --release
```

Binary: `target/release/macro_paste`

**Wichtig:** Beim ersten Start die Bedienungshilfen-Berechtigung erteilen unter **Systemeinstellungen > Datenschutz & Sicherheit > Bedienungshilfen**. Ohne diese Berechtigung funktioniert die Tastatureingabe-Simulation nicht.

## Verwendung

1. **Text kopieren** – z.B. ein Passwort mit `Ctrl+C` / `Cmd+C` in die Zwischenablage kopieren
2. **Zielfeld fokussieren** – z.B. das Passwort-Feld in der Fernwartungssitzung anklicken
3. **Hotkey drücken** – `Ctrl+Shift+V` (Standard) – der Text wird Zeichen für Zeichen eingegeben

### Tray-Menü (Rechtsklick / Klick auf das Icon)

| Eintrag | Funktion |
|---------|----------|
| Paste as Keystrokes | Manueller Trigger (alternativ zum Hotkey) |
| Hotkey | Tastenkombination ändern (Ctrl+Shift+V/P, Ctrl+Alt+V/P) |
| Delay | Verzögerung zwischen Tastenanschlägen anpassen |
| Autostart | App beim Systemstart automatisch starten |
| Beenden | App schließen |

## Konfiguration

Die Einstellungen werden in `config.json` neben der Binary gespeichert:

```json
{
  "hotkey": "Ctrl+Shift+V",
  "delay_ms": 30,
  "autostart": false
}
```

Die Datei wird beim ersten Start automatisch mit Standardwerten erstellt. Änderungen über das Tray-Menü werden sofort gespeichert.

## Update

1. App über Tray-Menü beenden
2. Binary durch die neue Version ersetzen
3. App neu starten – die `config.json` bleibt erhalten

## Datenschutz & Sicherheit

- Die App **speichert keine Passwörter oder Clipboard-Inhalte** – sie liest die Zwischenablage nur im Moment des Hotkey-Drucks
- **Kein Netzwerkzugriff** – die App arbeitet komplett offline, es werden keine Daten übertragen
- **Keine Telemetrie oder Analyse** – was du einfügst, bleibt auf deinem Rechner
- Die einzige Datei die geschrieben wird ist `config.json` (Hotkey, Delay, Autostart-Einstellung)

## Technische Details

- **Sprache:** Rust
- **Tastatureingabe-Simulation:**
  - Windows: `SendInput` mit `KEYEVENTF_UNICODE`
  - macOS: `CGEvent` mit `CGEventKeyboardSetUnicodeString`
- **Event Loop:** winit (plattformübergreifend)
- **Tray:** tray-icon + muda
- **Hotkey:** global-hotkey Crate
- **Clipboard:** arboard (plattformübergreifend)
- **Autostart:**
  - Windows: Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
  - macOS: Launch Agent (`~/Library/LaunchAgents/`)
- **Single Instance:**
  - Windows: Named Mutex
  - macOS: File Lock (`~/.macropaste/instance.lock`)

## Lizenz

MIT
