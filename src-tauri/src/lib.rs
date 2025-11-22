// src-tauri/src/lib.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{App, AppHandle, Manager};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! SparkNova greets from Rust!", name)
}

#[tauri::command]
fn open_or_focus_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // 检查窗口是否可见且有焦点
        let is_visible = window.is_visible().unwrap_or(false);
        let is_focused = window.is_focused().unwrap_or(false);
        
        println!("Window state - visible: {}, focused: {}", is_visible, is_focused);
        
        if is_visible && is_focused {
            // 窗口可见且有焦点，隐藏它
            let _ = window.hide();
            println!("Hiding window");
        } else {
            // 窗口不可见或没有焦点，显示并聚焦
            let _ = window.show();
            let _ = window.set_focus();
            let _ = window.center();
            println!("Showing and focusing window");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point())]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .invoke_handler(tauri::generate_handler![greet, open_or_focus_main_window])
        .setup(|app: &mut App| {
            let app_handle = app.handle().clone();
            
            // 用于跟踪窗口最后显示时间，防止立即隐藏
            let last_show_time = Arc::new(Mutex::new(Instant::now()));
            
            // 确保主窗口初始化时隐藏
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
                
                // 监听窗口失去焦点事件，自动隐藏
                let window_clone = window.clone();
                let last_show_time_clone = last_show_time.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(focused) = event {
                        if !focused {
                            // 检查距离上次显示是否超过500ms，避免刚显示就隐藏
                            let elapsed = last_show_time_clone.lock().unwrap().elapsed();
                            if elapsed > Duration::from_millis(500) {
                                println!("Window lost focus, hiding...");
                                let _ = window_clone.hide();
                            } else {
                                println!("Window just shown, ignoring focus lost event ({}ms)", elapsed.as_millis());
                            }
                        }
                    }
                });
            }
            
            // 注册全局快捷键并设置回调
            let app_handle_clone = app_handle.clone();
            let last_show_time_clone = last_show_time.clone();
            app_handle
                .global_shortcut()
                .on_shortcut("Ctrl+Shift+S", move |_app, _shortcut, _event| {
                    println!("Global shortcut triggered: Ctrl+Shift+S");
                    // 更新显示时间
                    *last_show_time_clone.lock().unwrap() = Instant::now();
                    open_or_focus_main_window(app_handle_clone.clone());
                })
                .unwrap();
            
            // 注册快捷键
            if let Err(e) = app.global_shortcut().register("Ctrl+Shift+S") {
                eprintln!("Failed to register shortcut: {:?}", e);
            } else {
                println!("Shortcut Ctrl+Shift+S registered successfully!");
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
