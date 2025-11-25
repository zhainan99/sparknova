# 将Svelte项目迁移到SvelteKit

## 迁移目标
将当前基于Svelte的项目迁移到SvelteKit框架，确保与Tauri 2.0的集成正常工作。

## 迁移步骤

### 1. 安装SvelteKit相关依赖
- 安装SvelteKit核心依赖
- 安装SvelteKit适配器
- 移除不再需要的Svelte相关依赖

### 2. 创建SvelteKit项目结构
- 创建`src/routes`目录，包含`+page.svelte`和`+layout.svelte`
- 创建`src/lib`目录用于存放共享组件和工具
- 创建`src/app.d.ts`类型定义文件
- 创建`src/app.html`入口文件

### 3. 迁移代码
- 将`App.svelte`的内容迁移到`src/routes/+page.svelte`
- 调整`main.ts`的逻辑，使用SvelteKit的方式初始化
- 确保Tauri API的调用方式与SvelteKit兼容

### 4. 配置文件调整
- 创建`Svelte.config.js`配置文件
- 调整`vite.config.mts`以支持SvelteKit
- 更新`package.json`脚本，使用SvelteKit的构建命令

### 5. 验证集成
- 确保Tauri与SvelteKit的集成正常
- 测试快捷键功能是否正常工作
- 测试窗口显示和隐藏功能

## 预期结果
- 项目成功迁移到SvelteKit框架
- 保持与Tauri 2.0的良好集成
- 所有原有功能正常工作
- 项目结构符合SvelteKit最佳实践

## 注意事项
- 确保SvelteKit的版本与Tauri兼容
- 注意SvelteKit的路由系统与原项目的差异
- 保持Tauri相关配置不变
- 确保构建流程正确