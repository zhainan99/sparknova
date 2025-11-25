# 清理Svelte传统项目残留文件和代码

## 分析结果
通过对项目目录结构和文件内容的分析，发现以下不符合SvelteKit规范的残留文件和代码：

### 1. 残留文件
- `src/App.svelte`：传统Svelte项目的主组件，已被`src/routes/+page.svelte`替代
- `src/main.ts`：传统Svelte项目的入口文件，已被SvelteKit的自动入口机制替代
- 根目录下的`index.html`：传统Vite项目的入口HTML文件，已被`src/app.html`替代
- `public`目录：SvelteKit通常使用`static`目录代替

### 2. 残留配置
- `package.json`：可能存在残留的传统Svelte相关依赖和脚本
- `vite.config.mts`：需要确认是否有其他残留配置
- `tsconfig.json`：需要确认是否有其他残留配置

## 清理计划

### 1. 删除残留文件
- 删除`src/App.svelte`
- 删除`src/main.ts`
- 删除根目录下的`index.html`
- 将`public`目录重命名为`static`

### 2. 更新配置文件
- 检查`package.json`，移除残留的传统Svelte相关依赖和脚本
- 检查`vite.config.mts`，确保只保留SvelteKit相关配置
- 检查`tsconfig.json`，确保只保留SvelteKit相关配置

### 3. 验证项目构建和运行
- 运行`bun run build`命令，确保项目能够正常构建
- 运行`bun run dev`命令，确保项目能够正常运行

### 4. 验证功能完整性
- 测试快捷键功能是否正常工作
- 测试窗口显示和隐藏功能是否正常工作
- 测试其他原有功能是否正常工作

## 预期结果
- 项目中不再存在任何不符合SvelteKit规范的残留文件和代码
- 项目能够正常构建和运行
- 所有原有功能保持正常工作
- 项目结构完全符合SvelteKit最佳实践

## 注意事项
- 在删除文件之前，确保备份重要文件
- 确保所有配置更改都符合SvelteKit规范
- 验证所有功能是否正常工作
- 确保项目能够正常构建和运行