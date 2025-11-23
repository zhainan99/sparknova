# Tauri 窗口自定义配置完整指南

## 目录

1. [配置概览](#配置概览)
2. [基础窗口配置](#基础窗口配置)
3. [窗口外观配置](#窗口外观配置)
4. [窗口行为配置](#窗口行为配置)
5. [窗口状态配置](#窗口状态配置)
6. [平台特定配置](#平台特定配置)
7. [完整配置示例](#完整配置示例)
8. [常用场景解决方案](#常用场景解决方案)
9. [注意事项与最佳实践](#注意事项与最佳实践)
10. [故障排除](#故障排除)

## 配置概览

Tauri 2.0窗口配置分为以下主要类别：

- **基础配置**：标题、尺寸、位置等基本属性
- **外观配置**：边框、阴影、透明度等视觉属性
- **行为配置**：调整大小、置顶、任务栏等交互属性
- **状态配置**：可见性、全屏、最大化等状态属性

## 基础窗口配置

### 标题配置

```json
{
  "title": "SparkNova",
  "titleBarStyle": "default" // macOS专有
}
```

**参数说明：**

- `title`: 窗口标题，显示在标题栏上
- `titleBarStyle`: macOS平台标题栏样式（`default`, `hidden`, `hiddenInset`）

### 尺寸配置

```json
{
  "width": 600,
  "height": 400,
  "minWidth": null,
  "minHeight": null,
  "maxWidth": null,
  "maxHeight": null
}
```

**参数说明：**

- `width`: 窗口宽度（像素）
- `height`: 窗口高度（像素）
- `minWidth`: 最小宽度限制
- `minHeight`: 最小高度限制
- `maxWidth`: 最大宽度限制
- `maxHeight`: 最大高度限制

### 位置配置

```json
{
  "center": true,
  "x": null,
  "y": null,
  "resizable": true
}
```

**参数说明：**

- `center`: 启动时是否居中显示
- `x`: 窗口X坐标（配合center使用）
- `y`: 窗口Y坐标（配合center使用）
- `resizable`: 是否允许调整窗口大小

## 窗口外观配置

### 装饰元素配置

```json
{
  "decorations": true,
  "shadow": false,
  "transparent": false,
  "vibrancy": false
}
```

**参数说明：**

- `decorations`: 是否显示窗口装饰（标题栏、边框等）
  - `true`: 显示标准窗口装饰
  - `false`: 无装饰窗口（自定义标题栏场景）

- `shadow`: 窗口阴影效果
  - `true`: 启用阴影（默认值）
  - `false`: 禁用阴影（解决边框问题的关键）

- `transparent`: 窗口透明度
  - `true`: 启用透明背景
  - `false`: 不透明背景

- `vibrancy`: 毛玻璃效果（macOS）
  - `true`: 启用毛玻璃效果
  - `false`: 普通显示效果

### 窗口边框配置

```json
{
  "border": "default", // Windows
  "hiddenTitle": false, // macOS
  "titleBarOverlay": null // Windows 11
}
```

**重要说明：**
根据Tauri 2.0官方文档，窗口边框颜色、焦点状态边框样式等属于**系统级装饰**，无法通过配置直接自定义。这些是操作系统级别的行为：

- Windows: 蓝色焦点边框 / 灰色失焦边框
- macOS: 蓝色焦点环
- Linux: 桌面环境相关的边框显示

## 窗口行为配置

### 层级与置顶配置

```json
{
  "alwaysOnTop": false,
  "visibleOnAllWorkspaces": false,
  "skipTaskbar": false,
  "skipTaskbar": false
}
```

**参数说明：**

- `alwaysOnTop`: 始终保持在其他窗口之上
- `visibleOnAllWorkspaces`: 在所有工作空间可见（macOS）
- `skipTaskbar`: 是否跳过任务栏显示
- `skipTaskbar`: 隐藏任务栏图标

### 窗口控制配置

```json
{
  "closable": true,
  "maximizable": true,
  "minimizable": true,
  "resizable": true
}
```

**参数说明：**

- `closable`: 是否显示关闭按钮
- `maximizable`: 是否显示最大化按钮
- `minimizable`: 是否显示最小化按钮
- `resizable`: 是否允许调整大小

## 窗口状态配置

### 启动状态配置

```json
{
  "visible": true,
  "maximized": false,
  "minimized": false,
  "fullscreen": false,
  "resized": false
}
```

**参数说明：**

- `visible`: 启动时是否可见
- `maximized`: 启动时是否最大化
- `minimized`: 启动时是否最小化
- `fullscreen`: 是否全屏启动
- `resized`: 窗口是否已调整大小

### 内容相关配置

```json
{
  "url": "index.html",
  "focus": true,
  "acceptFirstMouse": false,
  "webPreferences": {
    "preload": null,
    "contextIsolation": true
  }
}
```

**参数说明：**

- `url`: 窗口加载的初始URL
- `focus`: 启动时是否获得焦点
- `acceptFirstMouse`: 首次点击时是否激活窗口
- `webPreferences`: WebView偏好设置

## 平台特定配置

### Windows特定配置

```json
{
  "windows": {
    "wry": "auto",
    "theme": "light",
    "titleBarOverlay": {
      "color": "#224488",
      "symbolColor": "#FFFFFF",
      "height": 48
    }
  }
}
```

**参数说明：**

- `wry`: Windows WebView 2运行时版本
- `theme`: 应用主题（`light`, `dark`, `auto`）
- `titleBarOverlay`: Windows 11标题栏覆盖

### macOS特定配置

```json
{
  "macOS": {
    "entitlements": null,
    "exceptionDomain": "",
    "frame": true,
    "fullScreenEnabled": true,
    "hiddenTitle": false,
    "titleBarStyle": "default",
    "windowBtnEnabled": true
  }
}
```

### Linux特定配置

```json
{
  "linux": {
    "appId": null,
    "display": null,
    "desktop": null,
    "gobject": null,
    "icon": null
  }
}
```

## 完整配置示例

### 1. 标准桌面应用配置

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "桌面应用",
  "version": "1.0.0",
  "identifier": "com.example.desktop",
  "app": {
    "windows": [
      {
        "title": "我的应用",
        "width": 800,
        "height": 600,
        "minWidth": 640,
        "minHeight": 480,
        "maxWidth": 1920,
        "maxHeight": 1080,
        "center": true,
        "resizable": true,
        "decorations": true,
        "shadow": true,
        "transparent": false,
        "alwaysOnTop": false,
        "skipTaskbar": false,
        "visible": true,
        "maximized": false,
        "minimized": false,
        "fullscreen": false,
        "closable": true,
        "maximizable": true,
        "minimizable": true,
        "focus": true
      }
    ]
  }
}
```

### 2. 无边框透明窗口配置（类似Spotlight）

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Spotlight应用",
  "version": "1.0.0",
  "identifier": "com.example.spotlight",
  "app": {
    "windows": [
      {
        "title": "搜索",
        "width": 600,
        "height": 60,
        "center": true,
        "resizable": false,
        "decorations": false,      // 关键：禁用装饰
        "transparent": true,       // 关键：透明背景
        "shadow": false,           // 关键：禁用阴影
        "alwaysOnTop": true,       // 始终置顶
        "skipTaskbar": true,       // 隐藏任务栏图标
        "visible": false,          // 启动时隐藏
        "fullscreen": false,
        "maximized": false,
        "visibleOnAllWorkspaces": false
      }
    ]
  }
}
```

### 3. 全屏应用配置

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "全屏应用",
  "version": "1.0.0",
  "identifier": "com.example.fullscreen",
  "app": {
    "windows": [
      {
        "title": "全屏演示",
        "width": 1920,
        "height": 1080,
        "center": false,
        "resizable": false,
        "decorations": false,
        "transparent": false,
        "shadow": false,
        "alwaysOnTop": false,
        "skipTaskbar": false,
        "visible": true,
        "fullscreen": true,
        "maximized": false,
        "minimized": false
      }
    ]
  }
}
```

## 常用场景解决方案

### 场景1：类似Spotlight的搜索应用

**需求：**

- 无边框透明窗口
- 始终置顶
- 隐藏任务栏
- 启动时隐藏

**解决方案：**

```json
{
  "title": "搜索",
  "width": 600,
  "height": 60,
  "decorations": false,    // 移除系统标题栏
  "transparent": true,     // 透明背景
  "shadow": false,         // 禁用阴影解决边框问题
  "alwaysOnTop": true,     // 始终置顶
  "skipTaskbar": true,     // 跳过任务栏
  "visible": false         // 启动时隐藏
}
```

### 场景2：固定大小工具面板

**需求：**

- 固定尺寸不能调整
- 始终可见
- 保持原始外观

**解决方案：**

```json
{
  "title": "工具面板",
  "width": 300,
  "height": 500,
  "resizable": false,      // 禁止调整大小
  "decorations": true,     // 保持装饰
  "shadow": true,          // 保持阴影
  "transparent": false,    // 不透明
  "alwaysOnTop": true,     // 始终置顶
  "skipTaskbar": false,    // 显示在任务栏
  "visible": true          // 启动时显示
}
```

### 场景3：多媒体全屏应用

**需求：**

- 全屏显示
- 无装饰界面
- 隐藏系统元素

**解决方案：**

```json
{
  "title": "播放器",
  "width": 1920,
  "height": 1080,
  "resizable": false,      // 固定大小
  "decorations": false,    // 全屏无装饰
  "transparent": false,    // 不透明
  "shadow": false,         // 全屏不需要阴影
  "alwaysOnTop": false,    // 不需要置顶
  "skipTaskbar": true,     // 隐藏任务栏
  "visible": true,         // 启动时显示
  "fullscreen": true       // 进入全屏
}
```

## 注意事项与最佳实践

### 配置限制说明

1. **系统级装饰无法自定义**
   - 窗口边框颜色（蓝色/灰色）
   - 焦点状态边框样式
   - 系统级窗口装饰外观

2. **平台差异考虑**
   - Windows: WebView 2运行时要求
   - macOS: 毛玻璃效果支持
   - Linux: 桌面环境差异

3. **性能优化建议**
   - 禁用不必要的阴影：`shadow: false`
   - 透明窗口性能较低，考虑使用CSS效果替代
   - 避免频繁的窗口状态切换

### 安全注意事项

1. **权限配置**
   - 仅启用必要的插件功能
   - 配置适当的安全策略
   - 避免过度的系统访问权限

2. **用户隐私**
   - 合理设置任务栏显示
   - 避免过度监控用户行为
   - 透明度设置可能影响可访问性

### 开发最佳实践

1. **配置分层**
   - 基础配置与平台特定配置分离
   - 使用环境变量区分开发和生产环境
   - 保持配置的可维护性

2. **测试策略**
   - 在不同操作系统上测试配置效果
   - 验证窗口行为的一致性
   - 测试边界条件和异常情况

## 故障排除

### 常见问题及解决方案

#### 问题1：窗口显示边框问题

**现象：** 无装饰窗口仍然显示蓝色/灰色边框
**原因：** 这是系统级窗口装饰，无法通过配置禁用
**解决方案：**

```json
{
  "shadow": false,  // 禁用阴影
  "decorations": false  // 确保禁用装饰
}
```

**进一步处理：** 接受系统行为，通过CSS增强应用内元素的视觉焦点

#### 问题2：透明窗口性能问题

**现象：** 透明窗口运行缓慢
**解决方案：**

1. 减少透明区域的面积
2. 使用CSS渐变替代透明度
3. 考虑使用模糊效果而不是透明度

#### 问题3：窗口置顶失效

**现象：** 设置了`alwaysOnTop: true`但窗口仍被其他窗口遮挡
**检查项：**

1. 是否有其他应用强制置顶
2. 操作系统窗口管理策略
3. 权限设置是否正确

#### 问题4：任务栏显示异常

**现象：** 设置`skipTaskbar: true`但图标仍显示
**解决方案：**

```json
{
  "skipTaskbar": true,
  "visible": false  // 确保启动时不可见
}
```

### 调试工具

1. **Tauri开发模式**
   - 使用 `cargo tauri info` 检查配置
   - 查看控制台错误信息
   - 使用Tauri开发者工具

2. **平台特定调试**
   - Windows: 检查WebView 2运行时
   - macOS: 验证权限配置
   - Linux: 检查桌面环境兼容性

### 性能监控

1. **窗口渲染性能**
   - 监控帧率变化
   - 检查内存使用情况
   - 评估启动时间

2. **资源占用优化**
   - 合理设置窗口尺寸
   - 避免频繁的状态切换
   - 使用适当的视觉效果

## 总结

Tauri 2.0提供了丰富的窗口配置选项，能够满足从简单桌面应用到复杂无边框窗口的各种需求。关键配置包括：

- **装饰控制**：`decorations`、`transparent`、`shadow`
- **行为配置**：`alwaysOnTop`、`skipTaskbar`、`resizable`
- **状态管理**：`visible`、`fullscreen`、`maximized`
- **平台适配**：根据不同操作系统调整特定配置

通过合理配置这些选项，可以创建各种类型的窗口应用。对于无法通过配置解决的系统级问题（如窗口边框颜色），建议通过CSS和视觉设计来优化用户体验。
