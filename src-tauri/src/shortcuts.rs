#![allow(dead_code)]
#![allow(unused_imports)]
use std::sync::OnceLock;
use tauri::{App, AppHandle};
use tauri_plugin_global_shortcut::ShortcutState;
use tracing::{debug, info, warn};

// 全局快捷键注册状态
static SHORTCUTS_REGISTERED: OnceLock<()> = OnceLock::new();

/// 注册全局快捷键
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

                if event.state == ShortcutState::Pressed {
                    info!("快捷键被按下: {}", shortcut);
                    crate::window::toggle_main_window(&app);
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
