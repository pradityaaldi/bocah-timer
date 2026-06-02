use std::sync::Mutex;
use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_positioner::{Position, WindowExt};

// ---- settings sent from the popover ----
#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct Settings {
    message: String,
    color: String,
    opacity: f64,
    #[serde(rename = "loop")]
    looping: bool,
    sound: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            message: "Time's up!".into(),
            color: "#0b0b0f".into(),
            opacity: 0.78,
            looping: false,
            sound: true,
        }
    }
}

// ---- shared timer state ----
struct Inner {
    running: bool,
    remaining: u32,
    duration: u32,
    generation: u64,
    label: String,
    settings: Settings,
}

struct AppState {
    inner: Mutex<Inner>,
}

#[derive(serde::Serialize)]
struct OverlayConfig {
    message: String,
    label: String,
    color: String,
    opacity: f64,
    sound: bool,
}

#[derive(serde::Serialize)]
struct TimerSnapshot {
    running: bool,
    remaining: u32,
    label: String,
}

// ---- countdown loop, runs on a background thread ----
fn run_countdown(app: AppHandle, gen: u64) {
    loop {
        std::thread::sleep(Duration::from_secs(1));
        let state = app.state::<AppState>();
        let rem = {
            let mut g = state.inner.lock().unwrap();
            if g.generation != gen || !g.running {
                return; // cancelled / superseded
            }
            if g.remaining > 0 {
                g.remaining -= 1;
            }
            g.remaining
        };
        let _ = app.emit("tick", rem);
        if rem == 0 {
            {
                let mut g = state.inner.lock().unwrap();
                g.running = false;
            }
            let _ = app.emit("finished", ());
            show_overlay(app.clone());
            return;
        }
    }
}

fn spawn_timer(app: &AppHandle, duration: u32) {
    let state = app.state::<AppState>();
    let gen = {
        let mut g = state.inner.lock().unwrap();
        g.generation += 1;
        g.running = true;
        g.remaining = duration;
        g.duration = duration;
        g.generation
    };
    let a = app.clone();
    std::thread::spawn(move || run_countdown(a, gen));
}

#[tauri::command]
fn start_timer(
    app: AppHandle,
    state: State<AppState>,
    duration_secs: u32,
    label: String,
    settings: Settings,
) {
    {
        let mut g = state.inner.lock().unwrap();
        g.settings = settings;
        g.label = label;
    }
    spawn_timer(&app, duration_secs);
}

#[tauri::command]
fn stop_timer(app: AppHandle, state: State<AppState>) {
    {
        let mut g = state.inner.lock().unwrap();
        g.generation += 1;
        g.running = false;
        g.remaining = 0;
    }
    let _ = app.emit("stopped", ());
}

#[tauri::command]
fn dismiss_overlay(app: AppHandle, state: State<AppState>) {
    if let Some(w) = app.get_webview_window("overlay") {
        let _ = w.close();
    }
    let (looping, dur) = {
        let g = state.inner.lock().unwrap();
        (g.settings.looping, g.duration)
    };
    if looping && dur > 0 {
        spawn_timer(&app, dur);
    } else {
        let _ = app.emit("stopped", ());
    }
}

#[tauri::command]
fn get_overlay_config(state: State<AppState>) -> OverlayConfig {
    let g = state.inner.lock().unwrap();
    OverlayConfig {
        message: g.settings.message.clone(),
        label: g.label.clone(),
        color: g.settings.color.clone(),
        opacity: g.settings.opacity,
        sound: g.settings.sound,
    }
}

#[tauri::command]
fn get_state(state: State<AppState>) -> TimerSnapshot {
    let g = state.inner.lock().unwrap();
    TimerSnapshot {
        running: g.running,
        remaining: g.remaining,
        label: g.label.clone(),
    }
}

// ---- overlay window (main display only) ----
fn show_overlay(app: AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(w) = handle.get_webview_window("overlay") {
            let _ = w.show();
            let _ = w.set_focus();
            return;
        }
        let (x, y, w, h) = match handle.primary_monitor() {
            Ok(Some(m)) => {
                let scale = m.scale_factor();
                let size = m.size();
                let pos = m.position();
                (
                    pos.x as f64 / scale,
                    pos.y as f64 / scale,
                    size.width as f64 / scale,
                    size.height as f64 / scale,
                )
            }
            _ => (0.0, 0.0, 1440.0, 900.0),
        };

        let win = WebviewWindowBuilder::new(&handle, "overlay", WebviewUrl::App("overlay.html".into()))
            .title("")
            .inner_size(w, h)
            .position(x, y)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .resizable(false)
            .shadow(false)
            .skip_taskbar(true)
            .focused(true)
            .build();

        if let Ok(win) = win {
            #[cfg(target_os = "macos")]
            raise_above_everything(&win);
        }
    });
}

// Put the window above the menu bar and over every Space / fullscreen app,
// so it covers the screen without becoming a separate fullscreen Space.
#[cfg(target_os = "macos")]
fn raise_above_everything(win: &tauri::WebviewWindow) {
    use cocoa::appkit::{NSWindow, NSWindowCollectionBehavior};
    use cocoa::base::id;

    if let Ok(ptr) = win.ns_window() {
        let ns = ptr as id;
        unsafe {
            // kCGScreenSaverWindowLevel == 1000: above the menu bar and the Dock.
            ns.setLevel_(1000);
            ns.setCollectionBehavior_(
                NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
                    | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary,
            );
        }
    }
}

fn toggle_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        if w.is_visible().unwrap_or(false) {
            // Let the frontend play the exit animation, then call hide_window.
            let _ = app.emit("anim-hide", ());
        } else {
            // Anchor the popover under the tray icon, like a menu-bar dropdown.
            let _ = w.move_window(Position::TrayCenter);
            let _ = w.show();
            let _ = w.set_focus();
        }
    }
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .manage(AppState {
            inner: Mutex::new(Inner {
                running: false,
                remaining: 0,
                duration: 0,
                generation: 0,
                label: String::new(),
                settings: Settings::default(),
            }),
        })
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show", "Show / Hide", true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
                ],
            )?;

            let tray_icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;
            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => toggle_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_main(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
            // Auto-hide the popover when it loses focus (click elsewhere), like a menu-bar dropdown.
            // Ask the frontend to animate out; it calls hide_window when done.
            tauri::WindowEvent::Focused(false) => {
                if window.label() == "main" {
                    let _ = window.app_handle().emit("anim-hide", ());
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            start_timer,
            stop_timer,
            hide_window,
            dismiss_overlay,
            get_overlay_config,
            get_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
