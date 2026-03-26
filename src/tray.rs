//! Tray module – builds the system tray icon and context menu.
//! Uses tray-icon's re-exported muda types to avoid version mismatches.

use tray_icon::menu::{
    CheckMenuItem, Menu, MenuEvent, MenuId, PredefinedMenuItem, Submenu,
};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Collection of menu item IDs for event handling
pub struct MenuIds {
    pub paste_now: MenuId,
    pub autostart: MenuId,
    pub quit: MenuId,
    /// Hotkey option IDs mapped to their label strings
    pub hotkey_options: Vec<(MenuId, String)>,
    /// Delay option IDs mapped to their delay values in ms
    pub delay_options: Vec<(MenuId, u64)>,
}

/// Build the tray icon and context menu, returns the tray icon handle and menu IDs
pub fn build_tray(
    current_hotkey: &str,
    current_delay_ms: u64,
    autostart_enabled: bool,
) -> Result<(TrayIcon, MenuIds), String> {
    // Create menu items
    let paste_now = tray_icon::menu::MenuItem::new("Paste as Keystrokes", true, None);
    let paste_now_id = paste_now.id().clone();

    // Hotkey submenu with common combinations
    let hotkey_submenu = Submenu::new(format!("Hotkey: {current_hotkey}"), true);
    let hotkey_choices = vec![
        "Ctrl+Shift+V",
        "Ctrl+Shift+P",
        "Ctrl+Alt+V",
        "Ctrl+Alt+P",
    ];
    let mut hotkey_options = Vec::new();
    for choice in &hotkey_choices {
        let item = CheckMenuItem::new(*choice, true, *choice == current_hotkey, None);
        hotkey_options.push((item.id().clone(), choice.to_string()));
        hotkey_submenu.append(&item).map_err(|e| e.to_string())?;
    }

    // Delay submenu with preset values
    let delay_submenu = Submenu::new(format!("Delay: {current_delay_ms}ms"), true);
    let delay_choices: Vec<u64> = vec![10, 20, 30, 50, 100, 200];
    let mut delay_options = Vec::new();
    for &ms in &delay_choices {
        let label = format!("{ms}ms");
        let item = CheckMenuItem::new(&label, true, ms == current_delay_ms, None);
        delay_options.push((item.id().clone(), ms));
        delay_submenu.append(&item).map_err(|e| e.to_string())?;
    }

    // Autostart checkbox
    let autostart_item = CheckMenuItem::new("Autostart", true, autostart_enabled, None);
    let autostart_id = autostart_item.id().clone();

    // Quit item
    let quit_item = tray_icon::menu::MenuItem::new("Beenden", true, None);
    let quit_id = quit_item.id().clone();

    // Assemble the full menu
    let menu = Menu::new();
    menu.append(&paste_now).map_err(|e| e.to_string())?;
    menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
    menu.append(&hotkey_submenu).map_err(|e| e.to_string())?;
    menu.append(&delay_submenu).map_err(|e| e.to_string())?;
    menu.append(&autostart_item).map_err(|e| e.to_string())?;
    menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
    menu.append(&quit_item).map_err(|e| e.to_string())?;

    // Create a simple icon (blue square as placeholder)
    let icon = create_default_icon();

    // Build the tray icon
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("macro_paste – Clipboard als Keystrokes")
        .with_icon(icon)
        .build()
        .map_err(|e| format!("Failed to create tray icon: {e}"))?;

    let ids = MenuIds {
        paste_now: paste_now_id,
        autostart: autostart_id,
        quit: quit_id,
        hotkey_options,
        delay_options,
    };

    Ok((tray, ids))
}

/// Receive the next menu event (non-blocking)
pub fn poll_menu_event() -> Option<MenuEvent> {
    MenuEvent::receiver().try_recv().ok()
}

/// Create a simple colored icon programmatically (no .ico file needed)
fn create_default_icon() -> Icon {
    // 16x16 RGBA icon – a simple filled square in a teal/blue color
    let size = 16u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            // Create a simple shape with a border
            let border = x == 0 || x == size - 1 || y == 0 || y == size - 1;
            if border {
                // Dark border
                rgba.extend_from_slice(&[20, 80, 120, 255]);
            } else {
                // Teal fill
                rgba.extend_from_slice(&[40, 160, 200, 255]);
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("Failed to create icon")
}
