mod api;
mod commands;
mod keychain;
mod notify;
mod stats;
mod store;
mod types;

use commands::AppState;
use tauri::image::Image;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{LogicalPosition, Manager, PhysicalPosition, Position, WebviewWindow};

const TRAY_ICON_BYTES: &[u8] = include_bytes!("../icons/tray@2x.png");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let state = AppState::new();
            app.manage(state.clone());

            let app_handle = app.handle().clone();
            let tray_icon = Image::from_bytes(TRAY_ICON_BYTES)
                .expect("tray icon should decode");
            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Claude Monitor")
                .icon(tray_icon)
                .icon_as_template(true)
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        rect,
                        ..
                    } = event
                    {
                        toggle_window(&app_handle, Some(rect.position));
                    }
                })
                .build(app)?;

            // Kick off initial fetch + background refresh loop.
            let app_handle2 = app.handle().clone();
            let state2 = state.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    commands::refresh_impl(state2.clone(), app_handle2.clone()).await;
                    tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                }
            });

            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_visible_on_all_workspaces(true);
                #[cfg(target_os = "macos")]
                {
                    set_panel_behavior(&win);
                    install_global_click_monitor(win.clone());
                }
            }

            // Seed the tray label from whatever we have cached so it's not blank on launch.
            if let Some(tray) = app.tray_by_id("main-tray") {
                let title = {
                    let g = state.inner.blocking_lock();
                    g.usage
                        .as_ref()
                        .and_then(|u| u.five_hour.as_ref())
                        .map(|b| format!(" {}%", b.utilization as i64))
                };
                let _ = tray.set_title(title);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::refresh,
            commands::get_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn toggle_window(app: &tauri::AppHandle, tray_pos: Option<Position>) {
    let Some(win) = app.get_webview_window("main") else { return };
    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
    } else {
        if let Some(pos) = tray_pos {
            position_under_tray(&win, pos);
        }
        let _ = win.show();
        let _ = win.set_focus();
        #[cfg(target_os = "macos")]
        set_panel_behavior(&win);
    }
}

#[cfg(target_os = "macos")]
fn set_panel_behavior(win: &WebviewWindow) {
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};

    let Ok(ptr) = win.ns_window() else {
        eprintln!("[panel] ns_window() returned Err");
        return;
    };
    if ptr.is_null() {
        eprintln!("[panel] ns_window() returned null");
        return;
    }

    unsafe {
        // Convert NSWindow → NSPanel so the window can overlay fullscreen apps.
        let panel_class: *const objc2::runtime::AnyClass = objc2::class!(NSPanel);
        let raw_obj = ptr as *mut AnyObject;
        objc2::ffi::object_setClass(raw_obj, panel_class.cast());

        let ns_window = &*(ptr as *mut NSWindow);

        // Nonactivating panel: don't steal focus from whatever app is frontmost. This is
        // what actually keeps the panel alive over a fullscreen app. The cost is that
        // `Focused(false)` never fires (the panel never gained key), so we dismiss via
        // a global NSEvent click monitor installed at startup.
        let current_mask: usize = objc2::msg_send![ns_window, styleMask];
        let _: () = objc2::msg_send![ns_window, setStyleMask: current_mask | (1usize << 7)];

        let extra = NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::FullScreenAuxiliary
            | NSWindowCollectionBehavior::Stationary;
        ns_window.setCollectionBehavior(extra);

        let _: () = objc2::msg_send![ns_window, setLevel: 1000isize];
    }
}

#[cfg(target_os = "macos")]
fn install_global_click_monitor(win: WebviewWindow) {
    use block2::RcBlock;
    use objc2::runtime::AnyObject;

    // NSEventMaskLeftMouseDown (1<<1) | NSEventMaskRightMouseDown (1<<3)
    const MASK: u64 = (1 << 1) | (1 << 3);

    let win_cb = win.clone();
    let block = RcBlock::new(move |_event: *mut AnyObject| {
        if win_cb.is_visible().unwrap_or(false) {
            let _ = win_cb.hide();
        }
    });

    unsafe {
        let ns_event_class: *const objc2::runtime::AnyClass = objc2::class!(NSEvent);
        let _monitor: *mut AnyObject = objc2::msg_send![
            ns_event_class,
            addGlobalMonitorForEventsMatchingMask: MASK,
            handler: &*block
        ];
        // Keep the block alive for the lifetime of the app — we install this once,
        // and the monitor lives until termination.
        std::mem::forget(block);
    }
}

fn position_under_tray(win: &WebviewWindow, tray_pos: Position) {
    // Place the popup below the menu bar + tray icon so nothing overlaps the system menu.
    // Menu bar height is ~24pt on Retina. Using 28pt gives a small gap below the tray.
    const MENU_BAR_GAP: f64 = 28.0;
    let size = win.outer_size().unwrap_or(tauri::PhysicalSize::new(340, 460));
    let scale = win.scale_factor().unwrap_or(1.0);
    match tray_pos {
        Position::Physical(p) => {
            let x = p.x as f64 - (size.width as f64 / 2.0);
            let y = MENU_BAR_GAP * scale;
            let _ = win.set_position(PhysicalPosition::new(x, y));
        }
        Position::Logical(p) => {
            let logical_w = size.width as f64 / scale;
            let x = p.x - logical_w / 2.0;
            let _ = win.set_position(LogicalPosition::new(x, MENU_BAR_GAP));
        }
    }
}
