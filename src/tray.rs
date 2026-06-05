//! Tray module – builds the system tray icon and context menu.
//! Uses tray-icon's re-exported muda types to avoid version mismatches.

use tray_icon::menu::{
    CheckMenuItem, Menu, MenuEvent, MenuId, PredefinedMenuItem, Submenu,
};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Embedded icon bytes – ICO on Windows (multi-resolution), template PNG on macOS
#[cfg(target_os = "windows")]
const ICON_BYTES: &[u8] = include_bytes!("../assets/macro_paste.ico");
#[cfg(target_os = "macos")]
const ICON_BYTES: &[u8] = include_bytes!("../assets/macro_paste_tray.png");

/// Collection of menu item handles for event handling and visual updates
pub struct MenuIds {
    pub paste_now: MenuId,
    pub quit: MenuId,
    pub autostart: CheckMenuItem,
    pub hotkey_submenu: Submenu,
    pub hotkey_items: Vec<(CheckMenuItem, String)>,
    pub delay_submenu: Submenu,
    pub delay_items: Vec<(CheckMenuItem, u64)>,
}

/// Build the tray icon and context menu, returns the tray icon handle and menu handles
pub fn build_tray(
    current_hotkey: &str,
    current_delay_ms: u64,
    autostart_enabled: bool,
) -> Result<(TrayIcon, MenuIds), String> {
    // Version label (disabled, non-clickable)
    let version_item = tray_icon::menu::MenuItem::new(
        format!("macro_paste v{}", env!("CARGO_PKG_VERSION")),
        false,
        None,
    );

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
    let mut hotkey_items = Vec::new();
    for choice in &hotkey_choices {
        let item = CheckMenuItem::new(*choice, true, *choice == current_hotkey, None);
        hotkey_submenu.append(&item).map_err(|e| e.to_string())?;
        hotkey_items.push((item, choice.to_string()));
    }

    // Delay submenu with preset values
    let delay_submenu = Submenu::new(format!("Delay: {current_delay_ms}ms"), true);
    let delay_choices: Vec<u64> = vec![10, 20, 30, 50, 100, 200];
    let mut delay_items = Vec::new();
    for &ms in &delay_choices {
        let label = format!("{ms}ms");
        let item = CheckMenuItem::new(&label, true, ms == current_delay_ms, None);
        delay_submenu.append(&item).map_err(|e| e.to_string())?;
        delay_items.push((item, ms));
    }

    // Autostart checkbox
    let autostart_item = CheckMenuItem::new("Autostart", true, autostart_enabled, None);

    // Quit item
    let quit_item = tray_icon::menu::MenuItem::new("Quit", true, None);
    let quit_id = quit_item.id().clone();

    // Assemble the full menu
    let menu = Menu::new();
    menu.append(&version_item).map_err(|e| e.to_string())?;
    // macOS RDP hint (disabled, informational): RDP strips synthesized modifier
    // keys in scancode mode, so uppercase/special chars need Unicode mode.
    #[cfg(target_os = "macos")]
    {
        let rdp_hint = tray_icon::menu::MenuItem::new(
            "RDP tip: Keyboard Mode → Unicode (⌃⌘U)",
            false,
            None,
        );
        menu.append(&rdp_hint).map_err(|e| e.to_string())?;
    }
    menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
    menu.append(&paste_now).map_err(|e| e.to_string())?;
    menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
    menu.append(&hotkey_submenu).map_err(|e| e.to_string())?;
    menu.append(&delay_submenu).map_err(|e| e.to_string())?;
    menu.append(&autostart_item).map_err(|e| e.to_string())?;
    menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
    menu.append(&quit_item).map_err(|e| e.to_string())?;

    // Load the app icon from embedded ICO bytes
    let icon = load_icon_from_ico();

    // Build the tray icon. `with_icon_as_template` is macOS-only and tells the
    // system menu bar to auto-invert the (black-on-transparent) icon for light
    // and dark mode. The call is a no-op on Windows.
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("macro_paste – Paste clipboard as keystrokes")
        .with_icon(icon)
        .with_icon_as_template(true)
        .build()
        .map_err(|e| format!("Failed to create tray icon: {e}"))?;

    let ids = MenuIds {
        paste_now: paste_now_id,
        quit: quit_id,
        autostart: autostart_item,
        hotkey_submenu,
        hotkey_items,
        delay_submenu,
        delay_items,
    };

    Ok((tray, ids))
}

/// Receive the next menu event (non-blocking)
pub fn poll_menu_event() -> Option<MenuEvent> {
    MenuEvent::receiver().try_recv().ok()
}

/// Load the app icon from embedded image bytes, decode to RGBA.
/// Resizes to 32x32 only when the source is larger – the macOS tray PNG is
/// already 32x32 (16pt @2x) and shouldn't be touched, while the Windows .ico
/// contains multiple resolutions and the largest one needs to be downscaled.
fn load_icon_from_ico() -> Icon {
    use image::ImageReader;
    use std::io::Cursor;

    let reader = ImageReader::new(Cursor::new(ICON_BYTES))
        .with_guessed_format()
        .expect("Failed to read icon format");

    let img = reader.decode().expect("Failed to decode icon");
    let img = if img.width() != 32 || img.height() != 32 {
        img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());

    Icon::from_rgba(rgba.into_raw(), w, h).expect("Failed to create tray icon")
}
