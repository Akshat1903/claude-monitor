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
use tauri::{LogicalPosition, Manager, PhysicalPosition, Position, Rect, Size, WebviewWindow};

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
                        toggle_window(&app_handle, Some(rect));
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

fn toggle_window(app: &tauri::AppHandle, tray_rect: Option<Rect>) {
    let Some(win) = app.get_webview_window("main") else { return };
    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
    } else {
        // Position once so the window starts roughly in the right place, then again
        // after show() so we recompute using the target display's scale factor and
        // outer_size. Without the second pass, the first click on a secondary display
        // lands at stale coordinates (based on the primary's scale).
        if let Some(rect) = tray_rect {
            position_under_tray(&win, rect);
        }
        let _ = win.show();
        if let Some(rect) = tray_rect {
            position_under_tray(&win, rect);
        }
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

fn position_under_tray(win: &WebviewWindow, tray_rect: Rect) {
    // Tray rects come back in the PHYSICAL pixel coordinate system of the tray's
    // own display — not the window's current display. So we can't use
    // win.scale_factor(): we have to find which monitor contains the tray and use
    // THAT monitor's scale to convert to logical coords.
    const GAP: f64 = 4.0;
    const WIN_W: f64 = 340.0; // logical, from tauri.conf.json

    let (tray_x_p, tray_y_p, tray_w_p, tray_h_p) = match (tray_rect.position, tray_rect.size) {
        (Position::Physical(p), Size::Physical(s)) => (
            p.x as f64,
            p.y as f64,
            s.width as f64,
            s.height as f64,
        ),
        (Position::Logical(p), Size::Logical(s)) => {
            let cx = p.x + s.width / 2.0;
            let x = cx - WIN_W / 2.0;
            let y = p.y + s.height + GAP;
            let _ = win.set_position(LogicalPosition::new(x, y));
            return;
        }
        _ => return,
    };

    // Look up the tray's display by testing which monitor's physical rectangle
    // contains the tray's physical top-left.
    let app = win.app_handle();
    let monitors = app.available_monitors().unwrap_or_default();
    let target = monitors.iter().find(|m| {
        let mp = m.position();
        let ms = m.size();
        let mx = mp.x as f64;
        let my = mp.y as f64;
        let mw = ms.width as f64;
        let mh = ms.height as f64;
        tray_x_p >= mx && tray_x_p < mx + mw && tray_y_p >= my && tray_y_p < my + mh
    });
    let target_scale = target.map(|m| m.scale_factor()).unwrap_or(1.0);

    // Convert tray rect to logical using the target monitor's scale.
    let tray_x = tray_x_p / target_scale;
    let tray_y = tray_y_p / target_scale;
    let tray_w = tray_w_p / target_scale;
    let tray_h = tray_h_p / target_scale;

    let tray_center_x = tray_x + tray_w / 2.0;
    let x = tray_center_x - WIN_W / 2.0;
    let y = tray_y + tray_h + GAP;

    eprintln!(
        "[pos] tray_phys=({:.0},{:.0})+{:.0}x{:.0} target_scale={} win_scale={} → win_log=({:.0},{:.0})",
        tray_x_p,
        tray_y_p,
        tray_w_p,
        tray_h_p,
        target_scale,
        win.scale_factor().unwrap_or(1.0),
        x,
        y
    );
    let _ = win.set_position(LogicalPosition::new(x, y));
}
