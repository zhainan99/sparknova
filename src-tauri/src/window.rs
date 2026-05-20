#![allow(dead_code)]
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tauri::{App, AppHandle, Manager, WebviewWindow};
use tracing::{debug, info, warn};

// 常量定义
/// 失焦后自动隐藏的延迟时间（毫秒）
const BLUR_HIDE_DELAY_MS: u64 = 3000;
/// 窗口宽度占屏幕宽度的比例
const WINDOW_WIDTH_RATIO: f64 = 0.75;
/// 窗口最小宽度
const MIN_WINDOW_WIDTH: f64 = 400.0;
/// 窗口最大宽度
const MAX_WINDOW_WIDTH: f64 = 960.0;
/// 窗口高度（固定值）
const WINDOW_HEIGHT: u32 = 80;

/// 窗口控制器状态（实现 Clone，手动复制内部值避免锁竞争）
#[derive(Clone)]
pub struct WindowController {
    last_show_time: Instant,
    last_monitor_size: Option<(u32, u32)>,
    current_focused: bool,
}

impl WindowController {
    /// 创建控制器实例
    pub fn new() -> Self {
        Self {
            last_show_time: Instant::now(),
            last_monitor_size: None,
            current_focused: false,
        }
    }

    /// 检查是否超过失焦隐藏延迟
    pub fn should_hide_on_blur(&self) -> bool {
        self.last_show_time.elapsed() > Duration::from_millis(BLUR_HIDE_DELAY_MS)
    }

    /// 标记窗口已显示
    pub fn mark_shown(&mut self) {
        self.last_show_time = Instant::now();
    }

    /// 检查是否需要调整窗口大小
    pub fn needs_resize(&mut self, monitor_size: (u32, u32)) -> bool {
        if self.last_monitor_size.map_or(true, |s| s != monitor_size) {
            self.last_monitor_size = Some(monitor_size);
            true
        } else {
            false
        }
    }

    /// 设置窗口聚焦状态
    pub fn set_focused(&mut self, focused: bool) {
        self.current_focused = focused;
    }

    /// 获取窗口聚焦状态
    pub fn is_focused(&self) -> bool {
        self.current_focused
    }

    #[cfg(test)]
    pub fn set_last_show_time_for_test(&mut self, instant: Instant) {
        self.last_show_time = instant;
    }
}

/// 全局状态管理器（持有可变窗口控制器）
pub struct AppState {
    pub controller: Mutex<WindowController>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            controller: Mutex::new(WindowController::new()),
        }
    }
}

// 窗口获取
fn get_main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("main")
}

// 窗口大小管理
fn get_screen_resolution(window: &WebviewWindow) -> Option<(u32, u32)> {
    if let Some(monitor) = window.current_monitor().ok()? {
        let size = monitor.size();
        Some((size.width, size.height))
    } else {
        warn!("获取当前显示器失败");
        None
    }
}

// 根据屏幕分辨率计算合适的窗口大小
fn calculate_window_size(screen_width: u32) -> (u32, u32) {
    let window_width = (screen_width as f64 * WINDOW_WIDTH_RATIO)
        .clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH) as u32;
    let window_height = WINDOW_HEIGHT;
    (window_width, window_height)
}

// 窗口显示/隐藏/切换
pub fn show_main_window(app: &AppHandle) -> bool {
    if let Some(window) = get_main_window(app) {
        info!("显示主窗口");

        let _ = window.show();
        let _ = window.center();
        let _ = window.unminimize();
        let _ = window.set_focus();

        // 向前端发送聚焦输入事件
        let _ = tauri::Emitter::emit_to(app, "main", "activate-input", ());

        // 更新控制器状态
        if let Some(state) = app.try_state::<AppState>() {
            let mut ctrl = state.controller.lock().unwrap();
            ctrl.mark_shown();
        }

        debug!("显示主窗口成功");
        true
    } else {
        warn!("未找到主窗口");
        false
    }
}

// 隐藏主窗口
pub fn hide_main_window(app: &AppHandle) -> bool {
    if let Some(window) = get_main_window(app) {
        info!("隐藏主窗口");
        let _ = window.hide();

        // 向前端发送窗口隐藏事件
        let _ = tauri::Emitter::emit_to(app, "main", "window-hidden", ());

        debug!("隐藏主窗口成功");
        true
    } else {
        warn!("未找到主窗口");
        false
    }
}

// 切换主窗口显隐
pub fn toggle_main_window(app: &AppHandle) {
    debug!("切换主窗口显隐");

    if let Some(window) = get_main_window(app) {
        let is_visible = window.is_visible().unwrap_or(false);
        info!("切换窗口，当前可见: {}", is_visible);

        if is_visible {
            hide_main_window(app);
        } else {
            show_main_window(app);
        }
    } else {
        warn!("切换窗口时未找到主窗口");
    }
}

// 窗口事件处理
pub fn init_window_events(app: &AppHandle) {
    if let Some(window) = get_main_window(app) {
        // 初始调整窗口大小
        if let Some((screen_width, _)) = get_screen_resolution(&window) {
            if let Some(state) = app.try_state::<AppState>() {
                let mut ctrl = state.controller.lock().unwrap();
                if ctrl.needs_resize((screen_width, 0)) {
                    let (width, height) = calculate_window_size(screen_width);
                    let _ = window.set_size(tauri::PhysicalSize { width, height });
                }
            }
        }

        let app_handle = app.clone();
        let w = window.clone();

        window.on_window_event(move |event| {
            match event {
                tauri::WindowEvent::Focused(focused) => {
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let mut ctrl = state.controller.lock().unwrap();
                        ctrl.set_focused(*focused);

                        if !*focused {
                            info!("窗口失去焦点");
                            if w.is_visible().unwrap_or(false) && ctrl.should_hide_on_blur() {
                                info!("超过失焦延迟，隐藏窗口");
                                hide_main_window(&app_handle);
                            }
                        } else {
                            // 窗口获得焦点时调整大小
                            if let Some((screen_width, _)) = get_screen_resolution(&w) {
                                if ctrl.needs_resize((screen_width, 0)) {
                                    let (width, height) = calculate_window_size(screen_width);
                                    let _ = w.set_size(tauri::PhysicalSize { width, height });
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        });

        debug!("窗口事件初始化完成");
    }
}

// 注册全局快捷键
static SHORTCUTS_REGISTERED: OnceLock<()> = OnceLock::new();

pub fn register_global_shortcuts(app: &App) -> tauri::Result<()> {
    if SHORTCUTS_REGISTERED.get().is_some() {
        info!("全局快捷键已注册，跳过");
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    let shortcuts = ["cmd+shift+s"];
    #[cfg(not(target_os = "macos"))]
    let shortcuts = ["ctrl+shift+s"];

    info!("注册全局快捷键: {:?}", shortcuts);

    let builder = tauri_plugin_global_shortcut::Builder::new();

    match builder.with_shortcuts(shortcuts) {
        Ok(mut updated_builder) => {
            updated_builder = updated_builder.with_handler(|app, shortcut, event| {
                debug!("快捷键事件: {}: {:?}", shortcut, event);

                if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    info!("快捷键被按下: {}", shortcut);
                    toggle_main_window(&app);
                }
            });

            let plugin = updated_builder.build();

            match app.handle().plugin(plugin) {
                Ok(_) => {
                    let _ = SHORTCUTS_REGISTERED.set(());
                    info!("所有全局快捷键注册成功")
                }
                Err(e) => warn!("注册插件失败: {:?}", e),
            };
        }
        Err(e) => {
            warn!("设置快捷键失败: {:?}", e);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_controller_should_hide() {
        let mut ctrl = WindowController::new();
        assert!(!ctrl.should_hide_on_blur());
        ctrl.set_last_show_time_for_test(
            Instant::now() - Duration::from_millis(BLUR_HIDE_DELAY_MS + 50),
        );
        assert!(ctrl.should_hide_on_blur());
    }

    #[test]
    fn test_window_controller_clone() {
        let mut ctrl = WindowController::new();
        ctrl.set_focused(true);
        let cloned = ctrl.clone();
        assert_eq!(cloned.is_focused(), true);
    }
}