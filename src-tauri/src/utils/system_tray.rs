use tauri::{tray::{TrayIconBuilder}, Manager, AppHandle, Runtime};
use tauri::menu::{Menu, MenuItemBuilder};
use tauri::tray::{TrayIconEvent};

pub fn init_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), tauri::Error> {
    let toggle_i = MenuItemBuilder::with_id("toggle", "Hide").build(app)?;
    let kill_i = MenuItemBuilder::with_id("kill", "Force kill").build(app)?;
    let menu1 = Menu::with_items(app, &[&toggle_i, &kill_i])?;

    let _ = TrayIconBuilder::with_id("tray_1").tooltip(&app.config().product_name.clone().unwrap()).show_menu_on_left_click(true).menu(&menu1)
        .icon(app.default_window_icon().unwrap().clone())
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "toggle" => {
                if let Some(window) = app.get_window("main") {
                    let new_title = if window.is_visible().unwrap_or_default() {
                        let _ = window.hide();
                        "Show"
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                        "Hide"
                    };
                    toggle_i.set_text(new_title).unwrap();
                }
            }
            "kill" => {
                app.cleanup_before_exit();
                app.exit(0);
                std::process::exit(0);
            }
            _ => ()
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click { .. } => {
                let app = tray.app_handle();
                if let Some(window) = app.get_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            TrayIconEvent::Enter { .. } => {}
            TrayIconEvent::Move { .. } => {}
            TrayIconEvent::Leave { .. } => {}
            _ => {}
        }).build(app)?;
    Ok(())
}