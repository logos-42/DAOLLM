# Windows安装Solana和Anchor指南

## 方法1: 使用Cargo全局安装（推荐）

### 安装Solana CLI

```powershell
# 确保Cargo在PATH中
$env:Path += ";$env:USERPROFILE\.cargo\bin"

# 全局安装Solana CLI（这可能需要几分钟）
cargo install --git https://github.com/solana-labs/solana solana-cli --locked --force

# 确保Cargo bin目录永久添加到PATH
$cargoBin = "$env:USERPROFILE\.cargo\bin"
$currentPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($currentPath -notlike "*$cargoBin*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$cargoBin", [EnvironmentVariableTarget]::User)
}

# 重新加载PATH
$env:Path = [System.Environment]::GetEnvironmentVariable('Path','Machine') + ';' + [System.Environment]::GetEnvironmentVariable('Path','User')

# 验证安装
solana --version
```

**注意**: 如果安装后仍无法找到`solana`命令，请重启PowerShell或运行上面的PATH重新加载命令。

### 安装Anchor CLI

```powershell
# 安装AVM (Anchor Version Manager)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force

# 安装最新版Anchor
avm install latest
avm use latest

# 验证安装
anchor --version
```

## 方法2: 手动下载安装（如果方法1失败）

### Solana安装

1. 访问: https://github.com/solana-labs/solana/releases
2. 下载最新的Windows安装包（.exe或.msi）
3. 运行安装程序
4. 将安装目录添加到PATH环境变量

### Anchor安装

1. 访问: https://github.com/coral-xyz/anchor/releases
2. 下载Windows版本
3. 解压到本地目录
4. 添加到PATH环境变量

## 方法3: 使用WSL（Windows Subsystem for Linux）

如果您有WSL，可以在WSL中安装：

```bash
# 在WSL中执行
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

## 验证安装

安装完成后，重启PowerShell并运行：

```powershell
# 检查Solana
solana --version

# 检查Anchor
anchor --version

# 检查Rust工具链
rustc --version
cargo --version
```

## 常见问题

### 问题1: `solana`命令未找到

**解决方案:**
```powershell
# 手动添加到PATH
$solanaPath = "$env:USERPROFILE\.cargo\bin"
$env:Path += ";$solanaPath"
[Environment]::SetEnvironmentVariable("Path", $env:Path, [EnvironmentVariableTarget]::User)
```

### 问题2: Cargo安装失败

**解决方案:**
```powershell
# 更新Rust工具链
rustup update stable

# 清理cargo缓存
cargo clean

# 重新安装
cargo install --git https://github.com/solana-labs/solana solana-cli --locked --force
```

### 问题3: 网络问题

如果GitHub访问有问题，可以使用镜像或代理。

## 快速安装脚本

运行项目根目录下的安装脚本：

```powershell
cd d:\AI\model\DAOLLM
.\scripts\install-solana-windows.ps1
```

## 构建项目

安装完成后，使用Anchor构建项目：

```powershell
cd d:\AI\model\DAOLLM
anchor build
```

如果Anchor未安装，也可以直接使用cargo（但会有宏展开错误，这是正常的）：

```powershell
cd programs\daollm
cargo check  # 会有E0432错误，这是正常的，需要Anchor工具链
```

## 注意

- Anchor的`#[program]`宏需要Solana BPF工具链才能正确编译
- 使用`cargo check`在非BPF目标上会显示`unresolved import 'crate'`错误，这是正常的
- 只有使用`anchor build`或正确的BPF目标才能完整编译

