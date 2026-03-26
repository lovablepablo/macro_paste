//! macro_paste – System tray app that sends clipboard text as individual keystrokes.
//!
//! Designed for pasting passwords and text into remote desktop sessions
//! (TeamViewer, pcvisit, etc.) where Ctrl+V doesn't work, e.g. Windows login screens.
//! The app reads the local clipboard and simulates keyboard input via SendInput,
//! which the remote desktop software forwards as normal key presses.

// Hide the console window in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod autostart;
mod clipboard;
mod config;
mod hotkey;
mod keystrokes;
mod tray;

use config::Config;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

/// Main application state, implements the winit event handler
struct App {
    config: Arc<Mutex<Config>>,
    /// The tray icon handle – must be kept alive
    _tray: Option<tray_icon::TrayIcon>,
    /// Menu item IDs for event dispatch
    menu_ids: Option<tray::MenuIds>,
    /// Hotkey manager – must be kept alive for the hotkey to stay registered
    _hotkey_manager: Option<global_hotkey::GlobalHotKeyManager>,
    /// Currently registered hotkey (needed for re-registration)
    current_hotkey: Option<global_hotkey::hotkey::HotKey>,
}

impl App {
    fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(Config::load())),
            _tray: None,
            menu_ids: None,
            _hotkey_manager: None,
            current_hotkey: None,
        }
    }

    /// Execute the paste-as-keystrokes action: read clipboard and type it out
    fn do_paste(&self) {
        let delay_ms = self.config.lock().unwrap().delay_ms;

        match clipboard::get_clipboard_text() {
            Some(text) => {
                // Small delay before typing to allow key release from hotkey
                std::thread::sleep(std::time::Duration::from_millis(100));
                keystrokes::send_string_as_keystrokes(&text, delay_ms);
            }
            None => {
                // No text in clipboard – ignore silently
            }
        }
    }

    /// Handle menu events from the tray context menu
    fn handle_menu_events(&mut self) {
        while let Some(event) = tray::poll_menu_event() {
            let menu_ids = match &self.menu_ids {
                Some(ids) => ids,
                None => return,
            };

            // Check which menu item was clicked
            if event.id() == &menu_ids.paste_now {
                self.do_paste();
            } else if event.id() == &menu_ids.quit {
                std::process::exit(0);
            } else if event.id() == &menu_ids.autostart {
                // Toggle autostart
                let mut cfg = self.config.lock().unwrap();
                cfg.autostart = !cfg.autostart;
                let _ = autostart::toggle_autostart(cfg.autostart);
                let _ = cfg.save();
            } else if let Some((_id, hotkey_str)) = menu_ids
                .hotkey_options
                .iter()
                .find(|(id, _)| id == event.id())
            {
                // Change hotkey
                let hotkey_str = hotkey_str.clone();
                if let (Some(manager), Some(old_hk)) =
                    (&self._hotkey_manager, self.current_hotkey)
                {
                    match hotkey::update_hotkey(manager, old_hk, &hotkey_str) {
                        Ok(new_hk) => {
                            self.current_hotkey = Some(new_hk);
                            let mut cfg = self.config.lock().unwrap();
                            cfg.hotkey = hotkey_str;
                            let _ = cfg.save();
                        }
                        Err(e) => eprintln!("Failed to update hotkey: {e}"),
                    }
                }
            } else if let Some((_id, delay_ms)) = menu_ids
                .delay_options
                .iter()
                .find(|(id, _)| id == event.id())
            {
                // Change delay
                let mut cfg = self.config.lock().unwrap();
                cfg.delay_ms = *delay_ms;
                let _ = cfg.save();
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Initialize tray and hotkey on first resume (app start)
        if self._tray.is_some() {
            return; // Already initialized
        }

        let cfg = self.config.lock().unwrap().clone();

        // Set up the system tray
        match tray::build_tray(&cfg.hotkey, cfg.delay_ms, cfg.autostart) {
            Ok((tray_icon, menu_ids)) => {
                self._tray = Some(tray_icon);
                self.menu_ids = Some(menu_ids);
            }
            Err(e) => {
                eprintln!("Failed to create tray: {e}");
                std::process::exit(1);
            }
        }

        // Register global hotkey
        match hotkey::register_hotkey(&cfg.hotkey) {
            Ok((manager, hk)) => {
                self._hotkey_manager = Some(manager);
                self.current_hotkey = Some(hk);
            }
            Err(e) => {
                eprintln!("Failed to register hotkey: {e}");
            }
        }

        // Sync autostart state with registry
        let actual_autostart = autostart::is_autostart_enabled();
        if actual_autostart != cfg.autostart {
            let mut cfg_mut = self.config.lock().unwrap();
            cfg_mut.autostart = actual_autostart;
            let _ = cfg_mut.save();
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {
        // No windows to handle – we only use the tray
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Check for hotkey events – only trigger on key press, not release
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == HotKeyState::Pressed {
                if let Some(hk) = self.current_hotkey {
                    if event.id() == hk.id() {
                        self.do_paste();
                    }
                }
            }
        }

        // Check for menu events
        self.handle_menu_events();
    }
}

fn main() {
    // Create the winit event loop (drives the Windows message pump)
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Run the app – this blocks until exit
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
