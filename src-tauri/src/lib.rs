// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::io::Write;
use std::sync::{Arc, LazyLock, RwLock as StdRwLock, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;

use chrono::{DateTime, Datelike, Local, TimeZone};
use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::RwLock;
use rdev::{Event, EventType, listen};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

// macOS privacy-permission checks (Accessibility / Input Monitoring).
// These are required for rdev to receive global keyboard events. macOS
// SILENTLY drops events (no error) when the process lacks the permission,
// and ad-hoc signed apps lose their grant every time they are rebuilt
// (the grant is keyed to the binary's cdhash).
#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightListenEventAccess() -> bool;
}

#[cfg(target_os = "macos")]
fn macos_accessibility_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

#[cfg(target_os = "macos")]
fn macos_input_monitoring_granted() -> bool {
    unsafe { CGPreflightListenEventAccess() }
}

#[cfg(not(target_os = "macos"))]
fn macos_accessibility_trusted() -> bool {
    true
}

#[cfg(not(target_os = "macos"))]
fn macos_input_monitoring_granted() -> bool {
    true
}

const KEY_STORE_KEY: &str = "keypress_data";
const STORE_PATH: &str = "store.bin";
const DOCK_ICON_VISIBLE_KEY: &str = "settings_dock_icon_visible";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct KeyPressData {
    timestamps: Vec<u64>, // Unix timestamps in milliseconds
    // Specific key timestamps
    space_timestamps: Vec<u64>,
    backspace_timestamps: Vec<u64>,
    enter_timestamps: Vec<u64>,
    escape_timestamps: Vec<u64>,
}

impl KeyPressData {
    fn add_press(&mut self, timestamp: u64) {
        self.timestamps.push(timestamp);
    }

    fn add_specific_key(&mut self, key: SpecificKey, timestamp: u64) {
        match key {
            SpecificKey::Space => self.space_timestamps.push(timestamp),
            SpecificKey::Backspace => self.backspace_timestamps.push(timestamp),
            SpecificKey::Enter => self.enter_timestamps.push(timestamp),
            SpecificKey::Escape => self.escape_timestamps.push(timestamp),
        }
    }

    fn count_in_range(&self, start: u64, end: u64) -> u64 {
        self.timestamps
            .iter()
            .filter(|&&ts| ts >= start && ts <= end)
            .count() as u64
    }

    fn count_specific_in_range(&self, key: SpecificKey, start: u64, end: u64) -> u64 {
        let timestamps = match key {
            SpecificKey::Space => &self.space_timestamps,
            SpecificKey::Backspace => &self.backspace_timestamps,
            SpecificKey::Enter => &self.enter_timestamps,
            SpecificKey::Escape => &self.escape_timestamps,
        };
        timestamps
            .iter()
            .filter(|&&ts| ts >= start && ts <= end)
            .count() as u64
    }

    fn total(&self) -> u64 {
        self.timestamps.len() as u64
    }

    fn total_specific(&self, key: SpecificKey) -> u64 {
        let timestamps = match key {
            SpecificKey::Space => &self.space_timestamps,
            SpecificKey::Backspace => &self.backspace_timestamps,
            SpecificKey::Enter => &self.enter_timestamps,
            SpecificKey::Escape => &self.escape_timestamps,
        };
        timestamps.len() as u64
    }

    // Clean up old data (older than 2 years)
    fn cleanup(&mut self) {
        let cutoff = current_timestamp_ms() - 2 * 365 * 24 * 60 * 60 * 1000;
        self.timestamps.retain(|&ts| ts > cutoff);
        self.space_timestamps.retain(|&ts| ts > cutoff);
        self.backspace_timestamps.retain(|&ts| ts > cutoff);
        self.enter_timestamps.retain(|&ts| ts > cutoff);
        self.escape_timestamps.retain(|&ts| ts > cutoff);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpecificKey {
    Space,
    Backspace,
    Enter,
    Escape,
}

impl SpecificKey {
    fn from_rdev_key(key: rdev::Key) -> Option<Self> {
        use rdev::Key;
        match key {
            Key::Space => Some(SpecificKey::Space),
            Key::Backspace => Some(SpecificKey::Backspace),
            Key::Return => Some(SpecificKey::Enter),
            Key::Escape => Some(SpecificKey::Escape),
            _ => None,
        }
    }

}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

fn day_start_ts() -> u64 {
    let now = Local::now();
    let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let dt: DateTime<Local> = Local.from_local_datetime(&start).unwrap();
    dt.timestamp_millis() as u64
}

fn week_start_ts() -> u64 {
    let now = Local::now();
    let days_since_monday = now.weekday().num_days_from_monday() as i64;
    let start = now.date_naive() - chrono::Duration::days(days_since_monday);
    let start = start.and_hms_opt(0, 0, 0).unwrap();
    let dt: DateTime<Local> = Local.from_local_datetime(&start).unwrap();
    dt.timestamp_millis() as u64
}

fn month_start_ts() -> u64 {
    let now = Local::now();
    let start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
    let dt: DateTime<Local> = Local.from_local_datetime(&start).unwrap();
    dt.timestamp_millis() as u64
}

fn year_start_ts() -> u64 {
    let now = Local::now();
    let start = now.date_naive().with_month(1).unwrap().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
    let dt: DateTime<Local> = Local.from_local_datetime(&start).unwrap();
    dt.timestamp_millis() as u64
}

#[derive(Debug, Serialize, Deserialize)]
struct KeyPressStats {
    day: u64,
    week: u64,
    month: u64,
    year: u64,
    total: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpecificKeyTimeframes {
    day: u64,
    week: u64,
    month: u64,
    year: u64,
    total: u64,
}

impl SpecificKeyTimeframes {
    fn new(data: &KeyPressData, key: SpecificKey, now: u64) -> Self {
        Self {
            day: data.count_specific_in_range(key, day_start_ts(), now),
            week: data.count_specific_in_range(key, week_start_ts(), now),
            month: data.count_specific_in_range(key, month_start_ts(), now),
            year: data.count_specific_in_range(key, year_start_ts(), now),
            total: data.total_specific(key),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SpecificKeyStats {
    space: SpecificKeyTimeframes,
    backspace: SpecificKeyTimeframes,
    enter: SpecificKeyTimeframes,
    escape: SpecificKeyTimeframes,
}

struct KeyPressTracker {
    data: RwLock<KeyPressData>,
}

impl KeyPressTracker {
    fn new() -> Self {
        Self {
            data: RwLock::new(KeyPressData::default()),
        }
    }

    fn load_from_store(&self, app: &AppHandle) {
        if let Ok(store) = app.store(STORE_PATH) {
            if let Some(value) = store.get(KEY_STORE_KEY) {
                if let Ok(data) = serde_json::from_value::<KeyPressData>(value) {
                    *self.data.write() = data;
                }
            }
        }
    }

    fn save_to_store(&self, app: &AppHandle) {
        if let Ok(store) = app.store(STORE_PATH) {
            let data = self.data.read().clone();
            let _ = store.set(KEY_STORE_KEY, serde_json::to_value(data).unwrap());
            let _ = store.save();
        }
    }

    fn record_press(&self, timestamp: u64) {
        self.data.write().add_press(timestamp);
    }

    fn record_specific_key(&self, key: SpecificKey, timestamp: u64) {
        self.data.write().add_specific_key(key, timestamp);
    }

    fn should_save(&self) -> bool {
        self.data.read().timestamps.len() % 100 == 0
    }

    fn get_stats(&self) -> KeyPressStats {
        let data = self.data.read();
        let now = current_timestamp_ms();
        
        KeyPressStats {
            day: data.count_in_range(day_start_ts(), now),
            week: data.count_in_range(week_start_ts(), now),
            month: data.count_in_range(month_start_ts(), now),
            year: data.count_in_range(year_start_ts(), now),
            total: data.total(),
        }
    }

    fn get_specific_key_stats(&self) -> SpecificKeyStats {
        let data = self.data.read();
        let now = current_timestamp_ms();
        
        SpecificKeyStats {
            space: SpecificKeyTimeframes::new(&data, SpecificKey::Space, now),
            backspace: SpecificKeyTimeframes::new(&data, SpecificKey::Backspace, now),
            enter: SpecificKeyTimeframes::new(&data, SpecificKey::Enter, now),
            escape: SpecificKeyTimeframes::new(&data, SpecificKey::Escape, now),
        }
    }

    fn cleanup_old_data(&self) {
        self.data.write().cleanup();
    }
}

static KEY_TRACKER: LazyLock<Arc<KeyPressTracker>> = LazyLock::new(|| Arc::new(KeyPressTracker::new()));

// Global channel for keypress events (crossbeam supports multi-producer, multi-consumer)
static KEY_TX: LazyLock<StdRwLock<Option<Sender<(u64, Option<SpecificKey>)>>>> = LazyLock::new(|| StdRwLock::new(None));
static KEY_RX: LazyLock<StdRwLock<Option<Receiver<(u64, Option<SpecificKey>)>>>> = LazyLock::new(|| StdRwLock::new(None));

// Debug counters
static KEY_COUNT: AtomicU64 = AtomicU64::new(0);
static LISTENER_STARTED: AtomicBool = AtomicBool::new(false);
static LISTENER_ERROR: StdRwLock<Option<String>> = StdRwLock::new(None);
static LAST_KEY_TIME: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    use tauri::Manager;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn get_keypress_stats() -> KeyPressStats {
    KEY_TRACKER.get_stats()
}

#[tauri::command]
fn get_specific_key_stats() -> SpecificKeyStats {
    KEY_TRACKER.get_specific_key_stats()
}

#[tauri::command]
fn reset_keypress_data(app: tauri::AppHandle) {
    *KEY_TRACKER.data.write() = KeyPressData::default();
    KEY_TRACKER.save_to_store(&app);
}

#[derive(Debug, Serialize, Deserialize)]
struct MacPermissionStatus {
    accessibility: bool,
    input_monitoring: bool,
    // Both are required for global key listening on macOS.
    trusted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct KeyDebugInfo {
    key_count: u64,
    listener_started: bool,
    listener_error: Option<String>,
    last_key_time: u64,
    channel_tx_exists: bool,
    channel_rx_exists: bool,
    timestamps_count: usize,
    space_count: usize,
    backspace_count: usize,
    enter_count: usize,
    escape_count: usize,
    // None on non-macOS platforms
    mac_permissions: Option<MacPermissionStatus>,
}

#[tauri::command]
fn get_keypress_debug() -> KeyDebugInfo {
    let (accessibility, input_monitoring) =
        (macos_accessibility_trusted(), macos_input_monitoring_granted());
    let mac_permissions = if cfg!(target_os = "macos") {
        Some(MacPermissionStatus {
            accessibility,
            input_monitoring,
            trusted: accessibility && input_monitoring,
        })
    } else {
        None
    };
    KeyDebugInfo {
        key_count: KEY_COUNT.load(Ordering::Relaxed),
        listener_started: LISTENER_STARTED.load(Ordering::Relaxed),
        listener_error: LISTENER_ERROR.read().unwrap().clone(),
        last_key_time: LAST_KEY_TIME.load(Ordering::Relaxed),
        channel_tx_exists: KEY_TX.read().unwrap().is_some(),
        channel_rx_exists: KEY_RX.read().unwrap().is_some(),
        timestamps_count: KEY_TRACKER.data.read().timestamps.len(),
        space_count: KEY_TRACKER.data.read().space_timestamps.len(),
        backspace_count: KEY_TRACKER.data.read().backspace_timestamps.len(),
        enter_count: KEY_TRACKER.data.read().enter_timestamps.len(),
        escape_count: KEY_TRACKER.data.read().escape_timestamps.len(),
        mac_permissions,
    }
}

// Dock icon visibility (macOS). Hiding the icon switches the app to the
// .Accessory activation policy so it lives only in the menu-bar tray.
fn load_dock_icon_visible(app: &AppHandle) -> bool {
    app.store(STORE_PATH)
        .ok()
        .and_then(|store| store.get(DOCK_ICON_VISIBLE_KEY))
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

fn apply_dock_icon(app: &AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        let policy = if visible {
            tauri::ActivationPolicy::Regular
        } else {
            tauri::ActivationPolicy::Accessory
        };
        if let Err(err) = app.set_activation_policy(policy) {
            eprintln!("[DockIcon] Failed to set activation policy: {err}");
        }
    }
}

#[tauri::command]
fn get_dock_icon_visible(app: AppHandle) -> bool {
    load_dock_icon_visible(&app)
}

#[tauri::command]
fn set_dock_icon_visible(app: AppHandle, visible: bool) -> Result<(), String> {
    apply_dock_icon(&app, visible);
    // Switching to the Accessory policy can hide the app's windows on
    // macOS. Make sure the main window stays visible either way.
    if !visible {
        use tauri::Manager;
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
        }
    }
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store.set(DOCK_ICON_VISIBLE_KEY, serde_json::json!(visible));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

// Opens the macOS Privacy & Security settings panes where the user can
// grant the required permissions.
#[tauri::command]
fn open_permission_settings() {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
            .spawn();
    }
}

fn start_key_listener(app: AppHandle) {
    // Warn early if macOS permissions are missing. macOS silently drops
    // keyboard events for untrusted processes (rdev reports no error), so
    // without this explicit check the failure is invisible.
    let (ax_ok, lm_ok) = (macos_accessibility_trusted(), macos_input_monitoring_granted());
    if !(ax_ok && lm_ok) {
        eprintln!("[KeyListener] WARNING: macOS permissions missing! accessibility={} input_monitoring={}", ax_ok, lm_ok);
        eprintln!("[KeyListener] WARNING: Keypress events will be silently dropped until granted.");
        eprintln!("[KeyListener] WARNING: Grant in System Settings > Privacy & Security > Accessibility and Input Monitoring.");
        eprintln!("[KeyListener] WARNING: NOTE: ad-hoc signed apps lose these grants on every rebuild - re-grant after reinstalling.");
    } else {
        eprintln!("[KeyListener] macOS permissions OK (accessibility + input monitoring)");
    }

    // Create a channel for keypress events
    let (tx, rx) = unbounded::<(u64, Option<SpecificKey>)>();
    
    *KEY_TX.write().unwrap() = Some(tx);
    *KEY_RX.write().unwrap() = Some(rx);
    
    // Spawn the rdev listener thread
    thread::spawn(move || {
        eprintln!("[KeyListener] Thread spawned, starting...");
        std::io::stderr().flush().ok();
        LISTENER_STARTED.store(true, Ordering::Relaxed);
        *LISTENER_ERROR.write().unwrap() = None;

        eprintln!("[KeyListener] Listener thread started");
        std::io::stderr().flush().ok();
        let callback = move |event: Event| {
            // Wrap entire callback in panic catch
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                // Log ALL events for debugging
                eprintln!("[KeyListener] Event received: {:?}", event.event_type);
                // Only process KeyPress events
                if let EventType::KeyPress(key) = event.event_type {
                    eprintln!("[KeyListener] KeyPress received: {:?}", key);
                    KEY_COUNT.fetch_add(1, Ordering::Relaxed);
                    let ts = current_timestamp_ms();
                    LAST_KEY_TIME.store(ts, Ordering::Relaxed);
                    let specific_key = SpecificKey::from_rdev_key(key);
                    if let Some(tx) = KEY_TX.read().unwrap().as_ref() {
                        let _ = tx.send((ts, specific_key));
                    }
                }
            }));
        };
        
        eprintln!("[KeyListener] Starting global key listener...");
        eprintln!("[KeyListener] NOTE: On macOS, this requires Accessibility permissions in System Settings > Privacy & Security > Accessibility");
        eprintln!("[KeyListener] Calling listen()...");
        std::io::stderr().flush().ok();
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            listen(callback)
        })) {
            Ok(Err(e)) => {
                eprintln!("[KeyListener] listen() returned Err: {:?}", e);
                *LISTENER_ERROR.write().unwrap() = Some(format!("Failed to start: {:?}", e));
            }
            Ok(Ok(_)) => {
                eprintln!("[KeyListener] listen() returned Ok(Ok(())) - listener running");
                eprintln!("[KeyListener] Key listener started successfully");
            }
            Err(e) => {
                eprintln!("[KeyListener] PANIC in listen(): {:?}", e);
                *LISTENER_ERROR.write().unwrap() = Some(format!("Panic: {:?}", e));
            }
        }
    });
    
    // Spawn a thread to process keypresses on the main thread via Tauri's run_on_main_thread
    let app_clone = app.clone();
    thread::spawn(move || {
        LISTENER_STARTED.store(true, Ordering::Relaxed);
        *LISTENER_ERROR.write().unwrap() = None;

        loop {
            // Check for keypresses every 100ms
            thread::sleep(Duration::from_millis(100));
            
            let mut events = Vec::new();
            if let Some(rx) = KEY_RX.read().unwrap().as_ref() {
                while let Ok(event) = rx.try_recv() {
                    events.push(event);
                }
            }
            
            if !events.is_empty() {
                let app_for_save = app_clone.clone();
                let _ = app_clone.run_on_main_thread(move || {
                    for (ts, specific_key) in events {
                        KEY_TRACKER.record_press(ts);
                        if let Some(key) = specific_key {
                            KEY_TRACKER.record_specific_key(key, ts);
                        }
                    }
                    // Save periodically (every 100 presses)
                    if KEY_TRACKER.should_save() {
                        KEY_TRACKER.save_to_store(&app_for_save);
                    }
                });
            }
        }
    });
    
    // Periodic cleanup and save (every hour)
    thread::spawn(move || {
        LISTENER_STARTED.store(true, Ordering::Relaxed);
        *LISTENER_ERROR.write().unwrap() = None;

        loop {
            thread::sleep(Duration::from_secs(60 * 60));
            let app_for_cleanup = app.clone();
            let app_for_save = app_for_cleanup.clone();
            let _ = app_for_cleanup.run_on_main_thread(move || {
                KEY_TRACKER.cleanup_old_data();
                KEY_TRACKER.save_to_store(&app_for_save);
            });
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        // Window-state must be registered via Builder::plugin (NOT in .setup()).
        // Plugins registered in .setup() miss config-defined windows: window
        // creation dispatches the plugin's on_window_ready synchronously on the
        // main thread during setup, BEFORE the .setup() closure runs, so the
        // plugin never tracks the window and saves an empty {} state.
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            show_main_window, 
            get_keypress_stats,
            get_specific_key_stats,
            reset_keypress_data,
            get_keypress_debug,
            open_permission_settings,
            get_dock_icon_visible,
            set_dock_icon_visible
        ])
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
            use tauri::tray::TrayIconBuilder;

            // Load data from store
            KEY_TRACKER.load_from_store(app.handle());

            // Apply the saved dock-icon preference before the window shows.
            if !load_dock_icon_visible(app.handle()) {
                apply_dock_icon(app.handle(), false);
            }
            
            // Start global key listener
            start_key_listener(app.handle().clone());

            // Create menu items
            let show_main = MenuItem::with_id(app, "show_main", "Open Main Window", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;

            // Create the menu
            let menu = Menu::with_items(app, &[&show_main, &separator, &quit])?;

            // Load custom tray icon
            let icon_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("icons/traytest.png");
            let icon_bytes = std::fs::read(&icon_path).expect("Failed to read tray icon");
            let tray_icon = tauri::image::Image::from_bytes(&icon_bytes).expect("Failed to load tray icon");

            // Create tray icon
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show_main" => {
                            let app_handle = app.clone();
                            let _ = app.run_on_main_thread(move || {
                                show_main_window(app_handle);
                            });
                        }
                        "quit" => {
                            // Save keypress data before quitting. Window position/size
                            // is persisted by the window-state plugin on exit.
                            KEY_TRACKER.save_to_store(&app);
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
        .on_window_event(|window, event| {
            // Clicking the window close button hides the window to the tray
            // instead of quitting. Position/size persistence is NOT handled
            // here — tauri-plugin-window-state saves it on app exit and
            // restores it on launch.
            use tauri::WindowEvent;
            if window.label() == "main" {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}