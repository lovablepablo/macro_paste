# macro_paste

> **[English](README.md)**

Windows System-Tray-App, die Clipboard-Text als einzelne Tastaturanschläge sendet.

## Wozu?

In Fernwartungssitzungen (TeamViewer, pcvisit, AnyDesk etc.) kann man im **Windows Login-Dialog** der Zielmaschine kein `Ctrl+V` verwenden. macro_paste löst das Problem: Es liest den Text aus der lokalen Zwischenablage und simuliert einzelne Tastatureingaben via `SendInput`. Die Fernwartungssoftware leitet diese dann wie echte Tastaturanschläge an die Zielmaschine weiter.

## Features

- **System Tray** – läuft unauffällig im Hintergrund
- **Globaler Hotkey** – Standard: `Ctrl+Shift+V`, änderbar über Tray-Menü
- **Unicode-Support** – Sonderzeichen (äöü, @, €, ß) werden korrekt gesendet
- **Konfigurierbarer Delay** – 10 / 20 / 30 / 50 / 100 / 200ms zwischen Anschlägen (Standard: 30ms)
- **Autostart** – optional beim Windows-Start mitlaufen (Registry-basiert)
- **Portable** – einzelne .exe (~620 KB), keine Installation nötig
- **Config-Datei** – `config.json` neben der .exe, wird automatisch erstellt

## Installation

### Option A: Vorkompilierte .exe (empfohlen)

1. `macro_paste.exe` aus dem [neuesten Release](https://github.com/lovablepablo/macro_paste/releases) herunterladen
2. In einen beliebigen Ordner legen (z.B. `C:\Tools\`)
3. Starten – das Tray-Icon erscheint im System Tray

### Option B: Selbst kompilieren

**Voraussetzungen:**
- [Rust](https://rustup.rs/) (inkl. Cargo)
- [Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/) mit "Desktop development with C++" Workload

```bash
git clone https://github.com/lovablepablo/macro_paste.git
cd macro_paste
cargo build --release
```

Die fertige .exe liegt unter `target/release/macro_paste.exe`.

## Verwendung

1. **Text kopieren** – z.B. ein Passwort mit `Ctrl+C` in die Zwischenablage kopieren
2. **Zielfeld fokussieren** – z.B. das Passwort-Feld im Windows Login der Fernwartungssitzung anklicken
3. **Hotkey drücken** – `Ctrl+Shift+V` (Standard) – der Text wird Zeichen für Zeichen eingegeben

### Tray-Menü (Rechtsklick auf das Icon)

| Eintrag | Funktion |
|---------|----------|
| Paste as Keystrokes | Manueller Trigger (alternativ zum Hotkey) |
| Hotkey | Tastenkombination ändern (Ctrl+Shift+V/P, Ctrl+Alt+V/P) |
| Delay | Verzögerung zwischen Tastenanschlägen anpassen |
| Autostart | App beim Windows-Start automatisch starten |
| Beenden | App schließen |

## Konfiguration

Die Einstellungen werden in `config.json` neben der .exe gespeichert:

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
2. Neue `macro_paste.exe` herunterladen und die alte ersetzen
3. App neu starten – die `config.json` bleibt erhalten

## Technische Details

- **Sprache:** Rust
- **Keystroke-Methode:** `SendInput` mit `KEYEVENTF_UNICODE` – sendet Unicode-Zeichen direkt ohne VirtualKey-Mapping
- **Event Loop:** winit (Windows Message Pump)
- **Tray:** tray-icon + muda
- **Hotkey:** global-hotkey Crate
- **Clipboard:** clipboard-win (Windows API)
- **Autostart:** Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)

## Lizenz

MIT
