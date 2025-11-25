# SparkNova 开发说明

## 概览

- 技术栈：Tauri 2 + Vue 3 + Vite + TypeScript（前端），Rust（后端）
- 目标：通过全局快捷键 `Ctrl+Shift+S` 呼出/隐藏一个透明、无边框、置顶的小窗口；失焦后自动隐藏；前端 `Esc` 直接隐藏。

## 启动

- 开发：`npx tauri dev`（前端地址：`http://localhost:1420/`）
- 构建：`npx tauri build`

## 关键逻辑

- 快捷键事件仅处理 `Pressed`，忽略 `Released`，避免一次按键触发两次。
- 切换逻辑：可见即隐藏，否则显示并聚焦；显示后记录时间防止刚显示就隐藏。
- 失焦保护：显示后 800ms 内的失焦被忽略，确保交互稳定。
- 前端 Esc：调用后端 `hide_main_window` 命令，稳定隐藏。

## 代码结构

- 后端入口：`src-tauri/src/main.rs` → `sparknova_lib::run()`
- 窗口控制与快捷键：`src-tauri/src/lib.rs`
  - 纯函数：`should_hide_on_blur`（可单元测试）
  - 操作函数：`get_main_window`、`show_main_window`、`hide_main_window_internal`、`toggle_main_window`
  - 命令：`open_or_focus_main_window`、`hide_main_window`

## 变更说明

- 移除重复快捷键注册，仅保留一次绑定。
- 采用事件状态判断（Pressed only），不再使用时间防抖。
- 模块化重构 `lib.rs`，新增注释与流程图，提升可维护性。
- 新增针对保护逻辑的单元测试，增强可靠性。

## 推荐工具

- VS Code + Vue 官方插件 + Tauri 扩展 + rust-analyzer
