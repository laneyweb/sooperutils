// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn show_about_window(app: tauri::AppHandle) {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
    
    // Check if about window already exists
    if app.get_webview_window("about").is_some() {
        return;
    }
    
    let _ = WebviewWindowBuilder::new(
        &app,
        "about",
        WebviewUrl::App("about.html".into())
    )
    .title("About")
    .inner_size(400.0, 300.0)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .build();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, show_main_window, show_about_window])
        .on_window_event(|window, event| {
            use tauri::WindowEvent;
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
            use tauri::tray::TrayIconBuilder;

            // Create menu items
            let show_main = MenuItem::with_id(app, "show_main", "Open Main Window", true, None::<&str>)?;
            let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;

            // Create the menu
            let menu = Menu::with_items(app, &[&show_main, &separator, &about, &separator, &quit])?;

            // Load custom tray icon
            let icon_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons/traytest.png");
            let icon_bytes = std::fs::read(&icon_path).expect("Failed to read tray icon");
            let tray_icon = tauri::image::Image::from_bytes(&icon_bytes).expect("Failed to load tray icon");

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show_main" => {
                            let app_handle = app.clone();
                            let _ = app.run_on_main_thread(move || {
                                show_main_window(app_handle);
                            });
                        }
                        "about" => {
                            let app_handle = app.clone();
                            let _ = app.run_on_main_thread(move || {
                                show_about_window(app_handle);
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::TrayIconEvent;
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            let app_handle = tray.app_handle().clone();
                            let _ = tray.app_handle().run_on_main_thread(move || {
                                show_main_window(app_handle);
                            });
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
