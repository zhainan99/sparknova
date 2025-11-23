# SparkNova实现utools效果的配置建议

## 当前配置分析

你的SparkNova项目已经有了很好的Spotlight风格基础：

- ✅ 无边框透明窗口
- ✅ 置顶显示
- ✅ 隐藏任务栏
- ✅ 启动时隐藏

## utools效果的额外配置建议

### 1. 动态高度调整

```json
{
  "windows": [
    {
      "title": "SparkNova",
      "width": 600,
      "minHeight": 60,        // 最小高度
      "maxHeight": 500,       // 最大高度（utools可以显示更多搜索结果）
      "height": 60,
      "resizable": false,     // 保持宽度固定，但允许高度调整
      // ... 其他配置
    }
  ]
}
```

### 2. 改进的焦点管理

```json
{
  "windows": [
    {
      "title": "SparkNova",
      "focus": true,          // 启动时获得焦点
      "acceptFirstMouse": false,  // 避免意外激活
      "visible": false,
      // 失去焦点时自动隐藏（通过JS控制）
    }
  ]
}
```

### 3. 多窗口支持（utools的搜索结果窗口）

```json
{
  "app": {
    "windows": [
      {
        "title": "SparkNova",
        "width": 600,
        "height": 60,
        // 主搜索窗口配置
      },
      {
        "title": "SparkNova - 搜索结果",
        "width": 800,
        "height": 400,
        "center": true,
        "resizable": true,
        "decorations": false,
        "transparent": true,
        "shadow": true,       // 搜索结果窗口可以有阴影
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "focus": false        // 搜索结果窗口不自动获得焦点
      }
    ]
  }
}
```

### 4. 全局快捷键插件配置

```json
{
  "plugins": {
    "shell": {
      "all": false,
      "execute": true,
      "sidecar": true,
      "open": true
    },
    "global-shortcut": {}     // 添加全局快捷键支持
  }
}
```

### 5. 系统集成配置

```json
{
  "windows": [
    {
      "title": "SparkNova",
      "visibleOnAllWorkspaces": true,  // 在所有工作空间可用
      "skipTaskbar": true,
      // ... 其他配置
    }
  ]
}
```

### 6. 性能优化配置

```json
{
  "windows": [
    {
      "title": "SparkNova",
      "webPreferences": {
        "preload": null,
        "contextIsolation": true,
        "nodeIntegration": false,
        "webSecurity": true
      }
    }
  ]
}
```

### 7. 平台特定优化

#### Windows特定配置

```json
{
  "windows": {
    "wry": "auto",                    // WebView 2自动选择
    "theme": "auto",                  // 自动主题
    "titleBarOverlay": {
      "color": "#00000000",           // 透明标题栏
      "symbolColor": "#FFFFFF",
      "height": 0
    }
  }
}
```

#### macOS特定配置

```json
{
  "macOS": {
    "frame": false,                   // 无窗口框架
    "fullScreenEnabled": false,
    "hiddenTitle": true,              // 隐藏标题
    "titleBarStyle": "hidden",        // 隐藏标题栏
    "windowBtnEnabled": false         // 禁用窗口按钮
  }
}
```

### 8. 完整utools风格的配置文件

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "SparkNova",
  "version": "0.1.0",
  "identifier": "com.sparknova.app.v1",
  "build": {
    "beforeDevCommand": "bunx vite dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "SparkNova",
        "width": 600,
        "minHeight": 60,
        "maxHeight": 400,
        "height": 60,
        "center": true,
        "resizable": true,            // 允许调整高度
        "decorations": false,
        "transparent": true,
        "shadow": false,              // 解决边框问题
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "visibleOnAllWorkspaces": true, // utools需要这个
        "fullscreen": false,
        "maximized": false,
        "focus": true,
        "acceptFirstMouse": false,
        "resized": false
      },
      {
        "title": "SparkNova - 结果",
        "width": 800,
        "height": 500,
        "center": true,
        "resizable": true,
        "decorations": false,
        "transparent": true,
        "shadow": true,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "focus": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-eval' 'unsafe-inline';",
      "devCsp": null
    }
  },
  "plugins": {
    "shell": {
      "all": false,
      "execute": true,
      "sidecar": true,
      "open": true
    },
    "global-shortcut": {}
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "windows": {
    "wry": "auto",
    "theme": "auto",
    "titleBarOverlay": {
      "color": "#00000000",
      "symbolColor": "#FFFFFF",
      "height": 0
    }
  },
  "macOS": {
    "frame": false,
    "fullScreenEnabled": false,
    "hiddenTitle": true,
    "titleBarStyle": "hidden",
    "windowBtnEnabled": false
  }
}
```

## 重点改进说明

### 1. 动态高度调整方案

- 添加 `minHeight` 和 `maxHeight`
- 设置 `resizable: true` 允许高度调整
- utools会根据搜索结果数量动态调整高度

### 2. 全局快捷键支持

- 添加 `"global-shortcut": {}` 配置
- 这需要系统级权限，会在运行时提示用户授权

### 3. 多窗口架构

- 主搜索窗口：紧凑、快速响应
- 搜索结果窗口：详细展示、支持滚动

### 4. 性能和安全

- 配置适当的CSP策略
- 启用上下文隔离
- 禁用节点集成（提高安全性）

### 5. 跨工作空间支持

- 设置 `visibleOnAllWorkspaces: true`
- 确保在任何工作空间都能快速访问

## 实现建议

### 前端JS增强功能

```javascript
// 动态调整窗口高度
function resizeWindow(height) {
  window.tauri.invoke('resize_window', { height: Math.max(60, Math.min(400, height)) });
}

// 全局快捷键监听
window.tauri.invoke('register_global_shortcut', { 
  shortcut: 'CommandOrControl+Space' 
});

// 失去焦点时隐藏
document.addEventListener('visibilitychange', () => {
  if (document.hidden) {
    window.tauri.invoke('hide_window');
  }
});
```

### Rust后端增强

```rust
// 在 src/main.rs 中添加全局快捷键处理
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutManager};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

这些配置将帮助你的SparkNova项目更接近utools的功能体验！
