# DAO提案系统 - 去中心化AI分析平台

基于Solana和Rust构建的去中心化DAO提案分析系统，使用AI自动生成提案摘要、风险评估和决策建议。

## 项目概述

这个系统让DAO治理变得更简单、更高效：
- 🤖 AI自动分析提案，生成简洁易懂的摘要
- 🌐 去中心化推理网络，多个节点提供分析服务
- 💰 代币激励机制，贡献者获得奖励
- 🔒 IPFS去中心化存储，数据永久保存
- ⚡ Solana高性能区块链，交易费用极低

## 技术栈

- **智能合约：** Rust + Anchor框架
- **区块链：** Solana Devnet → Mainnet
- **后端：** Rust + Axum
- **前端：** Next.js 14 + @solana/wallet-adapter
- **存储：** IPFS (Pinata)
- **数据库：** PostgreSQL (可选)
- **推理：** 本地LLM (Llama 3/Mistral) 或多节点模拟

## 项目结构

```
daollm/
├── programs/          # Solana程序（Rust智能合约）
│   └── daollm/
├── tests/            # Anchor测试
├── backend/          # Rust后端服务（Axum）
├── frontend/         # Next.js前端应用
├── scripts/          # 部署和工具脚本
└── docs/             # 文档
```

## 快速开始

### 环境要求

- Rust 1.70+
- Solana CLI 1.18+
- Anchor 0.29+
- Node.js 18+
- Cargo (Rust包管理器)

### 安装步骤

1. **安装Rust**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **安装Solana CLI**
```bash
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
```

3. **安装Anchor**
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

4. **克隆项目**
```bash
git clone <repository-url>
cd daollm
```

5. **运行设置脚本**
```bash
chmod +x scripts/setup.sh
./scripts/setup.sh
```

### 开发

#### 构建Solana程序
```bash
anchor build
```

#### 运行测试
```bash
anchor test
# 或
chmod +x scripts/test.sh
./scripts/test.sh
```

#### 部署程序
```bash
anchor deploy
# 或
chmod +x scripts/deploy.sh
./scripts/deploy.sh devnet
```

#### 启动后端
```bash
cd backend
cargo run
```

#### 启动前端
```bash
cd frontend
npm install
npm run dev
```

## 环境配置

复制 `env.example` 到 `.env` 并配置：

```bash
# Solana配置
SOLANA_NETWORK=devnet
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=<部署后的程序ID>

# IPFS配置
PINATA_API_KEY=<你的Pinata API Key>
PINATA_SECRET_KEY=<你的Pinata Secret Key>

# 后端配置
API_PORT=8000
```

## 功能特性

### MVP核心功能

- ✅ 提案提交和IPFS存储
- ✅ Solana链上记录
- ✅ 推理节点注册和管理
- ✅ 多节点推理分析
- ✅ 结果聚合和展示
- ✅ 前端钱包集成

### 后续迭代（不在MVP范围）

- 模型治理模块
- 分布式训练协调
- 完整奖励分配系统
- Bittensor集成
- 质量保证系统

## 文档

- [架构设计](DAO_PROPOSAL_SYSTEM_DECENTRALIZED_AI.md)
- [实施计划](IMPLEMENTATION_PLAN.md)
- [用户指南](USER_GUIDE.md)
- [MVP状态](MVP_STATUS.md)

## 开发指南

### Solana程序开发

程序位于 `programs/daollm/`，使用Anchor框架：

```bash
cd programs/daollm
anchor build
anchor test
```

### 后端开发

后端使用Rust + Axum：

```bash
cd backend
cargo run
```

API文档：访问 `http://localhost:8000` 查看API端点

### 前端开发

前端使用Next.js：

```bash
cd frontend
npm install
npm run dev
```

访问 `http://localhost:3000` 查看应用

## 测试

### Solana程序测试
```bash
anchor test
```

### 后端测试
```bash
cd backend
cargo test
```

### 集成测试
```bash
./scripts/test.sh
```

## 部署

### 部署到Solana Devnet
```bash
solana config set --url devnet
anchor build
anchor deploy
```

### 部署到Solana Mainnet
```bash
solana config set --url mainnet-beta
anchor build
anchor deploy
```

## 贡献

欢迎提交Issue和Pull Request！

## 许可证

MIT License

## 联系方式

如有问题，请提交Issue或联系开发团队。

---

**状态：** MVP核心功能已完成 ✅  
**最后更新：** 2024年12月
