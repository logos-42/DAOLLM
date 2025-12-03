# 安装Solana工具链脚本
# 适用于Windows PowerShell

Write-Host "开始安装Solana工具链..." -ForegroundColor Green

# 方法1: 使用PowerShell下载安装脚本
$installScript = "$env:TEMP\solana-install.ps1"
$solanaInstalled = $false

try {
    Write-Host "下载Solana安装脚本..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri "https://release.solana.com/stable/install" -OutFile $installScript -TimeoutSec 30
    
    Write-Host "执行安装脚本..." -ForegroundColor Cyan
    & powershell -ExecutionPolicy Bypass -File $installScript
    
    # 检查安装路径
    $possiblePaths = @(
        "$env:USERPROFILE\.local\share\solana\install\active_release\bin",
        "$env:LOCALAPPDATA\solana\install\active_release\bin",
        "$env:ProgramFiles\Solana\bin"
    )
    
    foreach ($path in $possiblePaths) {
        if (Test-Path $path) {
            $env:Path += ";$path"
            [Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::User)
            Write-Host "Solana已安装到: $path" -ForegroundColor Green
            $solanaInstalled = $true
            break
        }
    }
    
    if (-not $solanaInstalled) {
        Write-Host "警告: 无法自动检测Solana安装路径" -ForegroundColor Yellow
        Write-Host "请手动将Solana bin目录添加到PATH环境变量" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "自动安装失败: $_" -ForegroundColor Red
    Write-Host "`n尝试方法2: 使用Cargo安装..." -ForegroundColor Cyan
    
    try {
        Write-Host "使用Cargo安装Solana CLI..." -ForegroundColor Cyan
        cargo install --git https://github.com/solana-labs/solana solana-cli --locked --force
        
        $cargoBin = "$env:USERPROFILE\.cargo\bin"
        if (Test-Path "$cargoBin\solana.exe") {
            Write-Host "Solana CLI已通过Cargo安装" -ForegroundColor Green
            $solanaInstalled = $true
        }
    } catch {
        Write-Host "Cargo安装也失败: $_" -ForegroundColor Red
        Write-Host "`n请手动安装Solana:" -ForegroundColor Yellow
        Write-Host "1. 访问 https://docs.solana.com/cli/install-solana-cli-tools" -ForegroundColor Yellow
        Write-Host "2. 下载Windows安装程序" -ForegroundColor Yellow
        Write-Host "3. 或使用WSL安装" -ForegroundColor Yellow
    }
}

# 验证安装
if ($solanaInstalled) {
    Write-Host "`n验证安装..." -ForegroundColor Cyan
    Start-Sleep -Seconds 2  # 等待PATH更新
    
    try {
        $solanaVersion = solana --version 2>&1
        Write-Host "✓ Solana已安装: $solanaVersion" -ForegroundColor Green
    } catch {
        Write-Host "✗ Solana命令未找到，请重启PowerShell后重试" -ForegroundColor Yellow
        Write-Host "或手动添加Solana bin目录到PATH" -ForegroundColor Yellow
    }
} else {
    Write-Host "`n✗ Solana安装未完成" -ForegroundColor Red
}

Write-Host "`n安装完成！" -ForegroundColor Green

