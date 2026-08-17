// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
            use tauri::tray::TrayIconBuilder;
            use tauri_plugin_dialog::DialogExt;

            // Create menu items
            let show_hello = MenuItem::with_id(app, "show_hello", "Show Hello World", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;

            // Create the menu
            let menu = Menu::with_items(app, &[&show_hello, &separator, &quit])?;

            // Load custom tray icon
            let icon_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons/traytest.png");
            let icon_bytes = std::fs::read(&icon_path).expect("Failed to read tray icon");
            let tray_icon = tauri::image::Image::from_bytes(&icon_bytes).expect("Failed to load tray icon");

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show_hello" => {
                            let _ = app.dialog()
                                .message("Hello World from the system tray!")
                                .title("Hello World")
                                .kind(tauri_plugin_dialog::MessageDialogKind::Info)
                                .show(|_| {});
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
