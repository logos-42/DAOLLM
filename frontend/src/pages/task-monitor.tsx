import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'
import { useState, useEffect } from 'react'
import { useRouter } from 'next/router'

// Types
type TaskStatus = 'Pending' | 'Reasoning' | 'Verifying' | 'WaitingProof' | 'ReadyForExecution' | 'Disputed' | 'Finalized' | 'Cancelled'

interface TaskLog {
  timestamp: string
  stage: string
  message: string
  nodeId?: string
  details?: Record<string, unknown>
}

interface TaskDetail {
  taskId: string
  intent: string
  status: TaskStatus
  complexityScore: number
  workflow: string
  criticality: string
  stakePool: number
  verificationScore: number
  requiresProof: boolean
  proofStatus?: 'pending' | 'generating' | 'verified' | 'failed'
  reasoningNodes: string[]
  verificationNodes: string[]
  cacheHitUsed: boolean
  ipfsResult?: string
  logs: TaskLog[]
  createdAt: string
  estimatedCompletion?: string
}

// Mock task data
const mockTask: TaskDetail = {
  taskId: '3',
  intent: '评估MakerDAO治理提案#847的经济影响',
  status: 'Verifying',
  complexityScore: 850,
  workflow: 'ConsensusGuarded',
  criticality: 'MissionCritical',
  stakePool: 20_000_000_000,
  verificationScore: 0,
  requiresProof: true,
  proofStatus: 'pending',
  reasoningNodes: ['7xKX...3fYz', '9aBC...xY12', '3dEF...pQ78'],
  verificationNodes: ['8bCD...4gHi'],
  cacheHitUsed: false,
  createdAt: '2024-12-03T09:00:00Z',
  estimatedCompletion: '2024-12-03T09:15:00Z',
  logs: [
    {
      timestamp: '2024-12-03T09:00:05Z',
      stage: 'created',
      message: '任务创建成功',
      details: { complexity: 850, workflow: 'ConsensusGuarded' }
    },
    {
      timestamp: '2024-12-03T09:00:15Z',
      stage: 'assigned',
      message: '任务分配给 3 个推理节点',
      nodeId: '7xKX...3fYz'
    },
    {
      timestamp: '2024-12-03T09:01:30Z',
      stage: 'reasoning',
      message: '节点 7xKX...3fYz 开始推理',
      nodeId: '7xKX...3fYz'
    },
    {
      timestamp: '2024-12-03T09:03:45Z',
      stage: 'reasoning',
      message: '节点 9aBC...xY12 开始推理',
      nodeId: '9aBC...xY12'
    },
    {
      timestamp: '2024-12-03T09:05:20Z',
      stage: 'reasoning_complete',
      message: '节点 7xKX...3fYz 推理完成，置信度 92%',
      nodeId: '7xKX...3fYz',
      details: { confidence: 92, latency: 230000 }
    },
    {
      timestamp: '2024-12-03T09:06:15Z',
      stage: 'reasoning_complete',
      message: '节点 9aBC...xY12 推理完成，置信度 89%',
      nodeId: '9aBC...xY12',
      details: { confidence: 89, latency: 150000 }
    },
    {
      timestamp: '2024-12-03T09:06:30Z',
      stage: 'verifying',
      message: '进入验证阶段，启动知识图谱校验',
    },
    {
      timestamp: '2024-12-03T09:07:00Z',
      stage: 'kg_check',
      message: '知识图谱匹配 47 个三元组，一致性 94%',
      details: { triplets: 47, consistency: 94 }
    },
    {
      timestamp: '2024-12-03T09:07:30Z',
      stage: 'nli_check',
      message: '事实一致性检查进行中...',
    },
  ]
}

const stageColors: Record<string, string> = {
  created: 'bg-gray-500',
  assigned: 'bg-blue-500',
  reasoning: 'bg-indigo-500',
  reasoning_complete: 'bg-purple-500',
  verifying: 'bg-yellow-500',
  kg_check: 'bg-orange-500',
  nli_check: 'bg-amber-500',
  proof_generating: 'bg-pink-500',
  proof_verified: 'bg-green-500',
  finalized: 'bg-emerald-500',
  error: 'bg-red-500',
}

const pipelineStages = [
  { key: 'Pending', label: '等待中', icon: '⏳' },
  { key: 'Reasoning', label: '推理中', icon: '🧠' },
  { key: 'Verifying', label: '验证中', icon: '✅' },
  { key: 'WaitingProof', label: 'ZK证明', icon: '🔐' },
  { key: 'ReadyForExecution', label: '待执行', icon: '⚡' },
  { key: 'Finalized', label: '已完成', icon: '🎉' },
]

export default function TaskMonitor() {
  const { connected } = useWallet()
  const router = useRouter()
  const { id } = router.query
  
  const [task, setTask] = useState<TaskDetail | null>(null)
  const [isLive, setIsLive] = useState(true)

  useEffect(() => {
    // In production, fetch from backend
    setTask(mockTask)
  }, [id])

  // Simulate live updates
  useEffect(() => {
    if (!isLive || !task) return

    const interval = setInterval(() => {
      // Simulate new log entry
      const newLog: TaskLog = {
        timestamp: new Date().toISOString(),
        stage: 'nli_check',
        message: `NLI验证进度: ${Math.floor(Math.random() * 30) + 70}%`,
      }
      setTask(prev => prev ? {
        ...prev,
        logs: [...prev.logs, newLog]
      } : null)
    }, 5000)

    return () => clearInterval(interval)
  }, [isLive, task])

  const formatDate = (iso: string) => new Date(iso).toLocaleString('zh-CN')
  const formatTime = (iso: string) => new Date(iso).toLocaleTimeString('zh-CN')
  const formatSol = (lamports: number) => (lamports / 1e9).toFixed(2)

  const getCurrentStageIndex = (status: TaskStatus) => {
    const index = pipelineStages.findIndex(s => s.key === status)
    return index >= 0 ? index : 0
  }

  if (!task) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900 flex items-center justify-center">
        <div className="text-white text-xl">加载中...</div>
      </div>
    )
  }

  const currentStageIndex = getCurrentStageIndex(task.status)

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-blue-900 to-slate-900">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <header className="flex justify-between items-center mb-8">
          <div className="flex items-center gap-4">
            <Link href="/tro-tasks">
              <span className="text-2xl font-bold text-white cursor-pointer hover:text-blue-300 transition">
                ← 任务列表
              </span>
            </Link>
            <h1 className="text-3xl font-bold text-white">任务监控</h1>
            <span className="text-gray-400">#{task.taskId}</span>
          </div>
          <div className="flex items-center gap-4">
            <button
              onClick={() => setIsLive(!isLive)}
              className={`flex items-center gap-2 px-4 py-2 rounded-lg transition ${
                isLive ? 'bg-green-500/20 text-green-400 border border-green-500/50' : 'bg-white/10 text-gray-400'
              }`}
            >
              <span className={`w-2 h-2 rounded-full ${isLive ? 'bg-green-400 animate-pulse' : 'bg-gray-500'}`} />
              {isLive ? '实时更新' : '暂停更新'}
            </button>
            <WalletMultiButton />
          </div>
        </header>

        {/* Task Intent */}
        <div className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20 mb-6">
          <div className="text-sm text-gray-400 mb-2">任务意图</div>
          <h2 className="text-xl font-semibold text-white">{task.intent}</h2>
          <div className="flex items-center gap-4 mt-4 text-sm text-gray-400">
            <span>复杂度: {task.complexityScore}</span>
            <span>•</span>
            <span>工作流: {task.workflow}</span>
            <span>•</span>
            <span>优先级: {task.criticality}</span>
            <span>•</span>
            <span>质押池: {formatSol(task.stakePool)} SOL</span>
          </div>
        </div>

        {/* Pipeline Progress */}
        <div className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20 mb-6">
          <div className="text-sm text-gray-400 mb-4">TRO 流水线进度</div>
          <div className="flex items-center justify-between">
            {pipelineStages.map((stage, index) => (
              <div key={stage.key} className="flex items-center">
                <div className="flex flex-col items-center">
                  <div className={`w-12 h-12 rounded-full flex items-center justify-center text-2xl ${
                    index < currentStageIndex ? 'bg-green-500/30' :
                    index === currentStageIndex ? 'bg-blue-500/30 animate-pulse' :
                    'bg-white/10'
                  }`}>
                    {stage.icon}
                  </div>
                  <div className={`text-xs mt-2 ${
                    index <= currentStageIndex ? 'text-white' : 'text-gray-500'
                  }`}>
                    {stage.label}
                  </div>
                </div>
                {index < pipelineStages.length - 1 && (
                  <div className={`w-20 h-1 mx-2 rounded ${
                    index < currentStageIndex ? 'bg-green-500' : 'bg-white/20'
                  }`} />
                )}
              </div>
            ))}
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Main Content - Logs */}
          <div className="lg:col-span-2 space-y-6">
            {/* Real-time Logs */}
            <div className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20">
              <div className="flex justify-between items-center mb-4">
                <div className="text-sm text-gray-400">执行日志</div>
                <span className="text-xs text-gray-500">{task.logs.length} 条记录</span>
              </div>
              <div className="space-y-3 max-h-96 overflow-y-auto pr-2">
                {task.logs.map((log, index) => (
                  <div key={index} className="flex gap-3">
                    <div className="flex flex-col items-center">
                      <div className={`w-3 h-3 rounded-full ${stageColors[log.stage] || 'bg-gray-500'}`} />
                      {index < task.logs.length - 1 && (
                        <div className="w-0.5 h-full bg-white/20 mt-1" />
                      )}
                    </div>
                    <div className="flex-1 pb-4">
                      <div className="flex justify-between items-start">
                        <div className="text-white text-sm">{log.message}</div>
                        <div className="text-xs text-gray-500">{formatTime(log.timestamp)}</div>
                      </div>
                      {log.nodeId && (
                        <div className="text-xs text-gray-500 mt-1">节点: {log.nodeId}</div>
                      )}
                      {log.details && (
                        <div className="text-xs text-gray-600 mt-1 font-mono">
                          {JSON.stringify(log.details)}
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Verification Details */}
            {task.status === 'Verifying' && (
              <div className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20">
                <div className="text-sm text-gray-400 mb-4">验证详情</div>
                <div className="grid grid-cols-3 gap-4">
                  <div className="bg-white/5 rounded-lg p-4">
                    <div className="text-xs text-gray-400 mb-1">语义相似度</div>
                    <div className="text-lg font-semibold text-white">92.5%</div>
                    <div className="h-2 bg-white/10 rounded-full mt-2">
                      <div className="h-full bg-blue-500 rounded-full" style={{ width: '92.5%' }} />
                    </div>
                  </div>
                  <div className="bg-white/5 rounded-lg p-4">
                    <div className="text-xs text-gray-400 mb-1">事实一致性</div>
                    <div className="text-lg font-semibold text-white">89.0%</div>
                    <div className="h-2 bg-white/10 rounded-full mt-2">
                      <div className="h-full bg-green-500 rounded-full" style={{ width: '89%' }} />
                    </div>
                  </div>
                  <div className="bg-white/5 rounded-lg p-4">
                    <div className="text-xs text-gray-400 mb-1">KG匹配度</div>
                    <div className="text-lg font-semibold text-white">94.0%</div>
                    <div className="h-2 bg-white/10 rounded-full mt-2">
                      <div className="h-full bg-purple-500 rounded-full" style={{ width: '94%' }} />
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Task Info */}
            <div className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20">
              <div className="text-sm text-gray-400 mb-4">任务信息</div>
              <div className="space-y-3">
                <div className="flex justify-between">
                  <span className="text-gray-400">状态</span>
                  <span className="text-white font-medium">{task.status}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">缓存命中</span>
                  <span className={task.cacheHitUsed ? 'text-green-400' : 'text-gray-500'}>
                    {task.cacheHitUsed ? '是' : '否'}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-400">需要ZK证明</span>
                  <span className={task.requiresProof ? 'text-purple-400' : 'text-gray-500'}>
                    {task.requiresProof ? '是' : '否'}
                  </span>
                </div>
                {task.requiresProof && (
                  <div className="flex justify-between">
                    <span className="text-gray-400">证明状态</span>
                    <span className={
                      task.proofStatus === 'verified' ? 'text-green-400' :
                      task.proofStatus === 'generating' ? 'text-yellow-400' :
                      task.proofStatus === 'failed' ? 'text-red-400' : 'text-gray-500'
                    }>
                      {task.proofStatus === 'verified' ? '已验证' :
                       task.proofStatus === 'generating' ? '生成中' :
                       task.proofStatus === 'failed' ? '失败' : '待生成'}
                    </span>
                  </div>
                )}
                <div className="flex justify-between">
                  <span className="text-gray-400">创建时间</span>
                  <span className="text-white">{formatDate(task.createdAt)}</span>
                </div>
                {task.estimatedCompletion && (
                  <div className="flex justify-between">
                    <span className="text-gray-400">预计完成</span>
                    <span className="text-white">{formatDate(task.estimatedCompletion)}</span>
                  </div>
                )}
              </div>
            </div>

            {/* Participating Nodes */}
            <div className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20">
              <div className="text-sm text-gray-400 mb-4">参与节点</div>
              <div className="space-y-3">
                <div>
                  <div className="text-xs text-gray-500 mb-2">推理节点 ({task.reasoningNodes.length})</div>
                  <div className="space-y-2">
                    {task.reasoningNodes.map((node, i) => (
                      <div key={i} className="flex items-center gap-2">
                        <div className="w-2 h-2 rounded-full bg-blue-500" />
                        <span className="text-white font-mono text-sm">{node}</span>
                      </div>
                    ))}
                  </div>
                </div>
                <div>
                  <div className="text-xs text-gray-500 mb-2">验证节点 ({task.verificationNodes.length})</div>
                  <div className="space-y-2">
                    {task.verificationNodes.map((node, i) => (
                      <div key={i} className="flex items-center gap-2">
                        <div className="w-2 h-2 rounded-full bg-green-500" />
                        <span className="text-white font-mono text-sm">{node}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>

            {/* Actions */}
            {connected && task.status === 'ReadyForExecution' && (
              <div className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20">
                <div className="text-sm text-gray-400 mb-4">操作</div>
                <div className="space-y-2">
                  <Link href={`/challenge?id=${task.taskId}`}>
                    <button className="w-full bg-red-500/20 border border-red-500/50 text-red-400 py-2 rounded-lg font-medium hover:bg-red-500/30 transition">
                      发起挑战
                    </button>
                  </Link>
                  <button className="w-full bg-green-500/20 border border-green-500/50 text-green-400 py-2 rounded-lg font-medium hover:bg-green-500/30 transition">
                    确认执行
                  </button>
                </div>
              </div>
            )}

            {/* Result Preview */}
            {task.ipfsResult && (
              <div className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20">
                <div className="text-sm text-gray-400 mb-4">结果预览</div>
                <a 
                  href={`https://gateway.pinata.cloud/ipfs/${task.ipfsResult}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-400 hover:text-blue-300 text-sm"
                >
                  查看 IPFS 结果 →
                </a>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}

