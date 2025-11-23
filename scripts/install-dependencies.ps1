# SparkNova Dependency Installation Script (Windows PowerShell Version)
# Usage: .\install-dependencies.ps1 [-Dev] [-Recommended] [-FrontendOnly] [-BackendOnly] [-Help]

param(
    [switch]$Dev,
    [switch]$Recommended,
    [switch]$FrontendOnly,
    [switch]$BackendOnly,
    [switch]$Help
)

# Color output functions
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

# Show help information
function Show-Help {
    Write-Host "SparkNova Dependency Installation Script" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor White
    Write-Host "  .\install-dependencies.ps1 [options]" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Options:" -ForegroundColor White
    Write-Host "  -Dev           Install development dependencies" -ForegroundColor Gray
    Write-Host "  -Recommended   Install recommended dependencies" -ForegroundColor Gray
    Write-Host "  -FrontendOnly  Install frontend dependencies only" -ForegroundColor Gray
    Write-Host "  -BackendOnly   Install backend dependencies only" -ForegroundColor Gray
    Write-Host "  -Help          Show this help information" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor White
    Write-Host "  .\install-dependencies.ps1 -Dev -Recommended" -ForegroundColor Gray
    Write-Host "  .\install-dependencies.ps1 -FrontendOnly" -ForegroundColor Gray
    Write-Host "  .\install-dependencies.ps1 -BackendOnly" -ForegroundColor Gray
}

# Check help option
if ($Help) {
    Show-Help
    exit 0
}

Write-Info "SparkNova Dependency Installation Script Started..."

# Check system requirements
function Test-SystemRequirements {
    Write-Info "Checking system requirements..."
    
    # Check Node.js
    try {
        $nodeVersion = node --version
        $nodeVersionNumber = $nodeVersion -replace 'v', ''
        
        # Simple version check (assuming Node.js 18+)
        $majorVersion = [int]($nodeVersionNumber -split '\.')[0]
        if ($majorVersion -lt 18) {
            throw "Node.js version too old: $nodeVersion (requires 18.0+)"
        }
        
        Write-Success "Node.js version check passed: $nodeVersion"
    }
    catch {
        Write-Error "Node.js not installed or version too old: $_"
        exit 1
    }
    
    # Check Rust
    try {
        $rustVersion = rustc --version
        Write-Success "Rust version check passed: $rustVersion"
    }
    catch {
        Write-Error "Rust not installed or rustc command unavailable: $_"
        exit 1
    }
    
    # Check package manager
    $packageManager = $null
    
    if (Get-Command bun -ErrorAction SilentlyContinue) {
        $packageManager = "bun"
    }
    elseif (Get-Command pnpm -ErrorAction SilentlyContinue) {
        $packageManager = "pnpm"
    }
    elseif (Get-Command yarn -ErrorAction SilentlyContinue) {
        $packageManager = "yarn"
    }
    elseif (Get-Command npm -ErrorAction SilentlyContinue) {
        $packageManager = "npm"
    }
    else {
        Write-Error "No supported package manager found (npm, yarn, pnpm, bun)"
        exit 1
    }
    
    Set-Variable -Name "PackageManager" -Value $packageManager -Scope Global
    Write-Success "System requirements check passed. Using package manager: $packageManager"
}

# Install frontend dependencies
function Install-FrontendDependencies {
    Write-Info "Installing frontend dependencies..."
    
    # Check if package.json exists
    if (-not (Test-Path "package.json")) {
        Write-Error "package.json file not found"
        exit 1
    }
    
    # Install basic dependencies
    switch ($Global:PackageManager) {
        "bun" {
            Write-Info "Installing frontend dependencies using Bun..."
            bun install
        }
        "pnpm" {
            Write-Info "Installing frontend dependencies using pnpm..."
            pnpm install
        }
        "yarn" {
            Write-Info "Installing frontend dependencies using Yarn..."
            yarn install
        }
        "npm" {
            Write-Info "Installing frontend dependencies using npm..."
            npm install
        }
    }
    
    # If development dependencies are included
    if ($Dev) {
        Write-Info "Installing development dependencies..."
        switch ($Global:PackageManager) {
            "bun" {
                if (Test-Path ".eslintrc.cjs") {
                    bun add -D @typescript-eslint/eslint-plugin @typescript-eslint/parser
                }
                if (Test-Path "vite.config.ts") {
                    bun add -D @vitejs/plugin-vue
                }
            }
            "pnpm" {
                if (Test-Path ".eslintrc.cjs") {
                    pnpm add -D @typescript-eslint/eslint-plugin @typescript-eslint/parser
                }
                if (Test-Path "vite.config.ts") {
                    pnpm add -D @vitejs/plugin-vue
                }
            }
            "yarn" {
                if (Test-Path ".eslintrc.cjs") {
                    yarn add -D @typescript-eslint/eslint-plugin @typescript-eslint/parser
                }
                if (Test-Path "vite.config.ts") {
                    yarn add -D @vitejs/plugin-vue
                }
            }
            "npm" {
                if (Test-Path ".eslintrc.cjs") {
                    npm install -D @typescript-eslint/eslint-plugin @typescript-eslint/parser
                }
                if (Test-Path "vite.config.ts") {
                    npm install -D @vitejs/plugin-vue
                }
            }
        }
    }
    
    # If recommended dependencies are included
    if ($Recommended) {
        Write-Info "Installing recommended frontend dependencies..."
        switch ($Global:PackageManager) {
            "bun" {
                bun add naive-ui @vicons/ionicons5 @vicons/tabler pinia pinia-plugin-persistedstate fuse.js lodash-es dayjs @vueuse/motion
            }
            "pnpm" {
                pnpm add naive-ui @vicons/ionicons5 @vicons/tabler pinia pinia-plugin-persistedstate fuse.js lodash-es dayjs @vueuse/motion
            }
            "yarn" {
                yarn add naive-ui @vicons/ionicons5 @vicons/tabler pinia pinia-plugin-persistedstate fuse.js lodash-es dayjs @vueuse/motion
            }
            "npm" {
                npm install naive-ui @vicons/ionicons5 @vicons/tabler pinia pinia-plugin-persistedstate fuse.js lodash-es dayjs @vueuse/motion
            }
        }
        
        # Install development tools
        switch ($Global:PackageManager) {
            "bun" {
                bun add -D eslint prettier husky lint-staged vitest @vue/test-utils typescript
            }
            "pnpm" {
                pnpm add -D eslint prettier husky lint-staged vitest @vue/test-utils typescript
            }
            "yarn" {
                yarn add -D eslint prettier husky lint-staged vitest @vue/test-utils typescript
            }
            "npm" {
                npm install -D eslint prettier husky lint-staged vitest @vue/test-utils typescript
            }
        }
    }
    
    Write-Success "Frontend dependencies installation completed."
}

# Update Cargo.toml file
function Update-CargoToml {
    Write-Info "Checking Cargo.toml file..."
    
    $cargoTomlPath = "src-tauri\Cargo.toml"
    
    # Check if file exists
    if (-not (Test-Path $cargoTomlPath)) {
        Write-Error "Cargo.toml file not found: $cargoTomlPath"
        return
    }
    
    # Add recommended dependencies
    if ($Recommended) {
        Write-Info "Checking recommended backend dependencies..."
        
        # Read file content
        try {
            $content = Get-Content $cargoTomlPath -Raw -ErrorAction Stop
        }
        catch {
            Write-Error "Cannot read Cargo.toml file: $_"
            return
        }
        
        # Check if recommended dependencies are already added
        if ($content -match "sqlx\s*=") {
            Write-Warning "Recommended dependencies seem to already be added, skipping update."
            return
        }
        
        # Create backup
        try {
            Copy-Item $cargoTomlPath "$cargoTomlPath.backup" -ErrorAction Stop
            Write-Info "Created Cargo.toml backup"
        }
        catch {
            Write-Warning "Could not create backup, but continuing"
        }
        
        # Add new dependencies
        $newDependencies = @"

# Recommended database dependencies
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite", "chrono", "uuid"] }
rusqlite = { version = "0.32", features = ["bundled"] }

# Configuration management
config = "0.14"
toml = "0.8"

# Tauri plugins (matching current version)
tauri-plugin-fs = "2.0.0-beta.8"
tauri-plugin-notification = "2.0.0-beta.8"
tauri-plugin-clipboard-manager = "2.0.0-beta.8"
tauri-plugin-tray-icon = "2.0.0-beta.8"
tauri-plugin-dialog = "2.0.0-beta.8"

# HTTP client
reqwest = { version = "0.12", features = ["json"] }

# Supabase support
supabase-rust = "0.1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
"@
        
        try {
            # Add new dependencies after [dependencies] section
            $updatedContent = $content + $newDependencies
            Set-Content $cargoTomlPath $updatedContent -NoNewline -ErrorAction Stop
            Write-Success "Cargo.toml file updated successfully"
        }
        catch {
            Write-Error "Failed to update Cargo.toml file: $_"
        }
    }
}

# Install backend dependencies
function Install-BackendDependencies {
    Write-Info "Installing backend dependencies..."
    
    # Update Cargo.toml
    Update-CargoToml
    
    # Check src-tauri directory
    if (-not (Test-Path "src-tauri")) {
        Write-Error "src-tauri directory not found"
        exit 1
    }
    
    # Build project to download dependencies
    try {
        Push-Location src-tauri
        Write-Info "Downloading and building Rust dependencies..."
        cargo build --verbose
        Pop-Location
        Write-Success "Backend dependencies installation completed"
    }
    catch {
        Write-Error "Failed to build Rust project: $_"
        Pop-Location
        exit 1
    }
}

# Setup Git hooks
function Setup-GitHooks {
    Write-Info "Setting up Git hooks..."
    
    # Check if in Git repository
    try {
        $null = git rev-parse --is-inside-work-tree 2>$null
        if ($LASTEXITCODE -ne 0) {
            throw "Not in git repo"
        }
    }
    catch {
        Write-Warning "Not in Git repository, skipping Git hooks setup"
        return
    }
    
    # Initialize Husky if needed
    if ($Recommended -and $Dev) {
        try {
            switch ($Global:PackageManager) {
                "bun" {
                    bunx husky install
                }
                "pnpm" {
                    pnpm exec husky install
                }
                "yarn" {
                    yarn exec husky install
                }
                "npm" {
                    npx husky install
                }
            }
            Write-Success "Git hooks setup completed"
        }
        catch {
            Write-Warning "Failed to setup Git hooks: $_"
        }
    }
}

# Main execution logic
try {
    # Check system requirements
    Test-SystemRequirements
    
    # Execute installation based on parameters
    if ($FrontendOnly) {
        Install-FrontendDependencies
    }
    elseif ($BackendOnly) {
        Install-BackendDependencies
    }
    else {
        # Install frontend dependencies
        if (-not $BackendOnly) {
            Install-FrontendDependencies
        }
        
        # Install backend dependencies
        if (-not $FrontendOnly) {
            Install-BackendDependencies
        }
    }
    
    # Setup Git hooks
    Setup-GitHooks
    
    Write-Success "All dependencies installation completed!"
    Write-Info "You can use the following commands to start the development server:"
    Write-Host "  Frontend: npm run dev" -ForegroundColor Cyan
    Write-Host "  Full app: npm run tauri dev" -ForegroundColor Cyan
    
}
catch {
    Write-Error "An error occurred during installation: $_"
    exit 1
}