use std::time::{Duration, Instant};
use tauri::{App, AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_global_shortcut::ShortcutState;

/// 失焦后自动隐藏的延迟时间（毫秒）
const BLUR_HIDE_DELAY_MS: u64 = 3000;

/// 窗口控制器状态
///
/// 描述：管理窗口最近显示时间，供失焦保护判断使用
/// 字段：
/// - `last_show_time` 最近一次显示时间戳
pub struct WindowController {
    last_show_time: Instant,
}

impl Clone for WindowController {
    fn clone(&self) -> Self {
        Self {
            last_show_time: self.last_show_time,
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
            last_show_time: Instant::now(),
        }
    }

    /// 检查是否超过失焦隐藏延迟
    pub fn should_hide_on_blur(&self) -> bool {
        self.last_show_time.elapsed() > Duration::from_millis(BLUR_HIDE_DELAY_MS)
    }
}

/// 获取主窗口
///
/// 参数：`app` 应用句柄
/// 返回：主窗口或 `None`
/// 示例：`if let Some(w) = get_main_window(&app) { ... }`
pub fn get_main_window(app: &AppHandle) -> Option<WebviewWindow> {
    let window = app.get_webview_window("main");
    println!("[Window] Get main window result: {:?}", window.is_some());

    // 打印所有可用窗口
    let windows = app.windows();
    println!("[Window] All windows: {:?}", windows.keys());

    window
}

/// 显示并聚焦主窗口
///
/// 参数：`app` 应用句柄、`ctrl` 控制器状态
/// 返回：是否成功
/// 示例：`show_main_window(&app, &ctrl)`
pub fn show_main_window(app: &AppHandle, _ctrl: &WindowController) -> bool {
    if let Some(window) = get_main_window(app) {
        println!("[Window] Showing main window");

        let result = window.show();
        println!("[Window] Show result: {:?}", result);

        let center_result = window.center();
        println!("[Window] Center result: {:?}", center_result);

        let focus_result = window.set_focus();
        println!("[Window] Focus result: {:?}", focus_result);

        // 检查窗口是否可见
        let is_visible = window.is_visible().unwrap_or(false);
        println!("[Window] Window visible after show: {}", is_visible);

        true
    } else {
        println!("[Window] Main window not found");
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
        println!("[Window] Hiding main window");
        let _ = window.hide();
        true
    } else {
        println!("[Window] Main window not found");
        false
    }
}

/// 切换主窗口显隐
///
/// 参数：`app` 应用句柄、`ctrl` 控制器状态
/// 返回：无
/// 示例：`toggle_main_window(&app, &ctrl)`
pub fn toggle_main_window(app: &AppHandle, ctrl: &WindowController) {
    println!("[Window] toggle_main_window called");

    let window = get_main_window(app);
    if let Some(window) = window {
        let is_visible_result = window.is_visible();
        println!("[Window] is_visible result: {:?}", is_visible_result);

        let is_visible = is_visible_result.unwrap_or(false);
        println!("[Window] Toggle window, current visible: {}", is_visible);

        if is_visible {
            let _ = hide_main_window(app);
        } else {
            let result = show_main_window(app, ctrl);
            println!("[Window] show_main_window result: {}", result);
        }
    } else {
        println!("[Window] Main window not found for toggle");
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
pub fn init_window_events(app_handle: &AppHandle, ctrl: State<WindowController>) {
    let app = app_handle.clone();
    let ctrl_clone = ctrl.inner().clone();

    // 初始隐藏主窗口
    if let Some(window) = get_main_window(&app_handle) {
        println!("[Window] Initializing window events, hiding main window initially");
        let _ = window.hide();

        // 监听窗口事件
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::Focused(focused) = event {
                if !focused {
                    println!("[Window] Window lost focus");
                    // 检查是否超过失焦隐藏延迟
                    if ctrl_clone.should_hide_on_blur() {
                        println!("[Window] Blur delay exceeded, hiding window");
                        hide_main_window(&app);
                    } else {
                        println!("[Window] Within blur protection period, ignoring hide");
                    }
                }
            }
        });
    } else {
        println!("[Window] Main window not found for event initialization");
    }
}

/// 集中注册快捷键（仅处理按下事件）
///
/// 参数：`app` 应用实例
/// 返回：`tauri::Result<()>`
/// 示例：`register_global_shortcuts(app)?;`
pub fn register_global_shortcuts(app: &App) -> tauri::Result<()> {
    let shortcuts = ["ctrl+shift+s"];
    println!("[Shortcut] Registering global shortcuts: {:?}", shortcuts);

    // 创建Builder
    let builder = tauri_plugin_global_shortcut::Builder::new();

    // 处理with_shortcuts的Result
    match builder.with_shortcuts(shortcuts) {
        Ok(mut updated_builder) => {
            // 设置事件处理器
            updated_builder = updated_builder.with_handler(|app, shortcut, event| {
                println!("[Shortcut] Event received for {}: {:?}", shortcut, event);

                // 只处理按下事件
                if event.state == ShortcutState::Pressed {
                    println!("[Shortcut] Shortcut pressed: {}", shortcut);

                    // 获取WindowController状态
                    let ctrl = app.state::<WindowController>();

                    // 调用toggle_main_window函数
                    toggle_main_window(&app, ctrl.inner());
                }
            });

            // 构建插件（build()方法不返回Result）
            let plugin = updated_builder.build();

            // 注册插件
            match app.handle().plugin(plugin) {
                Ok(_) => {
                    println!("[Shortcut] All global shortcuts registered successfully")
                }
                Err(e) => eprintln!("[Shortcut] Failed to register plugin: {:?}", e),
            };
        }
        Err(e) => {
            eprintln!("[Shortcut] Failed to set shortcuts: {:?}", e);
        }
    }

    Ok(())
}
