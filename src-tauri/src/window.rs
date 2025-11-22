use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri::{App, AppHandle, Emitter, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};
use anyhow::anyhow;

/// 失焦保护期（毫秒），用于避免窗口在刚显示时因焦点抖动被立即隐藏
pub const BLUR_HIDE_DELAY_MS: u64 = 800;

/// 窗口控制器状态
///
/// 描述：管理窗口最近显示时间，供失焦保护判断使用
/// 字段：
/// - `last_show_time` 最近一次显示时间戳
pub struct WindowController {
    pub last_show_time: Arc<Mutex<Instant>>,
}

impl WindowController {
    /// 创建控制器实例
    ///
    /// 返回：`WindowController`
    /// 示例：`app.manage(WindowController::new())`
    pub fn new() -> Self {
        Self {
            last_show_time: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

/// 获取主窗口
///
/// 参数：`app` 应用句柄
/// 返回：主窗口或 `None`
/// 示例：`if let Some(w) = get_main_window(&app) { ... }`
pub fn get_main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("main")
}

/// 标记“窗口已显示”的时间戳
///
/// 参数：`ctrl` 控制器状态
/// 返回：无
fn mark_shown_now(ctrl: &WindowController) {
    *ctrl.last_show_time.lock().unwrap() = Instant::now();
}

/// 失焦是否需要隐藏（纯函数）
///
/// 参数：
/// - `last_show` 最近一次显示时间
/// - `now` 当前时间
/// - `delay_ms` 保护期毫秒
/// 返回：是否应该隐藏
/// 示例：`should_hide_on_blur(last, Instant::now(), 800)`
pub fn should_hide_on_blur(last_show: Instant, now: Instant, delay_ms: u64) -> bool {
    now.duration_since(last_show) > Duration::from_millis(delay_ms)
}

/// 显示并聚焦主窗口
///
/// 参数：`app` 应用句柄、`ctrl` 控制器状态
/// 返回：是否成功
/// 示例：`show_main_window(&app, &ctrl)`
pub fn show_main_window(app: &AppHandle, ctrl: &WindowController) -> bool {
    if let Some(window) = get_main_window(app) {
        let _ = window.show();
        mark_shown_now(ctrl);
        let _ = window.set_focus();
        let _ = window.center();
        true
    } else {
        false
    }
}

/// 隐藏主窗口
///
/// 参数：`app` 应用句柄
/// 返回：是否成功
/// 示例：`hide_main_window_internal(&app)`
pub fn hide_main_window_internal(app: &AppHandle) -> bool {
    if let Some(window) = get_main_window(app) {
        let _ = window.hide();
        true
    } else {
        false
    }
}

/// 切换主窗口显隐
///
/// 参数：`app` 应用句柄、`ctrl` 控制器状态
/// 返回：无
/// 示例：`toggle_main_window(&app, &ctrl)`
pub fn toggle_main_window(app: &AppHandle, ctrl: &WindowController) {
    if let Some(window) = get_main_window(app) {
        let is_visible = window.is_visible().unwrap_or(false);
        if is_visible {
            let _ = window.hide();
            println!("Hiding window");
        } else {
            let _ = show_main_window(app, ctrl);
            let _ = app.emit("spark_focus_input", "");
            println!("Showing and focusing window");
        }
    }
}

/// 初始化窗口事件（初始隐藏与失焦保护）
///
/// 参数：`app_handle` 应用句柄、`ctrl` 控制器状态
/// 返回：无
/// 示例：`init_window_events(&app_handle, &ctrl)`
///
/// 流程图：
/// ```text
/// [setup]
///   -> hide main initially
///   -> on Focused(false):
///        if now - last_show_time > BLUR_HIDE_DELAY_MS => hide
///        else => ignore
/// ```
pub fn init_window_events(app_handle: &AppHandle, ctrl: &WindowController) {
    if let Some(window) = get_main_window(app_handle) {
        let _ = window.hide();
        let window_clone = window.clone();
        let last_lock = ctrl.last_show_time.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                if !focused {
                    let last = *last_lock.lock().unwrap();
                    if should_hide_on_blur(last, Instant::now(), BLUR_HIDE_DELAY_MS) {
                        println!("Window lost focus, hiding...");
                        let _ = window_clone.hide();
                    } else {
                        println!("Window just shown, ignoring focus lost event (protected)");
                    }
                }
            }
        });
    }
}

/// 集中注册快捷键（仅处理按下事件）
///
/// 参数：`app` 应用实例
/// 返回：`tauri::Result<()>`
/// 示例：`register_global_shortcuts(app)?;`
pub fn register_global_shortcuts(app: &App) -> tauri::Result<()> {
    let builder = tauri_plugin_global_shortcut::Builder::new()
        .with_shortcuts(["ctrl+shift+s"]).map_err(|e| anyhow!(e))?;
    app.handle().plugin(
        builder
            .with_handler(|app, shortcut, event| {
                if event.state == ShortcutState::Pressed
                    && shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyS)
                {
                    let ctrl = app.state::<WindowController>();
                    toggle_main_window(&app, ctrl.inner());
                }
            })
            .build(),
    )
}
