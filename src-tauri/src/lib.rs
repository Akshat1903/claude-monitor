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
    }
}

fn position_under_tray(win: &WebviewWindow, tray_pos: Position) {
    let size = win.outer_size().unwrap_or(tauri::PhysicalSize::new(340, 460));
    let scale = win.scale_factor().unwrap_or(1.0);
    match tray_pos {
        Position::Physical(p) => {
            let x = p.x as f64 - (size.width as f64 / 2.0);
            let y = p.y as f64 + 8.0;
            let _ = win.set_position(PhysicalPosition::new(x, y));
        }
        Position::Logical(p) => {
            let logical_w = size.width as f64 / scale;
            let x = p.x - logical_w / 2.0;
            let y = p.y + 8.0;
            let _ = win.set_position(LogicalPosition::new(x, y));
        }
    }
}
