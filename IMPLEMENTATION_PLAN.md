# 去中心化DAO提案系统实施计划

## 一、项目结构设计

```
daollm/
├── contracts/                    # 智能合约
│   ├── ProposalToken.sol        # 代币合约 ($PROPOSAL)
│   ├── DataContribution.sol     # 数据贡献合约
│   ├── InferenceNetwork.sol     # 推理网络合约
│   ├── ModelGovernance.sol      # 模型治理合约
│   └── RewardDistribution.sol   # 奖励分配合约
│
├── backend/                     # 后端服务
│   ├── api/                     # FastAPI服务
│   │   ├── routes/
│   │   │   ├── proposals.py     # 提案API
│   │   │   ├── data_contribution.py  # 数据贡献API
│   │   │   └── inference.py     # 推理API
│   │   ├── models/              # 数据模型
│   │   ├── services/            # 业务逻辑
│   │   │   ├── ipfs_service.py  # IPFS服务
│   │   │   ├── llm_service.py   # LLM服务（去中心化）
│   │   │   └── blockchain_service.py  # 区块链服务
│   │   └── main.py              # FastAPI入口
│   │
│   ├── workers/                 # 后台任务
│   │   ├── inference_worker.py  # 推理任务处理
│   │   ├── training_coordinator.py  # 训练协调器
│   │   └── reward_distributor.py   # 奖励分发器
│   │
│   ├── ml/                      # 机器学习模块
│   │   ├── models/              # 模型定义
│   │   ├── training/            # 训练脚本
│   │   │   ├── federated_learning.py  # 联邦学习
│   │   │   └── bittensor_integration.py  # Bittensor集成
│   │   └── inference/           # 推理服务
│   │       └── decentralized_inference.py  # 去中心化推理
│   │
│   └── config/                  # 配置文件
│       ├── settings.py
│       └── chains.py            # 多链配置
│
├── frontend/                    # 前端应用
│   ├── src/
│   │   ├── components/          # React组件
│   │   │   ├── ProposalForm.tsx
│   │   │   ├── ProposalCard.tsx
│   │   │   ├── DataContribution.tsx
│   │   │   └── NodeDashboard.tsx
│   │   ├── hooks/               # React Hooks
│   │   │   ├── useWeb3.ts      # Web3集成
│   │   │   └── useProposals.ts  # 提案数据
│   │   ├── pages/              # 页面
│   │   │   ├── Home.tsx
│   │   │   ├── SubmitProposal.tsx
│   │   │   ├── ContributeData.tsx
│   │   │   └── NodeManagement.tsx
│   │   └── App.tsx
│   ├── package.json
│   └── tsconfig.json
│
├── scripts/                     # 部署和工具脚本
│   ├── deploy.js                # 合约部署
│   ├── setup_nodes.js           # 节点设置
│   └── test_inference.js        # 推理测试
│
├── tests/                       # 测试
│   ├── contracts/              # 合约测试
│   ├── backend/                 # 后端测试
│   └── integration/            # 集成测试
│
├── docs/                        # 文档
│   ├── api/                    # API文档
│   └── deployment/             # 部署文档
│
├── docker/                      # Docker配置
│   ├── Dockerfile.backend
│   ├── Dockerfile.frontend
│   └── docker-compose.yml
│
├── .env.example                 # 环境变量示例
├── hardhat.config.js           # Hardhat配置
├── package.json                # 根package.json
└── README.md
```

---

## 二、技术栈选择

### 2.1 区块链层
- **智能合约语言：** Solidity 0.8.20+
- **开发框架：** Hardhat
- **测试框架：** Hardhat + Chai
- **部署网络：** Base Sepolia (测试) → Base Mainnet (生产)
- **Web3库：** ethers.js v6

### 2.2 后端服务
- **语言：** Python 3.11+
- **Web框架：** FastAPI
- **异步任务：** Celery + Redis
- **数据库：** PostgreSQL 15+
- **缓存：** Redis 7+
- **消息队列：** RabbitMQ (可选，Celery已包含)

### 2.3 AI/ML层
- **联邦学习：** PySyft (或 TensorFlow Federated)
- **模型框架：** Transformers (Hugging Face)
- **Bittensor集成：** Bittensor SDK
- **NLP处理：** spaCy, NLTK

### 2.4 去中心化存储
- **IPFS客户端：** ipfs-http-client
- **IPFS服务：** Pinata (开发) / 自建节点 (生产)
- **文件存储：** IPFS + Filecoin

### 2.5 前端
- **框架：** Next.js 14+ (App Router)
- **UI库：** Tailwind CSS + shadcn/ui
- **Web3：** wagmi + viem
- **状态管理：** Zustand
- **可视化：** Recharts

### 2.6 基础设施
- **容器化：** Docker + Docker Compose
- **CI/CD：** GitHub Actions
- **监控：** Prometheus + Grafana (可选)
- **日志：** 结构化日志 (Python logging)

---

## 三、实施步骤（分阶段）

### Phase 1: 基础设施搭建 (Week 1-2)

#### 1.1 项目初始化
- [ ] 创建项目目录结构
- [ ] 初始化Git仓库
- [ ] 配置开发环境 (Node.js, Python, Hardhat)
- [ ] 设置环境变量模板

#### 1.2 智能合约基础
- [ ] 设置Hardhat项目
- [ ] 创建ProposalToken代币合约
- [ ] 编写合约测试
- [ ] 部署到Base Sepolia测试网

#### 1.3 后端基础框架
- [ ] 创建FastAPI项目
- [ ] 配置数据库连接 (PostgreSQL)
- [ ] 配置Redis缓存
- [ ] 设置CORS和中间件

#### 1.4 前端基础框架
- [ ] 创建Next.js项目
- [ ] 配置Tailwind CSS
- [ ] 集成wagmi和Web3连接
- [ ] 创建基础布局组件

---

### Phase 2: 核心功能 - 数据贡献模块 (Week 3-4)

#### 2.1 IPFS集成
- [ ] 实现IPFS客户端服务
- [ ] 实现文件上传功能
- [ ] 实现内容哈希计算
- [ ] 测试IPFS存储和检索

#### 2.2 数据贡献智能合约
- [ ] 创建DataContribution.sol合约
- [ ] 实现数据提交功能（存储IPFS哈希）
- [ ] 实现标注提交功能
- [ ] 实现奖励分配逻辑
- [ ] 编写测试用例

#### 2.3 数据贡献后端API
- [ ] 创建提案提交API
- [ ] 创建数据标注API
- [ ] 实现IPFS存储逻辑
- [ ] 实现链上记录逻辑
- [ ] 实现奖励计算逻辑

#### 2.4 数据贡献前端
- [ ] 创建提案提交表单
- [ ] 创建数据标注界面
- [ ] 实现文件上传功能
- [ ] 显示贡献历史和奖励

---

### Phase 3: 去中心化推理网络 (Week 5-7)

#### 3.1 推理节点合约
- [ ] 创建InferenceNetwork.sol合约
- [ ] 实现节点注册功能（需要质押）
- [ ] 实现推理任务分发逻辑
- [ ] 实现结果共识机制
- [ ] 实现节点评分系统

#### 3.2 推理服务后端
- [ ] 创建推理节点注册API
- [ ] 实现推理任务分发逻辑
- [ ] 实现多节点推理协调
- [ ] 实现结果聚合算法（投票/加权平均）
- [ ] 实现节点质量评分

#### 3.3 LLM集成（去中心化）
- [ ] 集成本地LLM (Llama 3 / Mistral)
- [ ] 或集成Bittensor推理子网
- [ ] 实现推理请求封装
- [ ] 实现结果格式化

#### 3.4 推理前端
- [ ] 创建节点注册界面
- [ ] 创建推理请求界面
- [ ] 显示推理结果和节点信息
- [ ] 显示节点排行榜

---

### Phase 4: 模型治理模块 (Week 8-9)

#### 4.1 治理合约
- [ ] 创建ModelGovernance.sol合约
- [ ] 基于OpenZeppelin Governor
- [ ] 实现模型参数投票
- [ ] 实现模型升级提案
- [ ] 实现时间锁机制

#### 4.2 治理后端
- [ ] 创建治理提案API
- [ ] 实现投票逻辑
- [ ] 实现提案状态追踪
- [ ] 实现治理事件监听

#### 4.3 治理前端
- [ ] 创建治理提案界面
- [ ] 创建投票界面
- [ ] 显示提案历史和状态
- [ ] 显示模型版本信息

---

### Phase 5: 分布式训练协调 (Week 10-12)

#### 5.1 训练协调合约
- [ ] 创建训练任务发布合约
- [ ] 实现节点任务领取逻辑
- [ ] 实现训练结果提交
- [ ] 实现梯度聚合验证

#### 5.2 联邦学习框架
- [ ] 集成PySyft或TensorFlow Federated
- [ ] 实现训练任务分解
- [ ] 实现梯度收集和聚合
- [ ] 实现模型检查点管理

#### 5.3 Bittensor集成（可选）
- [ ] 研究Bittensor子网开发
- [ ] 创建提案分析专用子网
- [ ] 实现节点注册和评分
- [ ] 实现奖励分配

#### 5.4 训练协调后端
- [ ] 创建训练任务管理API
- [ ] 实现训练进度追踪
- [ ] 实现模型版本管理
- [ ] 实现训练节点管理

---

### Phase 6: 奖励分配系统 (Week 13-14)

#### 6.1 奖励分配合约
- [ ] 创建RewardDistribution.sol合约
- [ ] 实现数据贡献者奖励
- [ ] 实现训练节点奖励
- [ ] 实现推理节点奖励
- [ ] 实现自动分配逻辑

#### 6.2 奖励分发后端
- [ ] 创建奖励计算服务
- [ ] 实现奖励分发定时任务
- [ ] 实现奖励历史记录
- [ ] 实现异常处理

#### 6.3 奖励前端
- [ ] 创建奖励查询界面
- [ ] 显示奖励历史和统计
- [ ] 显示代币余额和质押信息

---

### Phase 7: 质量保证系统 (Week 15-16)

#### 7.1 质量验证合约
- [ ] 实现多节点结果验证
- [ ] 实现异常检测逻辑
- [ ] 实现节点信誉系统
- [ ] 实现自动惩罚机制

#### 7.2 质量保证后端
- [ ] 实现结果验证算法
- [ ] 实现异常检测服务
- [ ] 实现信誉评分计算
- [ ] 实现惩罚执行逻辑

---

### Phase 8: 前端完善和优化 (Week 17-18)

#### 8.1 用户体验优化
- [ ] 完善UI/UX设计
- [ ] 添加加载状态和错误处理
- [ ] 实现响应式设计
- [ ] 添加数据可视化

#### 8.2 性能优化
- [ ] 实现前端缓存
- [ ] 优化API调用
- [ ] 实现代码分割
- [ ] 优化图片和资源

---

### Phase 9: 测试和部署 (Week 19-20)

#### 9.1 测试
- [ ] 编写单元测试
- [ ] 编写集成测试
- [ ] 编写端到端测试
- [ ] 进行安全审计

#### 9.2 部署
- [ ] 部署智能合约到Base Mainnet
- [ ] 部署后端服务
- [ ] 部署前端应用
- [ ] 配置监控和日志

---

## 四、关键技术实现细节

### 4.1 智能合约架构

#### ProposalToken.sol (ERC20代币)
```solidity
// 核心功能：
// - 标准ERC20代币
// - 治理投票权（基于代币数量）
// - 质押功能
// - 奖励分配
```

#### DataContribution.sol
```solidity
// 核心功能：
// - submitProposal(ipfsHash) - 提交提案数据
// - submitAnnotation(proposalId, annotation) - 提交标注
// - verifyAnnotation(annotationId) - 验证标注
// - claimReward(contributionId) - 领取奖励
```

#### InferenceNetwork.sol
```solidity
// 核心功能：
// - registerNode(stakeAmount) - 注册推理节点
// - submitInference(taskId, result) - 提交推理结果
// - aggregateResults(taskId) - 聚合多个节点结果
// - rateNode(nodeId, score) - 评分节点
```

### 4.2 后端服务架构

#### IPFS服务实现
```python
# services/ipfs_service.py
class IPFSService:
    def upload_file(self, file_path: str) -> str:
        # 上传文件到IPFS，返回内容哈希
        
    def upload_json(self, data: dict) -> str:
        # 上传JSON数据到IPFS
        
    def retrieve(self, ipfs_hash: str) -> bytes:
        # 从IPFS检索数据
```

#### 去中心化推理服务
```python
# services/decentralized_inference.py
class DecentralizedInferenceService:
    def request_inference(self, proposal_text: str) -> dict:
        # 1. 选择多个推理节点
        # 2. 分发推理任务
        # 3. 收集结果
        # 4. 共识聚合
        # 5. 返回最终结果
```

#### 联邦学习协调器
```python
# workers/training_coordinator.py
class TrainingCoordinator:
    def distribute_training_task(self, model_config: dict):
        # 1. 发布训练任务到链上
        # 2. 节点领取任务
        # 3. 收集梯度更新
        # 4. 聚合梯度
        # 5. 更新全局模型
```

### 4.3 前端Web3集成

#### Web3连接和合约交互
```typescript
// hooks/useWeb3.ts
export function useWeb3() {
  // 连接钱包
  // 获取合约实例
  // 监听链上事件
  // 发送交易
}
```

#### 提案管理
```typescript
// hooks/useProposals.ts
export function useProposals() {
  // 提交提案
  // 查询提案列表
  // 获取提案详情
  // 监听提案状态变化
}
```

---

## 五、环境配置

### 5.1 环境变量 (.env.example)

```bash
# 区块链配置
PRIVATE_KEY=your_private_key
RPC_URL_BASE_SEPOLIA=https://sepolia.base.org
RPC_URL_BASE_MAINNET=https://mainnet.base.org

# 合约地址（部署后更新）
PROPOSAL_TOKEN_ADDRESS=
DATA_CONTRIBUTION_ADDRESS=
INFERENCE_NETWORK_ADDRESS=
MODEL_GOVERNANCE_ADDRESS=

# IPFS配置
IPFS_API_URL=http://localhost:5001
PINATA_API_KEY=
PINATA_SECRET_KEY=

# 数据库配置
DATABASE_URL=postgresql://user:password@localhost:5432/daollm
REDIS_URL=redis://localhost:6379

# LLM配置（去中心化）
BITTORNET_SUBNET_ID=
LOCAL_LLM_URL=http://localhost:8000  # 本地LLM服务

# 后端配置
API_PORT=8000
CELERY_BROKER_URL=redis://localhost:6379/0

# 前端配置
NEXT_PUBLIC_CHAIN_ID=84532  # Base Sepolia
NEXT_PUBLIC_RPC_URL=https://sepolia.base.org
```

---

## 六、开发工具和脚本

### 6.1 部署脚本

```javascript
// scripts/deploy.js
// - 部署所有智能合约
// - 初始化合约参数
// - 验证合约
// - 保存部署地址
```

### 6.2 测试脚本

```javascript
// scripts/test_inference.js
// - 测试推理网络功能
// - 模拟多节点推理
// - 验证结果共识
```

---

## 七、安全考虑

### 7.1 智能合约安全
- [ ] 使用OpenZeppelin标准库
- [ ] 进行安全审计
- [ ] 实现重入攻击防护
- [ ] 实现访问控制
- [ ] 实现时间锁机制

### 7.2 后端安全
- [ ] API认证和授权
- [ ] 输入验证和清理
- [ ] SQL注入防护
- [ ] 速率限制
- [ ] CORS配置

### 7.3 前端安全
- [ ] 钱包连接验证
- [ ] 交易签名验证
- [ ] XSS防护
- [ ] 敏感信息保护

---

## 八、监控和运维

### 8.1 监控指标
- 智能合约Gas使用
- API响应时间
- 推理节点在线率
- 代币分配情况
- 系统错误率

### 8.2 日志管理
- 结构化日志记录
- 错误追踪和告警
- 链上事件监听
- 用户行为分析

---

## 九、文档要求

### 9.1 技术文档
- [ ] API文档 (OpenAPI/Swagger)
- [ ] 智能合约文档
- [ ] 部署指南
- [ ] 开发指南

### 9.2 用户文档
- [ ] 用户使用手册
- [ ] 节点运行指南
- [ ] 代币经济说明
- [ ] 常见问题FAQ

---

## 十、里程碑和交付物

### Milestone 1: MVP (Week 4)
- ✅ 基础智能合约部署
- ✅ 数据贡献功能
- ✅ 简单前端界面

### Milestone 2: 推理网络 (Week 7)
- ✅ 去中心化推理功能
- ✅ 节点注册和管理
- ✅ 结果共识机制

### Milestone 3: 完整系统 (Week 14)
- ✅ 模型治理功能
- ✅ 奖励分配系统
- ✅ 完整前端应用

### Milestone 4: 生产就绪 (Week 20)
- ✅ 完整测试覆盖
- ✅ 安全审计通过
- ✅ 生产环境部署

---

**计划版本：** v1.0  
**创建日期：** 2024年  
**预计总时长：** 20周（5个月）

