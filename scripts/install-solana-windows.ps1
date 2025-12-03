# Solana和Anchor Windows安装脚本
# 适用于Windows PowerShell

Write-Host "开始安装Solana和Anchor工具链..." -ForegroundColor Green

# 检查是否已安装Solana
$solanaInstalled = Get-Command solana -ErrorAction SilentlyContinue
if ($solanaInstalled) {
    Write-Host "Solana已安装: $(solana --version)" -ForegroundColor Yellow
} else {
    Write-Host "正在全局安装Solana CLI..." -ForegroundColor Cyan
    
    # 确保Cargo在PATH中
    $cargoBin = "$env:USERPROFILE\.cargo\bin"
    if ($env:Path -notlike "*$cargoBin*") {
        $env:Path += ";$cargoBin"
        Write-Host "已将Cargo添加到当前会话PATH" -ForegroundColor Yellow
    }
    
    # 方法1: 使用Cargo全局安装（推荐，最可靠）
    Write-Host "使用Cargo全局安装Solana CLI..." -ForegroundColor Cyan
    Write-Host "这可能需要几分钟时间，请耐心等待..." -ForegroundColor Yellow
    
    try {
        # 检查Cargo是否可用
        $cargoCheck = Get-Command cargo -ErrorAction SilentlyContinue
        if (-not $cargoCheck) {
            Write-Host "错误: Cargo未找到。请先安装Rust工具链:" -ForegroundColor Red
            Write-Host "访问 https://rustup.rs/ 安装Rust" -ForegroundColor Yellow
            exit 1
        }
        
        # 测试网络连接
        Write-Host "检查网络连接..." -ForegroundColor Cyan
        try {
            $testConnection = Test-NetConnection -ComputerName github.com -Port 443 -WarningAction SilentlyContinue
            if (-not $testConnection.TcpTestSucceeded) {
                Write-Host "警告: 无法连接到GitHub，可能需要使用代理或镜像" -ForegroundColor Yellow
            }
        } catch {
            Write-Host "网络检查跳过，继续尝试安装..." -ForegroundColor Yellow
        }
        
        # 使用Cargo全局安装Solana CLI
        Write-Host "开始从GitHub下载并编译Solana CLI..." -ForegroundColor Cyan
        Write-Host "注意: 不使用 --locked 标志以避免依赖版本冲突" -ForegroundColor Yellow
        $installResult = cargo install --git https://github.com/solana-labs/solana solana-cli --force 2>&1
        
        # 检查安装是否成功
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo安装失败，退出代码: $LASTEXITCODE"
        }
        
        # 确保Cargo bin目录在PATH中（永久）
        if ($env:Path -notlike "*$cargoBin*") {
            $currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
            if ($currentPath -notlike "*$cargoBin*") {
                [Environment]::SetEnvironmentVariable("Path", "$currentPath;$cargoBin", [EnvironmentVariableTarget]::User)
                Write-Host "已将Cargo bin目录添加到用户PATH环境变量" -ForegroundColor Green
            }
        }
        
        # 重新加载PATH
        $env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')
        
        # 验证安装
        Start-Sleep -Seconds 2
        $solanaCheck = Get-Command solana -ErrorAction SilentlyContinue
        if ($solanaCheck) {
            Write-Host "✓ Solana CLI全局安装成功: $(solana --version)" -ForegroundColor Green
        } else {
            Write-Host "安装完成，但需要重启PowerShell才能使用solana命令" -ForegroundColor Yellow
            Write-Host "或运行: `$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "Cargo安装失败: $_" -ForegroundColor Red
        Write-Host "`n尝试备用安装方法..." -ForegroundColor Yellow
        
        # 方法2: 使用官方安装脚本（备用）
        try {
            $installScript = "$env:TEMP\solana-install.ps1"
            Invoke-WebRequest -Uri "https://release.solana.com/stable/install" -OutFile $installScript
            & powershell -ExecutionPolicy Bypass -File $installScript
            
            # 添加到PATH
            $solanaPath = "$env:USERPROFILE\.local\share\solana\install\active_release\bin"
            if (Test-Path $solanaPath) {
                $currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
                if ($currentPath -notlike "*$solanaPath*") {
                    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$solanaPath", [EnvironmentVariableTarget]::User)
                    Write-Host "Solana已添加到PATH" -ForegroundColor Green
                }
            }
        } catch {
            Write-Host "备用安装方法也失败: $_" -ForegroundColor Red
            Write-Host "`n=== 手动安装指南 ===" -ForegroundColor Yellow
            Write-Host "`n方法1: 使用Cargo安装（需要网络连接）" -ForegroundColor Cyan
            Write-Host "   注意: 不要使用 --locked 标志以避免依赖版本冲突" -ForegroundColor Yellow
            Write-Host "   cargo install --git https://github.com/solana-labs/solana solana-cli --force" -ForegroundColor White
            Write-Host "`n方法2: 下载预编译二进制文件" -ForegroundColor Cyan
            Write-Host "   1. 访问: https://github.com/solana-labs/solana/releases" -ForegroundColor White
            Write-Host "   2. 下载最新的Windows版本（solana-release-x86_64-pc-windows-msvc.tar.bz2）" -ForegroundColor White
            Write-Host "   3. 解压到本地目录（如 C:\solana）" -ForegroundColor White
            Write-Host "   4. 将bin目录添加到PATH环境变量" -ForegroundColor White
            Write-Host "`n方法3: 使用官方安装脚本" -ForegroundColor Cyan
            Write-Host "   访问: https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor White
            Write-Host "`n如果遇到网络问题，可以:" -ForegroundColor Yellow
            Write-Host "   - 配置Git代理: git config --global http.proxy http://proxy:port" -ForegroundColor White
            Write-Host "   - 配置Cargo镜像（在 ~/.cargo/config.toml 中）" -ForegroundColor White
            Write-Host "   - 使用VPN或代理工具" -ForegroundColor White
        }
    }
}

# 检查是否已安装Anchor
$anchorInstalled = Get-Command anchor -ErrorAction SilentlyContinue
if ($anchorInstalled) {
    Write-Host "Anchor已安装: $(anchor --version)" -ForegroundColor Yellow
} else {
    Write-Host "正在安装Anchor..." -ForegroundColor Cyan
    
    # 安装AVM (Anchor Version Manager)
    Write-Host "安装AVM (Anchor Version Manager)..." -ForegroundColor Cyan
    try {
        cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
        
        # 初始化AVM
        $avmPath = "$env:USERPROFILE\.cargo\bin\avm.exe"
        if (Test-Path $avmPath) {
            & $avmPath install latest
            & $avmPath use latest
            
            # 添加到PATH
            $cargoBin = "$env:USERPROFILE\.cargo\bin"
            if ($env:Path -notlike "*$cargoBin*") {
                $env:Path += ";$cargoBin"
                [Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::User)
            }
            
            Write-Host "Anchor已安装并配置" -ForegroundColor Green
        }
    } catch {
        Write-Host "Anchor安装失败: $_" -ForegroundColor Red
        Write-Host "请手动安装: cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked" -ForegroundColor Yellow
    }
}

# 验证安装
Write-Host "`n验证安装..." -ForegroundColor Cyan
$solanaCheck = Get-Command solana -ErrorAction SilentlyContinue
$anchorCheck = Get-Command anchor -ErrorAction SilentlyContinue

if ($solanaCheck) {
    Write-Host "✓ Solana: $(solana --version)" -ForegroundColor Green
} else {
    Write-Host "✗ Solana未安装" -ForegroundColor Red
}

if ($anchorCheck) {
    Write-Host "✓ Anchor: $(anchor --version)" -ForegroundColor Green
} else {
    Write-Host "✗ Anchor未安装" -ForegroundColor Red
}

Write-Host "`n安装完成！如果工具未找到，请重启PowerShell或重新加载环境变量。" -ForegroundColor Green
Write-Host "重新加载PATH: `$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')" -ForegroundColor Yellow

