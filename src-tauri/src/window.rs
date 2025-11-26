#![allow(dead_code)]
#![allow(unused_imports)]
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tauri::{App, AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_global_shortcut::ShortcutState;
use tracing::{debug, info, warn};

/// 失焦后自动隐藏的延迟时间（毫秒）
const BLUR_HIDE_DELAY_MS: u64 = 3000;

/// 窗口控制器状态
///
/// 描述：管理窗口最近显示时间，供失焦保护判断使用
/// 字段：
/// - `last_show_time` 最近一次显示时间戳
pub struct WindowController {
    last_show_time: Mutex<Instant>,
    last_monitor_size: Mutex<Option<(u32, u32)>>,
    current_focused: Mutex<bool>,
}

impl Clone for WindowController {
    fn clone(&self) -> Self {
        let last_time = *self.last_show_time.lock().unwrap();
        let last_size = *self.last_monitor_size.lock().unwrap();
        let focused = *self.current_focused.lock().unwrap();
        Self {
            last_show_time: Mutex::new(last_time),
            last_monitor_size: Mutex::new(last_size),
            current_focused: Mutex::new(focused),
        }
    }
}

impl WindowController {
    /// 创建控制器实例
    ///
    /// 返回：`WindowController`
    /// 示例：`app.manage(WindowController::new())`
    pub fn new() -> Self {
        Self {
            last_show_time: Mutex::new(Instant::now()),
            last_monitor_size: Mutex::new(None),
            current_focused: Mutex::new(false),
        }
    }

    /// 检查是否超过失焦隐藏延迟
    pub fn should_hide_on_blur(&self) -> bool {
        self.last_show_time.lock().unwrap().elapsed() > Duration::from_millis(BLUR_HIDE_DELAY_MS)
    }

    pub fn mark_shown(&self) {
        *self.last_show_time.lock().unwrap() = Instant::now();
    }

    pub fn needs_resize(&self, monitor_size: (u32, u32)) -> bool {
        let mut last = self.last_monitor_size.lock().unwrap();
        if last.map_or(true, |s| s != monitor_size) {
            *last = Some(monitor_size);
            true
        } else {
            false
        }
    }

    pub fn set_focused(&self, focused: bool) {
        *self.current_focused.lock().unwrap() = focused;
    }

    pub fn is_focused(&self) -> bool {
        *self.current_focused.lock().unwrap()
    }

    #[cfg(test)]
    pub fn set_last_show_time_for_test(&self, instant: Instant) {
        *self.last_show_time.lock().unwrap() = instant;
    }
}

/// 获取主窗口
///
/// 参数：`app` 应用句柄
/// 返回：主窗口或 `None`
/// 示例：`if let Some(w) = get_main_window(&app) { ... }`
pub fn get_main_window(app: &AppHandle) -> Option<WebviewWindow> {
    let window = app.get_webview_window("main");
    debug!("Get main window result: {}", window.is_some());
    let windows = app.windows();
    debug!("All windows: {:?}", windows.keys());

    window
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_resize_changes_on_monitor_size_change() {
        let ctrl = WindowController::new();
        assert!(ctrl.needs_resize((1920, 1080)));
        assert!(!ctrl.needs_resize((1920, 1080)));
        assert!(ctrl.needs_resize((2560, 1440)));
    }

    #[test]
    fn test_focus_state_set_and_get() {
        let ctrl = WindowController::new();
        assert!(!ctrl.is_focused());
        ctrl.set_focused(true);
        assert!(ctrl.is_focused());
        ctrl.set_focused(false);
        assert!(!ctrl.is_focused());
    }

    #[test]
    fn test_should_hide_on_blur_respects_delay() {
        let ctrl = WindowController::new();
        ctrl.mark_shown();
        assert!(!ctrl.should_hide_on_blur());
        ctrl.set_last_show_time_for_test(
            Instant::now() - Duration::from_millis(BLUR_HIDE_DELAY_MS + 50),
        );
        assert!(ctrl.should_hide_on_blur());
    }
}

/// 获取当前屏幕的分辨率
///
/// 参数：`window` 窗口实例
/// 返回：(宽度, 高度) 或 `None`
/// 示例：`if let Some((width, height)) = get_screen_resolution(&window) { ... }`
pub fn get_screen_resolution(window: &WebviewWindow) -> Option<(u32, u32)> {
    if let Some(monitor) = window.current_monitor().ok()? {
        let size = monitor.size();
        debug!("Current screen resolution: {}x{}", size.width, size.height);
        Some((size.width, size.height))
    } else {
        warn!("Failed to get current monitor");
        None
    }
}

/// 根据屏幕分辨率计算合适的窗口大小
///
/// 参数：`screen_width` 屏幕宽度、`screen_height` 屏幕高度
/// 返回：(窗口宽度, 窗口高度)
/// 示例：`let (window_width, window_height) = calculate_window_size(1920, 1080);`
pub fn calculate_window_size(screen_width: u32, screen_height: u32) -> (u32, u32) {
    // 计算窗口宽度为屏幕宽度的75%，但不小于400px，不大于800px
    let window_width = (screen_width as f64 * 0.75).clamp(400.0, 960.0) as u32;
    // 保持窗口高度为80px（根据当前设计）
    let window_height = 80;

    debug!(
        "Calculated window size: {}x{} for screen {}x{}",
        window_width, window_height, screen_width, screen_height
    );

    (window_width, window_height)
}

/// 显示并聚焦主窗口
///
/// 参数：`app` 应用句柄、`ctrl` 控制器状态
/// 返回：是否成功
/// 示例：`show_main_window(&app, &ctrl)`
pub fn show_main_window(app: &AppHandle) -> bool {
    if let Some(window) = get_main_window(app) {
        info!("Showing main window");

        // 根据屏幕分辨率动态调整窗口大小
        // 已简化：不再按屏幕变化调整大小，保持配置尺寸

        let result = window.show();
        debug!("Show result: {:?}", result);

        let center_result = window.center();
        debug!("Center result: {:?}", center_result);

        // 确保窗口不处于最小化状态，便于系统授予焦点
        let unminimize_result = window.unminimize();
        debug!("Unminimize result: {:?}", unminimize_result);

        let focus_result = window.set_focus();
        debug!("Focus result: {:?}", focus_result);

        // 延迟二次设焦，提升跨平台（尤其 Windows 透明窗口/跳过任务栏）场景的成功率
        {
            let window_clone = window.clone();
            tauri::async_runtime::spawn_blocking(move || {
                std::thread::sleep(Duration::from_millis(50));
                let _ = window_clone.set_focus();
            });
        }

        // 向前端发送聚焦输入事件
        if let Err(e) = tauri::Emitter::emit_to(app, "main", "focus-search-input", ()) {
            warn!("Emit focus-search-input failed: {:?}", e);
        } else {
            debug!("Emitted focus-search-input to main window");
        }

        // 简化：不再记录显示时间

        // 检查窗口是否可见
        let is_visible = window.is_visible().unwrap_or(false);
        info!("Window visible after show: {}", is_visible);

        let is_focused = window.is_focused().unwrap_or(false);
        info!("Window focused after show: {}", is_focused);

        is_visible && is_focused
    } else {
        warn!("Main window not found");
        false
    }
}

/// 隐藏主窗口
///
/// 参数：`app` 应用句柄
/// 返回：是否成功
/// 示例：`hide_main_window_internal(&app)`
pub fn hide_main_window(app: &AppHandle) -> bool {
    if let Some(window) = get_main_window(app) {
        info!("Hiding main window");
        let _ = window.hide();
        true
    } else {
        warn!("Main window not found");
        false
    }
}

/// 切换主窗口显隐
///
/// 参数：`app` 应用句柄、`ctrl` 控制器状态
/// 返回：无
/// 示例：`toggle_main_window(&app, &ctrl)`
pub fn toggle_main_window(app: &AppHandle) {
    debug!("toggle_main_window called");

    let window = get_main_window(app);
    if let Some(window) = window {
        let is_visible_result = window.is_visible();
        debug!("is_visible result: {:?}", is_visible_result);

        let is_visible = is_visible_result.unwrap_or(false);
        info!("Toggle window, current visible: {}", is_visible);

        if is_visible {
            let _ = hide_main_window(app);
        } else {
            let result = show_main_window(app);
            info!("show_main_window result: {}", result);
        }
    } else {
        warn!("Main window not found for toggle");
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
///   -> set initial window size based on screen resolution
///   -> on Focused(false):
///        if now - last_show_time > BLUR_HIDE_DELAY_MS => hide
///        else => ignore
/// ```
pub fn init_window_events(app_handle: &AppHandle, _ctrl: State<WindowController>) {
    let app = app_handle.clone();
    // 使用共享状态，避免深拷贝造成状态不一致

    // 初始隐藏主窗口并设置初始大小
    if let Some(window) = get_main_window(&app_handle) {
        info!("Initializing window events, hiding main window initially");

        // 根据屏幕分辨率设置初始窗口大小
        if let Some((screen_width, screen_height)) = get_screen_resolution(&window) {
            let ctrl_state = app.state::<WindowController>();
            if ctrl_state
                .inner()
                .needs_resize((screen_width, screen_height))
            {
                let (window_width, window_height) =
                    calculate_window_size(screen_width, screen_height);
                let resize_result = window.set_size(tauri::PhysicalSize {
                    width: window_width,
                    height: window_height,
                });
                debug!("Initial resize result: {:?}", resize_result);
            }
        }

        let _ = window.hide();

        // 在闭包外克隆窗口
        let window_clone = window.clone();

        // 监听窗口事件
        window.on_window_event(move |event| {
            match event {
                tauri::WindowEvent::Focused(focused) => {
                    let ctrl_state = app.state::<WindowController>();
                    ctrl_state.inner().set_focused(*focused);
                    if !*focused {
                        info!("Window lost focus");
                        if window_clone.is_visible().unwrap_or(false) {
                            if ctrl_state.inner().should_hide_on_blur() {
                                info!("Blur delay exceeded, hiding window");
                                hide_main_window(&app);
                            } else {
                                debug!("Within blur protection period, ignoring hide");
                            }
                        }
                    } else {
                        // 当窗口获得焦点时，根据当前屏幕分辨率调整窗口大小
                        info!("Window focused, checking screen resolution");
                        let window_clone2 = window_clone.clone();

                        if let Some((screen_width, screen_height)) =
                            get_screen_resolution(&window_clone2)
                        {
                            let ctrl_state = app.state::<WindowController>();
                            if ctrl_state
                                .inner()
                                .needs_resize((screen_width, screen_height))
                            {
                                let (window_width, window_height) =
                                    calculate_window_size(screen_width, screen_height);
                                let resize_result = window_clone2.set_size(tauri::PhysicalSize {
                                    width: window_width,
                                    height: window_height,
                                });
                                debug!("Resize on focus result: {:?}", resize_result);
                            }
                        }

                        // 再次发送聚焦输入事件，适配平台聚焦时序
                        if let Err(e) =
                            tauri::Emitter::emit_to(&app, "main", "focus-search-input", ())
                        {
                            warn!("Emit focus-search-input on focus failed: {:?}", e);
                        } else {
                            debug!("Emitted focus-search-input on focus to main window");
                        }
                    }
                }
                tauri::WindowEvent::Moved(_) => {
                    // 当窗口移动时，根据当前屏幕分辨率调整窗口大小
                    debug!("Window moved, checking screen resolution");
                    let window_clone2 = window_clone.clone();

                    if let Some((screen_width, screen_height)) =
                        get_screen_resolution(&window_clone2)
                    {
                        let ctrl_state = app.state::<WindowController>();
                        if ctrl_state
                            .inner()
                            .needs_resize((screen_width, screen_height))
                        {
                            let (window_width, window_height) =
                                calculate_window_size(screen_width, screen_height);
                            let resize_result = window_clone2.set_size(tauri::PhysicalSize {
                                width: window_width,
                                height: window_height,
                            });
                            debug!("Resize on move result: {:?}", resize_result);
                        }
                    }
                }
                _ => {}
            }
        });
    } else {
        warn!("Main window not found for event initialization");
    }
}

/// 集中注册快捷键（仅处理按下事件）
///
/// 参数：`app` 应用实例
/// 返回：`tauri::Result<()>`
/// 示例：`register_global_shortcuts(app)?;`
static SHORTCUTS_REGISTERED: OnceLock<()> = OnceLock::new();

pub fn register_global_shortcuts(app: &App) -> tauri::Result<()> {
    if SHORTCUTS_REGISTERED.get().is_some() {
        info!("Global shortcuts already registered, skipping");
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    let shortcuts = ["cmd+shift+s"];
    #[cfg(not(target_os = "macos"))]
    let shortcuts = ["ctrl+shift+s"];

    info!("Registering global shortcuts: {:?}", shortcuts);

    // 创建Builder
    let builder = tauri_plugin_global_shortcut::Builder::new();

    // 处理with_shortcuts的Result
    match builder.with_shortcuts(shortcuts) {
        Ok(mut updated_builder) => {
            // 设置事件处理器
            updated_builder = updated_builder.with_handler(|app, shortcut, event| {
                debug!("Shortcut event: {}: {:?}", shortcut, event);

                // 只处理按下事件
                if event.state == ShortcutState::Pressed {
                    info!("Shortcut pressed: {}", shortcut);

                    // 调用toggle_main_window函数（已简化）
                    toggle_main_window(&app);
                }
            });

            // 构建插件（build()方法不返回Result）
            let plugin = updated_builder.build();

            // 注册插件
            match app.handle().plugin(plugin) {
                Ok(_) => {
                    let _ = SHORTCUTS_REGISTERED.set(());
                    info!("All global shortcuts registered successfully")
                }
                Err(e) => warn!("Failed to register plugin: {:?}", e),
            };
        }
        Err(e) => {
            warn!("Failed to set shortcuts: {:?}", e);
        }
    }

    Ok(())
}
