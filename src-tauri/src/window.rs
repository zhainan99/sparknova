#![allow(dead_code)]
#![allow(unused_imports)]
use std::sync::Mutex;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tauri::{App, AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_global_shortcut::ShortcutState;
use tracing::{debug, info, warn};

// 常量定义
// 失焦后自动隐藏的延迟时间（毫秒）
const BLUR_HIDE_DELAY_MS: u64 = 3000;
// 窗口宽度占屏幕宽度的比例
const WINDOW_WIDTH_RATIO: f64 = 0.75;
// 窗口最小宽度
const MIN_WINDOW_WIDTH: f64 = 400.0;
// 窗口最大宽度
const MAX_WINDOW_WIDTH: f64 = 960.0;
// 窗口高度（固定值）
const WINDOW_HEIGHT: u32 = 80;

// 窗口控制器状态
pub struct WindowController {
    last_show_time: Mutex<Instant>,               // 最近一次显示时间戳
    last_monitor_size: Mutex<Option<(u32, u32)>>, // 上次屏幕分辨率
    current_focused: Mutex<bool>,                 // 当前是否聚焦
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
    /// 初始化最近显示时间为当前时间，屏幕分辨率为 None，聚焦状态为 false
    pub fn new() -> Self {
        Self {
            last_show_time: Mutex::new(Instant::now()),
            last_monitor_size: Mutex::new(None),
            current_focused: Mutex::new(false),
        }
    }

    /// 检查是否超过失焦隐藏延迟
    ///
    /// 若当前聚焦状态为 false 且距离最近显示时间超过 BLUR_HIDE_DELAY_MS 毫秒，则返回 true
    pub fn should_hide_on_blur(&self) -> bool {
        self.last_show_time.lock().unwrap().elapsed() > Duration::from_millis(BLUR_HIDE_DELAY_MS)
    }

    /// 标记窗口已显示
    pub fn mark_shown(&self) {
        *self.last_show_time.lock().unwrap() = Instant::now();
    }

    /// 检查是否需要调整窗口大小
    pub fn needs_resize(&self, monitor_size: (u32, u32)) -> bool {
        let mut last = self.last_monitor_size.lock().unwrap();
        if last.map_or(true, |s| s != monitor_size) {
            *last = Some(monitor_size);
            true
        } else {
            false
        }
    }

    /// 设置窗口聚焦状态
    pub fn set_focused(&self, focused: bool) {
        *self.current_focused.lock().unwrap() = focused;
    }

    /// 获取窗口聚焦状态
    pub fn is_focused(&self) -> bool {
        *self.current_focused.lock().unwrap()
    }

    #[cfg(test)]
    pub fn set_last_show_time_for_test(&self, instant: Instant) {
        *self.last_show_time.lock().unwrap() = instant;
    }
}

// 主窗口获取
pub fn get_main_window(app: &AppHandle) -> Option<WebviewWindow> {
    let window = app.get_webview_window("main");
    debug!("获取主窗口结果: {}", window.is_some());
    let windows = app.windows();
    debug!("所有窗口: {:?}", windows.keys());

    window
}

// 窗口大小管理
pub fn get_screen_resolution(window: &WebviewWindow) -> Option<(u32, u32)> {
    if let Some(monitor) = window.current_monitor().ok()? {
        let size = monitor.size();
        debug!("当前屏幕分辨率: {}x{}", size.width, size.height);
        Some((size.width, size.height))
    } else {
        warn!("获取当前显示器失败");
        None
    }
}

// 根据屏幕分辨率计算合适的窗口大小
pub fn calculate_window_size(screen_width: u32, screen_height: u32) -> (u32, u32) {
    let window_width =
        (screen_width as f64 * WINDOW_WIDTH_RATIO).clamp(MIN_WINDOW_WIDTH, MAX_WINDOW_WIDTH) as u32;
    let window_height = WINDOW_HEIGHT;

    debug!(
        "计算窗口大小: {}x{} (屏幕: {}x{})
",
        window_width, window_height, screen_width, screen_height
    );

    (window_width, window_height)
}

// 窗口显示/隐藏/切换
pub fn show_main_window(app: &AppHandle) -> bool {
    if let Some(window) = get_main_window(app) {
        info!("显示主窗口");

        let result = window.show();
        debug!("显示结果: {:?}", result);

        let center_result = window.center();
        debug!("居中结果: {:?}", center_result);

        let unminimize_result = window.unminimize();
        debug!("取消最小化结果: {:?}", unminimize_result);

        let focus_result = window.set_focus();
        debug!("聚焦结果: {:?}", focus_result);

        // 向前端发送聚焦输入事件
        if let Err(e) = tauri::Emitter::emit_to(app, "main", "activate-input", ()) {
            warn!("发送聚焦输入事件失败: {:?}", e);
        }

        let is_visible = window.is_visible().unwrap_or(false);
        info!("显示后窗口可见: {}", is_visible);

        let is_focused = window.is_focused().unwrap_or(false);
        info!("显示后窗口聚焦: {}", is_focused);

        is_visible && is_focused
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
        if let Err(e) = tauri::Emitter::emit_to(app, "main", "window-hidden", ()) {
            warn!("发送窗口隐藏事件失败: {:?}", e);
        }

        let is_visible = window.is_visible().unwrap_or(false);
        info!("隐藏后窗口可见: {}", !is_visible);

        let is_focused = window.is_focused().unwrap_or(false);
        info!("隐藏后窗口聚焦: {}", !is_focused);

        !is_visible && !is_focused
    } else {
        warn!("未找到主窗口");
        false
    }
}

// 切换主窗口显隐
pub fn toggle_main_window(app: &AppHandle) {
    debug!("切换主窗口显隐");

    let window = get_main_window(app);
    if let Some(window) = window {
        let is_visible_result = window.is_visible();
        debug!("窗口可见性结果: {:?}", is_visible_result);

        let is_visible = is_visible_result.unwrap_or(false);
        info!("切换窗口，当前可见: {}", is_visible);

        if is_visible {
            let _ = hide_main_window(app);
        } else {
            let result = show_main_window(app);
            info!("显示主窗口结果: {}", result);
        }
    } else {
        warn!("切换窗口时未找到主窗口");
    }
}

// 窗口事件处理辅助函数
fn resize_window_if_needed(window: &WebviewWindow, app: &AppHandle) {
    if let Some((screen_width, screen_height)) = get_screen_resolution(window) {
        let ctrl_state = app.state::<WindowController>();
        if ctrl_state
            .inner()
            .needs_resize((screen_width, screen_height))
        {
            let (window_width, window_height) = calculate_window_size(screen_width, screen_height);
            let resize_result = window.set_size(tauri::PhysicalSize {
                width: window_width,
                height: window_height,
            });
            debug!("调整大小结果: {:?}", resize_result);
        }
    }
}

// 处理窗口失去焦点事件
fn handle_focus_lost(window: &WebviewWindow, app: &AppHandle) {
    info!("窗口失去焦点");
    if window.is_visible().unwrap_or(false) {
        let ctrl_state = app.state::<WindowController>();
        if ctrl_state.inner().should_hide_on_blur() {
            info!("超过失焦延迟，隐藏窗口");
            hide_main_window(app);
        } else {
            debug!("处于保护期内，忽略隐藏");
        }
    }
}

// 处理窗口获得焦点事件
fn handle_focus_gained(window: &WebviewWindow, app: &AppHandle) {
    info!("窗口获得焦点，检查屏幕分辨率");
    resize_window_if_needed(window, app);

    // 再次发送聚焦输入事件，适配平台聚焦时序
    if let Err(e) = tauri::Emitter::emit_to(app, "main", "focus-search-input", ()) {
        warn!("聚焦时发送输入事件失败: {:?}", e);
    } else {
        debug!("聚焦时已发送输入事件到主窗口");
    }
}

// 处理窗口移动事件
fn handle_window_moved(window: &WebviewWindow, app: &AppHandle) {
    debug!("窗口移动，检查屏幕分辨率");
    resize_window_if_needed(window, app);
}
