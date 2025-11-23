# SparkNova 依赖安装与更新指南

本项目提供了便捷的依赖安装和更新脚本，支持多种操作系统和包管理器。

## 快速开始

### Windows 用户

在 PowerShell 中运行：

```powershell
# 安装所有依赖（包括推荐依赖）
.\scripts\install-dependencies.ps1 -Dev -Recommended

# 仅安装前端依赖
.\scripts\install-dependencies.ps1 -FrontendOnly

# 仅安装后端依赖
.\scripts\install-dependencies.ps1 -BackendOnly

# 查看帮助
.\scripts\install-dependencies.ps1 -Help
```

### macOS/Linux 用户

在终端中运行：

```bash
# 给脚本添加执行权限
chmod +x scripts/install-dependencies.sh

# 安装所有依赖（包括推荐依赖）
./scripts/install-dependencies.sh -d -r

# 仅安装前端依赖
./scripts/install-dependencies.sh -f

# 仅安装后端依赖
./scripts/install-dependencies.sh -b

# 查看帮助
./scripts/install-dependencies.sh -h
```

### 更新依赖

#### Windows 安装步骤

在 PowerShell 中运行：

```powershell
# 检查过时的依赖
.\scripts\update-dependencies.ps1 -Check

# 更新前端依赖到次版本
.\scripts\update-dependencies.ps1 -Frontend -Minor

# 更新后端依赖到补丁版本
.\scripts\update-dependencies.ps1 -Backend -Patch

# 更新所有依赖到主版本
.\scripts\update-dependencies.ps1 -Major

# 查看帮助
.\scripts\update-dependencies.ps1 -Help
```

#### macOS/Linux 安装步骤

在终端中运行：

```bash
# 给脚本添加执行权限
chmod +x scripts/update-dependencies.sh

# 检查过时的依赖
./scripts/update-dependencies.sh -c

# 更新前端依赖到次版本
./scripts/update-dependencies.sh -f -m

# 更新后端依赖到补丁版本
./scripts/update-dependencies.sh -b -p

# 更新所有依赖到主版本
./scripts/update-dependencies.sh -M

# 查看帮助
./scripts/update-dependencies.sh -h
```

## 系统要求

- **Node.js**: 18.0 或更高版本
- **Rust**: 1.70.0 或更高版本
- **包管理器**: npm、yarn、pnpm 或 bun（自动检测）

## 依赖管理文档

详细的依赖管理文档请参考：[docs/dependency-management.md](../docs/dependency-management.md)

## 故障排除

### Windows 执行策略问题

如果在运行 PowerShell 脚本时遇到执行策略问题，请以管理员身份运行 PowerShell 并执行：

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 权限问题 (macOS/Linux)

如果在 Linux/macOS 上遇到权限问题，请确保脚本有执行权限：

```bash
chmod +x scripts/install-dependencies.sh
```

### 网络问题

如果安装依赖时遇到网络问题，可以尝试：

1. 使用国内镜像源（如淘宝镜像）
2. 配置代理
3. 检查防火墙设置

## 手动安装

如果自动安装脚本无法工作，您也可以手动安装依赖：

### 前端依赖

```bash
# 使用 npm
npm install

# 使用 yarn
yarn install

# 使用 pnpm
pnpm install

# 使用 bun
bun install
```

### 后端依赖

```bash
cd src-tauri
cargo build
```

### 推荐依赖

请参考 [docs/dependency-management.md](../docs/dependency-management.md) 中的推荐依赖列表。

## 开发环境设置

安装完依赖后，您可以：

1. 启动开发服务器：

   ```bash
   npm run tauri dev
   ```

2. 构建生产版本：

   ```bash
   npm run tauri build
   ```

3. 运行测试：

   ```bash
   npm run test
   ```

## 贡献

如果您对依赖管理有改进建议，请提交 Issue 或 Pull Request。

## 脚本详细说明

### 安装脚本

- `install-dependencies.ps1` (Windows PowerShell)
- `install-dependencies.sh` (macOS/Linux Bash)

这些脚本用于安装项目依赖，包括：

- 前端依赖 (Node.js)
- 后端依赖 (Rust/Cargo)
- 推荐依赖 (可选)
- 开发工具配置 (可选)

### 更新脚本

- `update-dependencies.ps1` (Windows PowerShell)
- `update-dependencies.sh` (macOS/Linux Bash)

这些脚本用于更新项目依赖，包括：

- 检查过时的依赖
- 更新到指定版本范围 (补丁/次版本/主版本)
- 自动生成更新日志

### 更新版本策略

- **补丁版本更新** (`-p`/`-Patch`): 仅更新补丁版本，风险最低
- **次版本更新** (`-m`/`-Minor`): 更新到次版本，可能包含新功能和修复
- **主版本更新** (`-M`/`-Major`): 更新到主版本，可能包含破坏性变更

建议定期进行补丁和次版本更新，主版本更新需要谨慎评估。
