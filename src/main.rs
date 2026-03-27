//! macro_paste – System tray app that sends clipboard text as individual keystrokes.
//!
//! Designed for pasting passwords and text into remote desktop sessions
//! (TeamViewer, pcvisit, etc.) where Ctrl+V doesn't work, e.g. Windows login screens.
//! The app reads the local clipboard and simulates keyboard input,
//! which the remote desktop software forwards as normal key presses.
//!
//! Cross-platform: works on Windows and macOS.

// Hide the console window in release builds (Windows only)
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod autostart;
mod clipboard;
mod config;
mod hotkey;
mod keystrokes;
mod notify;
mod single_instance;
mod tray;

use config::Config;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

/// Decoded menu action – avoids holding borrows across mutation
enum MenuAction {
    Paste,
    Quit,
    ToggleAutostart,
    ChangeHotkey(String),
    ChangeDelay(u64),
}

/// Main application state, implements the winit event handler
struct App {
    config: Config,
    /// The tray icon handle – must be kept alive
    _tray: Option<tray_icon::TrayIcon>,
    /// Menu item handles for event dispatch and visual updates
    menu_ids: Option<tray::MenuIds>,
    /// Hotkey manager – must be kept alive for the hotkey to stay registered
    _hotkey_manager: Option<global_hotkey::GlobalHotKeyManager>,
    /// Currently registered hotkey (needed for re-registration)
    current_hotkey: Option<global_hotkey::hotkey::HotKey>,
    /// Set to true to trigger a clean exit on the next event loop tick
    should_exit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            config: Config::load(),
            _tray: None,
            menu_ids: None,
            _hotkey_manager: None,
            current_hotkey: None,
            should_exit: false,
        }
    }

    /// Execute the paste-as-keystrokes action: read clipboard and type it out.
    /// Runs in a background thread to avoid blocking the event loop.
    fn do_paste(&self) {
        let delay_ms = self.config.delay_ms;

        if let Some(text) = clipboard::get_clipboard_text() {
            std::thread::spawn(move || {
                // Wait for the user to release the hotkey modifiers (Ctrl+Shift) before
                // typing starts – otherwise those modifiers corrupt the output.
                // macOS needs a longer delay because modifier release is slower there.
                #[cfg(target_os = "macos")]
                std::thread::sleep(std::time::Duration::from_millis(350));
                #[cfg(target_os = "windows")]
                std::thread::sleep(std::time::Duration::from_millis(300));
                keystrokes::send_string_as_keystrokes(&text, delay_ms);
            });
        }
    }

    /// Handle menu events from the tray context menu.
    /// First decodes the event into a MenuAction (releasing borrows on menu_ids),
    /// then executes the action with full mutable access to self.
    fn handle_menu_events(&mut self) {
        while let Some(event) = tray::poll_menu_event() {
            // Decode which action to take while borrowing menu_ids immutably
            let action = {
                let menu_ids = match &self.menu_ids {
                    Some(ids) => ids,
                    None => return,
                };

                if event.id() == &menu_ids.paste_now {
                    MenuAction::Paste
                } else if event.id() == &menu_ids.quit {
                    MenuAction::Quit
                } else if event.id() == menu_ids.autostart.id() {
                    MenuAction::ToggleAutostart
                } else if let Some((_, hotkey_str)) = menu_ids
                    .hotkey_items
                    .iter()
                    .find(|(item, _)| item.id() == event.id())
                {
                    MenuAction::ChangeHotkey(hotkey_str.clone())
                } else if let Some((_, delay_ms)) = menu_ids
                    .delay_items
                    .iter()
                    .find(|(item, _)| item.id() == event.id())
                {
                    MenuAction::ChangeDelay(*delay_ms)
                } else {
                    continue;
                }
            }; // borrow on self.menu_ids released here

            // Execute the action with full access to self
            match action {
                MenuAction::Paste => self.do_paste(),
                MenuAction::Quit => {
                    self.should_exit = true;
                }
                MenuAction::ToggleAutostart => {
                    self.config.autostart = !self.config.autostart;
                    if let Err(e) = autostart::toggle_autostart(self.config.autostart) {
                        notify::show_error("Autostart-Fehler", &e);
                    }
                    if let Some(ids) = &self.menu_ids {
                        ids.autostart.set_checked(self.config.autostart);
                    }
                    if let Err(e) = self.config.save() {
                        notify::show_error("Config-Fehler", &e);
                    }
                }
                MenuAction::ChangeHotkey(hotkey_str) => {
                    // Already active – restore checkmark (muda auto-toggled it) and skip
                    if hotkey_str == self.config.hotkey {
                        if let Some(ids) = &self.menu_ids {
                            for (item, hs) in &ids.hotkey_items {
                                item.set_checked(*hs == hotkey_str);
                            }
                        }
                        continue;
                    }

                    if let (Some(manager), Some(old_hk)) =
                        (&self._hotkey_manager, self.current_hotkey)
                    {
                        match hotkey::update_hotkey(manager, old_hk, &hotkey_str) {
                            Ok(new_hk) => {
                                self.current_hotkey = Some(new_hk);
                                self.config.hotkey = hotkey_str.clone();
                                if let Err(e) = self.config.save() {
                                    notify::show_error("Config-Fehler", &e);
                                }
                                // Update menu visuals – only the new one is checked
                                if let Some(ids) = &self.menu_ids {
                                    for (item, hs) in &ids.hotkey_items {
                                        item.set_checked(*hs == hotkey_str);
                                    }
                                    ids.hotkey_submenu
                                        .set_text(&format!("Hotkey: {hotkey_str}"));
                                }
                            }
                            Err(e) => {
                                notify::show_error("Hotkey-Fehler", &e);
                                // Revert checkmarks to current config
                                if let Some(ids) = &self.menu_ids {
                                    for (item, hs) in &ids.hotkey_items {
                                        item.set_checked(*hs == self.config.hotkey);
                                    }
                                }
                            }
                        }
                    }
                }
                MenuAction::ChangeDelay(delay_ms) => {
                    // Already active – restore checkmark and skip
                    if delay_ms == self.config.delay_ms {
                        if let Some(ids) = &self.menu_ids {
                            for (item, ms) in &ids.delay_items {
                                item.set_checked(*ms == delay_ms);
                            }
                        }
                        continue;
                    }

                    self.config.delay_ms = delay_ms;
                    if let Err(e) = self.config.save() {
                        notify::show_error("Config-Fehler", &e);
                    }
                    // Update menu visuals
                    if let Some(ids) = &self.menu_ids {
                        for (item, ms) in &ids.delay_items {
                            item.set_checked(*ms == delay_ms);
                        }
                        ids.delay_submenu
                            .set_text(&format!("Delay: {delay_ms}ms"));
                    }
                }
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

        let cfg = self.config.clone();

        // Set up the system tray
        match tray::build_tray(&cfg.hotkey, cfg.delay_ms, cfg.autostart) {
            Ok((tray_icon, menu_ids)) => {
                self._tray = Some(tray_icon);
                self.menu_ids = Some(menu_ids);
            }
            Err(e) => {
                notify::show_error("Tray-Fehler", &e);
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
                notify::show_error("Hotkey-Fehler", &e);
            }
        }

        // Sync autostart state with system
        let actual_autostart = autostart::is_autostart_enabled();
        if actual_autostart != cfg.autostart {
            self.config.autostart = actual_autostart;
            let _ = self.config.save();
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

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Clean exit requested via menu
        if self.should_exit {
            event_loop.exit();
            return;
        }

        // Poll every 50ms so hotkey/menu events are detected promptly
        event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
            std::time::Instant::now() + std::time::Duration::from_millis(50),
        ));

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
    // Prevent multiple instances
    let _lock = single_instance::ensure_single_instance();

    // Create the event loop (cross-platform message pump)
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Run the app – this blocks until exit
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("Event loop error");
}
