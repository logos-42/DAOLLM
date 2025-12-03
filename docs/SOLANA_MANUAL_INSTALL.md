# Solana CLI 手动安装指南

由于网络连接问题，自动安装失败。以下是几种手动安装方法：

## 方法1: 使用Cargo安装（推荐，需要网络）

### 前提条件
- 已安装Rust工具链（`rustc` 和 `cargo`）
- 能够访问GitHub

### 安装步骤

```powershell
# 1. 确保Cargo在PATH中
$env:Path += ";$env:USERPROFILE\.cargo\bin"

# 2. 全局安装Solana CLI
cargo install --git https://github.com/solana-labs/solana solana-cli --locked --force

# 3. 验证安装
solana --version
```

### 如果遇到网络问题

#### 配置Git代理（如果有代理）
```powershell
# 设置HTTP代理
git config --global http.proxy http://proxy.example.com:8080
git config --global https.proxy https://proxy.example.com:8080

# 取消代理
git config --global --unset http.proxy
git config --global --unset https.proxy
```

#### 配置Cargo镜像（使用国内镜像）
创建或编辑 `$env:USERPROFILE\.cargo\config.toml`：

```toml
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"

[net]
git-fetch-with-cli = true
```

## 方法2: 下载预编译二进制文件（无需编译）

### 步骤

1. **访问Solana发布页面**
   - https://github.com/solana-labs/solana/releases
   - 找到最新的稳定版本

2. **下载Windows版本**
   - 文件名类似：`solana-release-x86_64-pc-windows-msvc.tar.bz2`
   - 或查找 `.zip` 格式的Windows安装包

3. **解压文件**
   ```powershell
   # 创建安装目录
   New-Item -ItemType Directory -Force -Path "C:\solana"
   
   # 解压到该目录（使用7-Zip或WinRAR）
   # 或使用PowerShell解压
   Expand-Archive -Path "solana-release-*.zip" -DestinationPath "C:\solana"
   ```

4. **添加到PATH环境变量**
   ```powershell
   # 获取bin目录路径（通常是 C:\solana\bin）
   $solanaBin = "C:\solana\bin"
   
   # 添加到用户PATH
   $currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
   if ($currentPath -notlike "*$solanaBin*") {
       [Environment]::SetEnvironmentVariable("Path", "$currentPath;$solanaBin", [EnvironmentVariableTarget]::User)
   }
   
   # 重新加载PATH
   $env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')
   ```

5. **验证安装**
   ```powershell
   solana --version
   ```

## 方法3: 使用官方安装脚本

### Windows PowerShell脚本

```powershell
# 下载并运行官方安装脚本
Invoke-WebRequest https://release.solana.com/stable/install -OutFile solana-install.ps1
powershell -ExecutionPolicy Bypass -File solana-install.ps1
```

### 如果脚本下载失败

1. 手动下载安装脚本：
   - 访问：https://release.solana.com/stable/install
   - 保存为 `solana-install.ps1`

2. 运行脚本：
   ```powershell
   powershell -ExecutionPolicy Bypass -File solana-install.ps1
   ```

3. 添加到PATH（如果脚本没有自动添加）：
   ```powershell
   $solanaPath = "$env:USERPROFILE\.local\share\solana\install\active_release\bin"
   if (Test-Path $solanaPath) {
       $currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
       [Environment]::SetEnvironmentVariable("Path", "$currentPath;$solanaPath", [EnvironmentVariableTarget]::User)
   }
   ```

## 方法4: 使用WSL（Windows Subsystem for Linux）

如果您已安装WSL，可以在Linux环境中安装：

```bash
# 在WSL中执行
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# 添加到PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# 验证
solana --version
```

## 验证安装

安装完成后，重启PowerShell并运行：

```powershell
# 检查Solana版本
solana --version

# 检查Solana配置
solana config get

# 设置测试网（可选）
solana config set --url https://api.testnet.solana.com
```

## 常见问题

### 问题1: `solana` 命令未找到

**解决方案：**
```powershell
# 重新加载PATH
$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')

# 或重启PowerShell
```

### 问题2: Cargo安装失败（网络超时）

**解决方案：**
- 配置Git和Cargo代理（见方法1）
- 使用预编译二进制文件（方法2）
- 使用VPN或代理工具

### 问题3: 权限错误

**解决方案：**
```powershell
# 以管理员身份运行PowerShell
# 或使用用户级安装目录
```

## 快速检查清单

- [ ] Rust工具链已安装（`rustc --version`）
- [ ] Cargo可用（`cargo --version`）
- [ ] 网络连接正常（可访问GitHub）
- [ ] PATH环境变量已配置
- [ ] Solana CLI已安装（`solana --version`）

## 下一步

安装完成后，可以：

1. 配置Solana网络：
   ```powershell
   solana config set --url https://api.testnet.solana.com  # 测试网
   solana config set --url https://api.mainnet-beta.solana.com  # 主网
   ```

2. 创建钱包：
   ```powershell
   solana-keygen new
   ```

3. 检查余额：
   ```powershell
   solana balance
   ```

4. 构建项目：
   ```powershell
   anchor build
   ```

