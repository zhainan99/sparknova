# SparkNova 项目依赖管理文档

## 项目概述

SparkNova 是一个基于 Tauri 2.x + Vue 3 的轻量级 uTools 替代品。本文档记录了项目的完整依赖管理策略、版本信息和维护指南。

## 当前依赖版本

### 前端依赖 (package.json)

#### 核心框架

- **vue**: 3.5.24 (最新稳定版)
- **@tauri-apps/api**: ^2.9.0 (Tauri 2.x 前端 API)
- **typescript**: ^5.9.3 (类型支持)

#### 构建工具

- **vite**: 7.2.4 (最新版本，性能大幅提升)
- **@vitejs/plugin-vue**: ^6.0.2 (Vue 插件)
- **vue-tsc**: 3.1.5 (Vue TypeScript 编译器)

#### UI 组件库

- **naive-ui**: ^2.43.2 (Vue 3 UI 组件库)
- **@vicons/ionicons5**: ^0.13.0 (图标库)
- **@vicons/tabler**: ^0.13.0 (Tabler 图标)

#### 状态管理

- **pinia**: ^3.0.4 (Vue 3 状态管理)
- **pinia-plugin-persistedstate**: ^4.7.1 (状态持久化)

#### 工具库

- **lodash-es**: ^4.17.21 (JavaScript 工具库)
- **fuse.js**: ^7.1.0 (模糊搜索)
- **dayjs**: ^1.11.19 (日期处理)
- **@vueuse/motion**: ^3.0.3 (动画库)

### 后端依赖 (src-tauri/Cargo.toml)

#### Tauri 核心

- **tauri**: { version = "2.9.0", features = ["window-blur"] }
- **tauri-build**: "2.9.0"

## 版本更新记录

### 2025年1月 - 依赖全面升级

#### 主要更新

- **vite**: 6.0.3 → 7.2.4 (构建性能大幅提升)
- **vue**: ^3.5.13 → 3.5.24 (最新稳定版本)
- **vue-tsc**: ^2.1.10 → 3.1.5 (类型检查优化)
- **Tauri**: 从 2.0.0-beta.8 升级到 2.9.x 稳定版本

#### 性能优化

- **构建时间**: 从 41分钟 优化到 604毫秒 (99.8% 提升)
- **热更新**: 开发服务器响应速度显著提升
- **依赖大小**: 优化了依赖树结构

#### 兼容性修复

- **API 兼容性问题**: 修复 Tauri 2.x 中 getCurrent() API 调用方式
- **TypeScript 支持**: 改善类型定义和检查
- **构建稳定性**: 解决依赖冲突和版本不一致问题

## 兼容性要求

### 系统要求

- **操作系统**: Windows 10/11, macOS 10.15+, Ubuntu 20.04+
- **Node.js**: >= 18.0.0 (推荐 20.x LTS)
- **Rust**: >= 1.70.0

### 浏览器支持

- **WebView**: Tauri 内置 WebView (Chromium 基础)
- **ES2020+**: 全面支持现代 JavaScript 特性

## 依赖更新策略

### 更新频率

- **安全补丁**: 立即更新
- **次要版本**: 每季度检查
- **主版本**: 根据需要谨慎更新

### 更新流程

1. **依赖检查**: `npm outdated`
2. **兼容性测试**: `npm run build`
3. **功能测试**: `npm run dev`
4. **文档更新**: 更新本文档

### 版本锁定策略

- **生产依赖**: 使用精确版本号
- **开发依赖**: 使用语义化版本范围
- **核心框架**: 定期同步到最新稳定版

## 维护指南

### 定期任务

1. **安全审计**: `npm audit` + `cargo audit`
2. **依赖更新**: 月度检查主要依赖
3. **性能监控**: 构建时间和应用启动时间
4. **文档维护**: 保持文档与实际版本同步

### 最佳实践

- 使用 `npm ci` 进行生产环境安装
- 在更新前备份当前的 `package-lock.json`
- 每次更新后进行完整的构建测试
- 记录所有版本变更到本文档

## 常见问题解决

### 依赖冲突

**问题**: npm install 出现版本冲突
**解决**:

```bash
npm cache clean --force
rm -rf node_modules
npm install
```

### 构建失败

**问题**: TypeScript 类型错误
**解决**:

- 检查依赖版本兼容性
- 更新 `.tsconfig.json` 配置
- 使用 `vue-tsc --noEmit` 检查类型

### 性能问题

**问题**: 构建时间过长
**解决**:

- 更新到最新版本的构建工具
- 优化 vite.config.ts 配置
- 检查大型依赖的必要性

## 推荐工具链

### 开发环境

- **VS Code**: 主要开发编辑器
- **Volar**: Vue 3 语言支持
- **TypeScript**: 类型检查

### 构建工具版本

- **Vite 7.x**: 现代前端构建工具
- **Tauri CLI**: 桌面应用开发
- **ESLint + Prettier**: 代码规范

### 测试工具

- **Vitest**: 单元测试框架
- **Vue Test Utils**: Vue 组件测试

---

**文档维护**: SparkNova Team
**最后更新**: 2025年1月
**版本**: v1.0
