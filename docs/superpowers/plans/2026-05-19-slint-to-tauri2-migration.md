# Nimbus Slint → Tauri 2 迁移计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 Nimbus 从 Slint UI 框架迁移到 Tauri 2，使用 Svelte + shadcn-svelte 作为前端技术栈。基于 sparknova 项目（已有 Tauri 2 + Svelte 骨架）进行迁移。

**Architecture:** sparknova 已有 Tauri 2 基础 + 窗口管理 + 热键实现。迁移重点：添加 nimbus 的 Rust 业务逻辑（domain/search/indexer/storage），重写前端 UI。

**Tech Stack:** Tauri 2, Svelte 5, shadcn-svelte, Tailwind CSS, tauri-plugin-global-shortcut, tauri-plugin-shell, tauri-plugin-dialog, tauri-plugin-clipboard-manager

---

## 项目现状

### sparknova 已有（可复用）
- ✅ `src-tauri/` - Tauri 2 项目结构
- ✅ `src-tauri/src/window.rs` - 窗口管理（居中、显隐、失焦保护）
- ✅ `src-tauri/src/lib.rs` - 热键注册 + 事件循环（使用 `tauri-plugin-global-shortcut`，热键 `Ctrl+Alt+F`）
- ✅ `src/` - Svelte 前端项目
- ✅ 全局热键 `Ctrl+Alt+F`（使用 `tauri-plugin-global-shortcut`）
- ✅ 透明窗口 + 无边框

### 需要迁移的 Nimbus 模块
| 模块 | 路径 | 说明 |
|------|------|------|
| domain | `src/domain/` | AppEntry, AppSource, Note 类型定义 |
| search | `src/search/` | 搜索引擎核心 |
| indexer | `src/indexer/` | 扫描逻辑 |
| storage | `src/storage/` | redb + SQLite 数据库 |
| config | `src/config.rs` | 配置管理 |
| paths | `src/paths.rs` | 路径工具 |

**跳过（sparknova 已实现或不需要）：**
| 模块 | 原因 |
|------|------|
| hotkey | sparknova 使用 `tauri-plugin-global-shortcut`，不复制 nimbus 的 `hotkey/` |
| tray | sparknova 使用 Tauri 内置托盘，不复制 nimbus 的 `tray.rs` |
| ui | Slint UI 文件，迁移到 Svelte 后重写 |

---

## 二、迁移阶段

### Phase 1: 添加 Nimbus Rust 业务逻辑

#### Task 1: 添加 Nimbus 核心模块到 sparknova

**Files:**
- Create: `src-tauri/src/domain/` (从 nimbus/src/domain/ 复制)
- Create: `src-tauri/src/search/` (从 nimbus/src/search/ 复制)
- Create: `src-tauri/src/indexer/` (从 nimbus/src/indexer/ 复制)
- Create: `src-tauri/src/storage/` (从 nimbus/src/storage/ 复制)
- Create: `src-tauri/src/paths.rs` (从 nimbus/src/paths.rs 复制)
- Create: `src-tauri/src/config.rs` (从 nimbus/src/config.rs 复制)
- Modify: `src-tauri/Cargo.toml` (添加 nimbus 依赖)

**Steps:**

- [ ] **Step 1: 添加 paths.rs**

从 nimbus 复制 `paths.rs` 到 sparknova 的 `src-tauri/src/` 目录。

- [ ] **Step 2: 添加 domain 模块**

从 nimbus 复制 `domain/` 目录到 sparknova。确保 `mod.rs` 中声明的子模块存在。

- [ ] **Step 3: 添加 search 模块**

从 nimbus 复制 `search/` 目录到 sparknova，包含：
- `engine.rs` - 搜索引擎核心
- `matcher.rs` - 模糊匹配
- `ranker.rs` - 频次加权排序（包含 `rerank` 函数）
- `pinyin.rs` - 拼音转换

注意：`rerank` 函数在 `ranker.rs` 中，不在 `engine.rs` 中。

- [ ] **Step 4: 添加 indexer 模块**

从 nimbus 复制 `indexer/` 目录到 sparknova。

- [ ] **Step 5: 添加 storage 模块**

从 nimbus 复制 `storage/` 目录到 sparknova。

- [ ] **Step 6: 添加 config.rs**

从 nimbus 复制 `config.rs` 到 sparknova。

- [ ] **Step 7: 更新 Cargo.toml**

添加 nimbus 需要的依赖到 `src-tauri/Cargo.toml`：
- lnk = "0.6"
- winreg = "0.52"
- redb = "2"
- icu_segmenter = { version = "2.2", features = ["lstm"] }
- pinyin = "0.10"
- rfd = "0.15"
- arboard = "3"

保留 sparknova 现有依赖：
- sqlx, rusqlite（仍有使用）
- reqwest（可能需要）
- tracing 系列

移除（如果存在）：
- supabase-rust（nimbus 不需要）

- [ ] **Step 7.5: 验证编译**

复制完模块后，运行：
```bash
cd src-tauri && cargo check
```

如果有编译错误，检查：
1. 模块间依赖是否完整
2. Send + Sync trait 是否满足（Tauri 命令在后台线程执行）
3. 路径引用是否正确

- [ ] **Step 8: 更新 lib.rs 初始化**

修改 `src-tauri/src/lib.rs` 添加业务逻辑初始化：

---

### Phase 2: Rust 命令层

#### Task 2: 实现搜索命令

**Files:**
- Create: `src-tauri/src/commands/search.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Steps:**

- [ ] **Step 1: 创建 `src-tauri/src/commands/mod.rs`**

```rust
pub mod search;
pub mod activate;
pub mod notes;
pub mod config;
```

- [ ] **Step 2: 创建 `src-tauri/src/commands/search.rs`**

从 `src/search/engine.rs` 迁移搜索逻辑。注意：SearchEngine 需要实现 `Send + Sync` 因为 Tauri 命令在后台线程执行。

```rust
use std::sync::{Arc, RwLock};
use tauri::AppHandle;

pub struct SearchState {
    engine: RwLock<Option<Arc<crate::search::SearchEngine>>>,
    frequency_cache: RwLock<std::collections::HashMap<String, u32>>,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            engine: RwLock::new(None),
            frequency_cache: RwLock::new(std::collections::HashMap::new()),
        }
    }

    pub fn init(&self, engine: Arc<crate::search::SearchEngine>, frequency_cache: std::collections::HashMap<String, u32>) {
        *self.engine.write().unwrap() = Some(engine);
        *self.frequency_cache.write().unwrap() = frequency_cache;
    }
}

#[tauri::command]
pub async fn query(app: AppHandle, q: String) -> Result<Vec<serde_json::Value>, String> {
    let state = app.state::<SearchState>();
    let engine = state.engine.read().unwrap();
    let freq_cache = state.frequency_cache.read().unwrap();

    if let Some(ref eng) = *engine {
        let hits = eng.search(&q, 8);
        let reranked = crate::search::rerank(&hits, &freq_cache);

        Ok(reranked.into_iter().map(|app| serde_json::json!({
            "name": app.name,
            "path": app.path.to_string_lossy()
        })).collect())
    } else {
        Ok(vec![])
    }
}

#[tauri::command]
pub async fn hide_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn register(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    app.manage(SearchState::new());

    // 初始化搜索引擎（在 app setup 时调用）
    let storage = Arc::new(crate::storage::Storage::open().map_err(|e| e.to_string())?);
    let engine = Arc::new(crate::search::SearchEngine::new(storage.clone()));
    let frequency_cache = std::collections::HashMap::new();

    // 获取 state 并初始化
    let state = app.state::<SearchState>();
    state.init(engine, frequency_cache);

    Ok(())
}
```

- [ ] **Step 3: 修改 `src-tauri/src/lib.rs` 添加 commands 模块**

```rust
mod commands;

pub use commands::*;
```

---

#### Task 3: 实现启动命令

**Files:**
- Create: `src-tauri/src/commands/activate.rs`

**Steps:**

- [ ] **Step 1: 创建 `src-tauri/src/commands/activate.rs`**

```rust
use std::path::PathBuf;
use std::process::Command;
use tauri::AppHandle;

#[tauri::command]
pub async fn activate(app: AppHandle, path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    // 启动进程
    let mut cmd = Command::new(&path);
    if let Some(cwd) = path.parent() {
        cmd.current_dir(cwd);
    }
    cmd.spawn().map_err(|e| e.to_string())?;

    // Layer 3 "用过即学" 逻辑：更新频次缓存
    let state = app.state::<crate::commands::search::SearchState>();
    if let Some(ref path_str) = path.to_str() {
        let mut freq_cache = state.frequency_cache.write().unwrap();
        let count = freq_cache.get(path_str).copied().unwrap_or(0);
        freq_cache.insert(path_str.to_string(), count + 1);
    }

    // 隐藏窗口
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

---

#### Task 4: 实现笔记命令

**Files:**
- Create: `src-tauri/src/commands/notes.rs`

**Steps:**

- [ ] **Step 1: 创建 `src-tauri/src/commands/notes.rs`**

```rust
use tauri::AppHandle;
use crate::domain::NoteKind;
use crate::storage::Storage;

#[tauri::command]
pub async fn list_notes(app: AppHandle) -> Result<Vec<serde_json::Value>, String> {
    let storage = app.state::<std::sync::Arc<Storage>>();
    let notes = storage.note_store().list_all().map_err(|e| e.to_string())?;
    Ok(notes.into_iter().map(|n| serde_json::json!({
        "id": n.id,
        "content": n.content,
        "kind": format!("{:?}", n.kind),
        "created_at": n.created_at,
    })).collect())
}

#[tauri::command]
pub async fn create_note(app: AppHandle, content: String, kind: String) -> Result<serde_json::Value, String> {
    let storage = app.state::<std::sync::Arc<Storage>>();
    let note_kind = match kind.as_str() {
        "Memo" => NoteKind::Memo,
        "Todo" => NoteKind::Todo,
        "Link" => NoteKind::Link,
        _ => NoteKind::Memo,
    };
    let note = storage.note_store().insert(&content, note_kind).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "id": note.id,
        "content": note.content,
        "kind": format!("{:?}", note.kind),
        "created_at": note.created_at,
    }))
}

#[tauri::command]
pub async fn delete_note(app: AppHandle, id: i64) -> Result<(), String> {
    let storage = app.state::<std::sync::Arc<Storage>>();
    storage.note_store().delete(id).map_err(|e| e.to_string())
}
```

---

#### Task 5: 实现配置命令

**Files:**
- Create: `src-tauri/src/commands/config.rs`

**Steps:**

- [ ] **Step 1: 创建 `src-tauri/src/commands/config.rs`**

从 nimbus 复制 `src/config.rs` 后，添加以下 Tauri 命令：

```rust
use tauri::AppHandle;
use crate::config::{UserConfig, THEME_DARK, THEME_LIGHT};

#[tauri::command]
pub async fn get_config(_app: AppHandle) -> Result<serde_json::Value, String> {
    let config = UserConfig::load().map_err(|e| e.to_string())?;
    Ok(serde_json::json!({
        "scan_dirs": config.scan_dirs,
        "theme_mode": config.theme_mode,
        "show_main_on_start": config.show_main_on_start,
        "show_notes_on_start": config.show_notes_on_start,
    }))
}

#[tauri::command]
pub async fn save_config(_app: AppHandle, config: serde_json::Value) -> Result<(), String> {
    let mut user_config = UserConfig::load().map_err(|e| e.to_string())?;

    if let Some(scan_dirs) = config.get("scan_dirs").and_then(|v| v.as_array()) {
        user_config.scan_dirs = scan_dirs
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    if let Some(theme) = config.get("theme_mode").and_then(|v| v.as_str()) {
        user_config.theme_mode = theme.to_string();
    }
    if let Some(show) = config.get("show_main_on_start").and_then(|v| v.as_bool()) {
        user_config.show_main_on_start = show;
    }
    if let Some(show) = config.get("show_notes_on_start").and_then(|v| v.as_bool()) {
        user_config.show_notes_on_start = show;
    }

    user_config.save().map_err(|e| e.to_string())
}
```

---

### Phase 3: 窗口和事件处理（sparknova 已有实现，跳过）

> ⚠️ sparknova 已实现 Task 6、7、8，跳过以下任务，直接复用。

#### Task 6: 实现窗口管理 ✅

**已有：** `src-tauri/src/window.rs` 完整实现

- 窗口居中
- 失焦 3 秒保护
- 屏幕分辨率自适应
- 显隐切换

#### Task 7: 实现托盘 ✅

**已有：** Tauri 内置托盘支持

#### Task 8: 实现全局热键 ✅

**已有：** `Ctrl+Alt+F` 全局热键（使用 `tauri-plugin-global-shortcut`）

---

### Phase 4: 前端 UI 完成

#### Task 9: 完成主窗口 UI

**Files:**
- Modify: `src/routes/+page.svelte`
- Modify: `src/lib/SearchInput.svelte`
- Create: `src/lib/stores/search.ts`
- Create: `src/lib/components/ResultList.svelte`

**Steps:**

- [ ] **Step 1: 添加搜索 store**

创建 `src/lib/stores/search.ts`：
```typescript
import { writable } from 'svelte/store';

interface ResultItem {
  name: string;
  path: string;
}

interface SearchState {
  query: string;
  results: ResultItem[];
  selectedIndex: number;
}

export const searchStore = writable<SearchState>({
  query: '',
  results: [],
  selectedIndex: 0
});
```

- [ ] **Step 2: 完善搜索框组件**

修改 `src/lib/SearchInput.svelte` 添加：
- 键盘导航支持（上下箭头选择）
- Enter 键激活选中项
- 调用 Rust `query` 命令获取搜索结果

- [ ] **Step 3: 创建结果列表组件**

创建 `src/lib/components/ResultList.svelte`：
- 显示搜索结果列表
- 鼠标点击激活
- 选中高亮

- [ ] **Step 4: 更新主页**

修改 `src/routes/+page.svelte`：
- 搜索框 + 结果列表组合
- 添加 debounce（100ms）

```svelte
<script lang="ts">
  import { writable } from 'svelte/store';

  type ThemeMode = 'light' | 'dark' | 'system';

  export const themeStore = writable<ThemeMode>('system');

  $effect(() => {
    document.documentElement.classList.toggle('dark', $themeStore === 'dark');
  });
</script>
```

- [ ] **Step 4: 添加样式**

参考 `ui/theme.slint` 的颜色定义，转换为 Tailwind 类。

---

#### Task 10: 实现设置窗口

**Files:**
- Create: `src/routes/settings/+page.svelte`
- Create: `src/lib/components/SettingsForm.svelte`

**Steps:**

- [ ] **Step 1: 创建设置页面**

实现：
- 主题切换（浅色/深色/跟随系统）
- 开机自启切换
- 扫描目录管理（添加/删除）
- 关于页面

- [ ] **Step 2: 创建文件夹选择器**

使用 `invoke('dialog:openDirectory')` 调用 Tauri dialog 插件。

---

#### Task 11: 实现笔记窗口

**Files:**
- Create: `src/routes/notes/+page.svelte`
- Create: `src/lib/components/NoteList.svelte`
- Create: `src/lib/components/NoteEditor.svelte`

**Steps:**

- [ ] **Step 1: 创建笔记列表组件**

实现：
- 笔记列表展示
- 搜索过滤
- 新建/删除笔记

- [ ] **Step 2: 创建笔记编辑器**

实现：
- Markdown 编辑支持
- 自动保存

---

### Phase 5: 数据迁移和测试

#### Task 12: 数据层适配

**Files:**
- Modify: `src-tauri/src/commands/search.rs`
- Modify: `src-tauri/src/commands/activate.rs`

**Steps:**

- [ ] **Step 1: 确保 Storage 在 Tauri 命令中正常工作**

检查 `src/storage/` 模块是否需要修改以支持 `Send + Sync`。

- [ ] **Step 2: 迁移频次缓存逻辑**

确认 `frequency_cache` 的更新逻辑在 Tauri 命令中正确工作。

---

#### Task 13: 集成测试

**Files:**
- 测试前端构建
- 测试热键响应
- 测试托盘菜单
- 测试搜索功能
- 测试启动应用
- 测试窗口显示/隐藏

**Steps:**

- [ ] **Step 1: 前端构建测试**

```bash
cd frontend && npm run build
```

- [ ] **Step 2: Tauri 开发模式测试**

```bash
cd src-tauri && cargo tauri dev
```

- [ ] **Step 3: 热键功能测试**

按 `Ctrl+Alt+F` 验证窗口显示/隐藏。

- [ ] **Step 4: 搜索功能测试**

输入查询验证结果返回。

- [ ] **Step 5: 应用启动测试**

选择结果验证进程启动。

---

## 三、技术风险和缓解

### 风险 1: 热键响应延迟

**问题**: Tauri WebView 冷启动比 Slint 慢 100-200ms，可能影响 ≤50ms 热键响应目标。

**缓解**: 
- 使用 Tauri 2 的 `window_visible` 特性，窗口保持隐藏而非销毁
- 前端使用懒加载，首屏只加载搜索框

### 风险 2: 多窗口状态同步

**问题**: 主窗口、设置窗口、笔记窗口需要共享状态。

**缓解**:
- 使用 Svelte stores 统一管理状态
- 通过 Tauri 事件在窗口间通信

### 风险 3: IPC 通信开销

**问题**: 每次搜索输入都触发 Rust 调用。

**缓解**:
- 前端实现 debounce（100ms）
- 使用 `invoke` 的异步版本避免阻塞 UI

---

## 四、验收标准

1. **冷启动测试**: 应用启动后 500ms 内可响应热键显示窗口
2. **热键响应**: 按 `Ctrl+Alt+F` 后 200ms 内显示窗口（考虑 WebView 开销，目标调整为 ≤200ms）
3. **搜索测试**: 输入到结果返回 ≤100ms（debounce 后的总时间）
4. **托盘测试**: 托盘图标显示，菜单项可点击
5. **主题切换**: 浅色/深色模式正确切换
6. **应用启动**: 从列表选择应用能正确启动
7. **窗口隐藏**: Escape 键或失焦能正确隐藏窗口

---

## 五、推荐开发顺序

```
1. Task 1:  添加 Nimbus Rust 业务逻辑（domain/search/indexer/storage）
2. Task 2:  实现搜索命令
3. Task 3:  实现启动命令
4. Task 4:  实现笔记命令
5. Task 5:  实现配置命令
6. Task 9:  完成主窗口 UI（搜索框 + 结果列表）
7. Task 10: 实现设置窗口
8. Task 11: 实现笔记窗口
9. Task 12: 数据层适配
10. Task 13: 集成测试
```

> 注：窗口管理(Task 6)、托盘(Task 7)、热键(Task 8) 已由 sparknova 实现，跳过。

---

## 六、技术栈（最终）

| 层面 | 技术 |
|------|------|
| 前端框架 | Svelte 5 + SvelteKit |
| UI 组件 | Tailwind CSS + 自定义组件 |
| 后端 | Tauri 2.9 + Rust |
| 插件 | tauri-plugin-global-shortcut, tauri-plugin-shell, tauri-plugin-dialog, tauri-plugin-clipboard-manager |
| 存储 | redb + SQLite (nimbus) |
| 搜索 | 自研 fuzzy search + pinyin (nimbus) |

---

**Plan complete.**