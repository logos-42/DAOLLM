import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'
import { useState, useEffect } from 'react'

// Task types matching on-chain enums
type TaskType = 'SimpleQA' | 'ComplexReasoning' | 'MultiStep' | 'DataAnalysis'
type WorkflowClass = 'FastRealtime' | 'Balanced' | 'DeepReasoning' | 'ConsensusGuarded'
type TaskCriticality = 'Low' | 'Standard' | 'High' | 'MissionCritical'
type TaskStatus = 'Pending' | 'Reasoning' | 'Verifying' | 'WaitingProof' | 'ReadyForExecution' | 'Disputed' | 'Finalized' | 'Cancelled'

interface TroTask {
  taskId: string
  intent: string
  taskType: TaskType
  workflow: WorkflowClass
  criticality: TaskCriticality
  status: TaskStatus
  complexityScore: number
  stakePool: number
  verificationScore: number
  requiresProof: boolean
  cacheHitUsed: boolean
  ipfsResult?: string
  createdAt: string
  challengePeriodEnd?: string
}

// Mock data for demo
const mockTasks: TroTask[] = [
  {
    taskId: '1',
    intent: '分析Uniswap V4的核心创新点和潜在风险',
    taskType: 'ComplexReasoning',
    workflow: 'DeepReasoning',
    criticality: 'High',
    status: 'Finalized',
    complexityScore: 720,
    stakePool: 5000000000,
    verificationScore: 9500,
    requiresProof: true,
    cacheHitUsed: false,
    ipfsResult: 'QmX7bVbZ...',
    createdAt: '2024-12-01T10:30:00Z',
    challengePeriodEnd: '2024-12-03T10:30:00Z',
  },
  {
    taskId: '2',
    intent: 'ETH当前价格是多少？',
    taskType: 'SimpleQA',
    workflow: 'FastRealtime',
    criticality: 'Low',
    status: 'Finalized',
    complexityScore: 80,
    stakePool: 100000000,
    verificationScore: 9900,
    requiresProof: false,
    cacheHitUsed: true,
    ipfsResult: 'QmY8cWaA...',
    createdAt: '2024-12-02T15:45:00Z',
  },
  {
    taskId: '3',
    intent: '评估MakerDAO治理提案#847的经济影响',
    taskType: 'DataAnalysis',
    workflow: 'ConsensusGuarded',
    criticality: 'MissionCritical',
    status: 'Verifying',
    complexityScore: 850,
    stakePool: 20000000000,
    verificationScore: 0,
    requiresProof: true,
    cacheHitUsed: false,
    createdAt: '2024-12-03T09:00:00Z',
  },
]

const statusColors: Record<TaskStatus, string> = {
  Pending: 'bg-gray-100 text-gray-800',
  Reasoning: 'bg-blue-100 text-blue-800',
  Verifying: 'bg-yellow-100 text-yellow-800',
  WaitingProof: 'bg-purple-100 text-purple-800',
  ReadyForExecution: 'bg-green-100 text-green-800',
  Disputed: 'bg-red-100 text-red-800',
  Finalized: 'bg-emerald-100 text-emerald-800',
  Cancelled: 'bg-gray-300 text-gray-600',
}

const statusLabels: Record<TaskStatus, string> = {
  Pending: '等待中',
  Reasoning: '推理中',
  Verifying: '验证中',
  WaitingProof: '等待证明',
  ReadyForExecution: '待执行',
  Disputed: '争议中',
  Finalized: '已完成',
  Cancelled: '已取消',
}

const criticalityColors: Record<TaskCriticality, string> = {
  Low: 'text-green-600',
  Standard: 'text-blue-600',
  High: 'text-orange-600',
  MissionCritical: 'text-red-600',
}

export default function TroTasks() {
  const { connected, publicKey } = useWallet()
  const [tasks, setTasks] = useState<TroTask[]>(mockTasks)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [filter, setFilter] = useState<TaskStatus | 'all'>('all')

  // Form state
  const [newIntent, setNewIntent] = useState('')
  const [newTaskType, setNewTaskType] = useState<TaskType>('SimpleQA')
  const [newCriticality, setNewCriticality] = useState<TaskCriticality>('Standard')
  const [estimatedComplexity, setEstimatedComplexity] = useState(0)
  const [estimatedCost, setEstimatedCost] = useState(0)

  // Estimate complexity and cost based on input
  useEffect(() => {
    const wordCount = newIntent.split(/\s+/).filter(Boolean).length
    const hasAnalysis = /分析|评估|比较|研究/.test(newIntent)
    const hasReasoning = /为什么|如何|推理|逻辑/.test(newIntent)
    
    let complexity = wordCount * 10
    if (hasAnalysis) complexity += 200
    if (hasReasoning) complexity += 300
    if (newTaskType === 'ComplexReasoning') complexity *= 1.5
    if (newTaskType === 'DataAnalysis') complexity *= 1.8
    if (newCriticality === 'High') complexity *= 1.3
    if (newCriticality === 'MissionCritical') complexity *= 1.5

    setEstimatedComplexity(Math.min(Math.round(complexity), 1000))
    
    // Cost in lamports
    const baseCost = 100000000 // 0.1 SOL
    const complexityCost = complexity * 10000
    const criticalityCost = newCriticality === 'MissionCritical' ? 500000000 : 
                           newCriticality === 'High' ? 200000000 : 0
    setEstimatedCost(baseCost + complexityCost + criticalityCost)
  }, [newIntent, newTaskType, newCriticality])

  const handleCreateTask = async () => {
    if (!connected || !publicKey) {
      alert('请先连接钱包')
      return
    }

    // In production, this would call the Solana program
    const newTask: TroTask = {
      taskId: String(tasks.length + 1),
      intent: newIntent,
      taskType: newTaskType,
      workflow: newCriticality === 'MissionCritical' ? 'ConsensusGuarded' :
                newCriticality === 'High' ? 'DeepReasoning' :
                newTaskType === 'SimpleQA' ? 'FastRealtime' : 'Balanced',
      criticality: newCriticality,
      status: 'Pending',
      complexityScore: estimatedComplexity,
      stakePool: estimatedCost,
      verificationScore: 0,
      requiresProof: newCriticality === 'High' || newCriticality === 'MissionCritical',
      cacheHitUsed: false,
      createdAt: new Date().toISOString(),
    }

    setTasks([newTask, ...tasks])
    setShowCreateModal(false)
    setNewIntent('')
    setNewTaskType('SimpleQA')
    setNewCriticality('Standard')
  }

  const filteredTasks = filter === 'all' ? tasks : tasks.filter(t => t.status === filter)

  const formatSol = (lamports: number) => (lamports / 1e9).toFixed(4)
  const formatDate = (iso: string) => new Date(iso).toLocaleString('zh-CN')

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <header className="flex justify-between items-center mb-8">
          <div className="flex items-center gap-4">
            <Link href="/">
              <span className="text-2xl font-bold text-white cursor-pointer hover:text-purple-300 transition">
                ← DAOLLM
              </span>
            </Link>
            <h1 className="text-3xl font-bold text-white">TRO 任务中心</h1>
          </div>
          <WalletMultiButton />
        </header>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">总任务数</div>
            <div className="text-2xl font-bold text-white">{tasks.length}</div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">进行中</div>
            <div className="text-2xl font-bold text-yellow-400">
              {tasks.filter(t => ['Pending', 'Reasoning', 'Verifying', 'WaitingProof'].includes(t.status)).length}
            </div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">缓存命中率</div>
            <div className="text-2xl font-bold text-green-400">
              {Math.round((tasks.filter(t => t.cacheHitUsed).length / tasks.length) * 100)}%
            </div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">平均验证分数</div>
            <div className="text-2xl font-bold text-purple-400">
              {Math.round(tasks.filter(t => t.verificationScore > 0).reduce((a, t) => a + t.verificationScore, 0) / 
                tasks.filter(t => t.verificationScore > 0).length / 100)}%
            </div>
          </div>
        </div>

        {/* Actions & Filter */}
        <div className="flex justify-between items-center mb-6">
          <div className="flex gap-2">
            <button
              onClick={() => setFilter('all')}
              className={`px-4 py-2 rounded-lg transition ${filter === 'all' ? 'bg-purple-600 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
            >
              全部
            </button>
            <button
              onClick={() => setFilter('Reasoning')}
              className={`px-4 py-2 rounded-lg transition ${filter === 'Reasoning' ? 'bg-blue-600 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
            >
              推理中
            </button>
            <button
              onClick={() => setFilter('Verifying')}
              className={`px-4 py-2 rounded-lg transition ${filter === 'Verifying' ? 'bg-yellow-600 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
            >
              验证中
            </button>
            <button
              onClick={() => setFilter('Disputed')}
              className={`px-4 py-2 rounded-lg transition ${filter === 'Disputed' ? 'bg-red-600 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
            >
              争议中
            </button>
            <button
              onClick={() => setFilter('Finalized')}
              className={`px-4 py-2 rounded-lg transition ${filter === 'Finalized' ? 'bg-emerald-600 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
            >
              已完成
            </button>
          </div>
          <button
            onClick={() => setShowCreateModal(true)}
            className="bg-gradient-to-r from-purple-600 to-pink-600 text-white px-6 py-2 rounded-lg font-semibold hover:opacity-90 transition"
          >
            + 创建任务
          </button>
        </div>

        {/* Task List */}
        <div className="space-y-4">
          {filteredTasks.map(task => (
            <div key={task.taskId} className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20 hover:border-purple-500/50 transition">
              <div className="flex justify-between items-start mb-4">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${statusColors[task.status]}`}>
                      {statusLabels[task.status]}
                    </span>
                    <span className={`text-xs font-medium ${criticalityColors[task.criticality]}`}>
                      {task.criticality === 'MissionCritical' ? '🔴 关键任务' :
                       task.criticality === 'High' ? '🟠 高优先级' :
                       task.criticality === 'Standard' ? '🟢 标准' : '⚪ 低优先级'}
                    </span>
                    {task.requiresProof && (
                      <span className="text-xs text-purple-400">🔐 需要ZK证明</span>
                    )}
                    {task.cacheHitUsed && (
                      <span className="text-xs text-green-400">⚡ 缓存命中</span>
                    )}
                  </div>
                  <h3 className="text-lg font-semibold text-white mb-2">{task.intent}</h3>
                  <div className="flex items-center gap-4 text-sm text-gray-400">
                    <span>类型: {task.taskType}</span>
                    <span>•</span>
                    <span>工作流: {task.workflow}</span>
                    <span>•</span>
                    <span>复杂度: {task.complexityScore}</span>
                  </div>
                </div>
                <div className="text-right">
                  <div className="text-sm text-gray-400">质押池</div>
                  <div className="text-lg font-bold text-white">{formatSol(task.stakePool)} SOL</div>
                </div>
              </div>

              {/* Progress Bar */}
              <div className="mb-4">
                <div className="flex justify-between text-xs text-gray-400 mb-1">
                  <span>任务进度</span>
                  <span>{task.status === 'Finalized' ? '100%' : 
                         task.status === 'ReadyForExecution' ? '90%' :
                         task.status === 'WaitingProof' ? '75%' :
                         task.status === 'Verifying' ? '60%' :
                         task.status === 'Reasoning' ? '30%' : '10%'}</span>
                </div>
                <div className="h-2 bg-white/10 rounded-full overflow-hidden">
                  <div 
                    className={`h-full rounded-full transition-all ${
                      task.status === 'Disputed' ? 'bg-red-500' :
                      task.status === 'Finalized' ? 'bg-green-500' : 'bg-purple-500'
                    }`}
                    style={{ 
                      width: task.status === 'Finalized' ? '100%' : 
                             task.status === 'ReadyForExecution' ? '90%' :
                             task.status === 'WaitingProof' ? '75%' :
                             task.status === 'Verifying' ? '60%' :
                             task.status === 'Reasoning' ? '30%' : '10%'
                    }}
                  />
                </div>
              </div>

              {/* Verification Score */}
              {task.verificationScore > 0 && (
                <div className="flex items-center gap-4 mb-4">
                  <div className="flex-1">
                    <div className="text-xs text-gray-400 mb-1">验证分数</div>
                    <div className="h-3 bg-white/10 rounded-full overflow-hidden">
                      <div 
                        className={`h-full rounded-full ${
                          task.verificationScore >= 9000 ? 'bg-green-500' :
                          task.verificationScore >= 7000 ? 'bg-yellow-500' : 'bg-red-500'
                        }`}
                        style={{ width: `${task.verificationScore / 100}%` }}
                      />
                    </div>
                  </div>
                  <span className="text-lg font-bold text-white">{(task.verificationScore / 100).toFixed(1)}%</span>
                </div>
              )}

              {/* Footer */}
              <div className="flex justify-between items-center pt-4 border-t border-white/10">
                <div className="text-sm text-gray-400">
                  创建于 {formatDate(task.createdAt)}
                  {task.challengePeriodEnd && (
                    <span className="ml-4">争议期至 {formatDate(task.challengePeriodEnd)}</span>
                  )}
                </div>
                <div className="flex gap-2">
                  {task.ipfsResult && (
                    <a 
                      href={`https://gateway.pinata.cloud/ipfs/${task.ipfsResult}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-purple-400 hover:text-purple-300 text-sm"
                    >
                      查看结果 →
                    </a>
                  )}
                  {task.status !== 'Finalized' && task.status !== 'Cancelled' && (
                    <Link href={`/task-monitor?id=${task.taskId}`}>
                      <span className="text-blue-400 hover:text-blue-300 text-sm cursor-pointer">
                        监控详情 →
                      </span>
                    </Link>
                  )}
                  {task.status === 'ReadyForExecution' && (
                    <Link href={`/challenge?id=${task.taskId}`}>
                      <span className="text-red-400 hover:text-red-300 text-sm cursor-pointer">
                        发起挑战 →
                      </span>
                    </Link>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Create Task Modal */}
        {showCreateModal && (
          <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
            <div className="bg-slate-800 rounded-2xl p-8 max-w-lg w-full mx-4 border border-purple-500/30">
              <h2 className="text-2xl font-bold text-white mb-6">创建 TRO 任务</h2>
              
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-300 mb-2">任务意图</label>
                  <textarea
                    value={newIntent}
                    onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setNewIntent(e.target.value)}
                    placeholder="描述你想让AI完成的任务..."
                    className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-purple-500"
                    rows={3}
                  />
                </div>

                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-gray-300 mb-2">任务类型</label>
                    <select
                      value={newTaskType}
                      onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setNewTaskType(e.target.value as TaskType)}
                      className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-purple-500"
                      title="选择任务类型"
                    >
                      <option value="SimpleQA">简单问答</option>
                      <option value="ComplexReasoning">复杂推理</option>
                      <option value="MultiStep">多步骤任务</option>
                      <option value="DataAnalysis">数据分析</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm text-gray-300 mb-2">优先级</label>
                    <select
                      value={newCriticality}
                      onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setNewCriticality(e.target.value as TaskCriticality)}
                      className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white focus:outline-none focus:border-purple-500"
                      title="选择优先级"
                    >
                      <option value="Low">低优先级</option>
                      <option value="Standard">标准</option>
                      <option value="High">高优先级 (需ZK证明)</option>
                      <option value="MissionCritical">关键任务 (全量验证)</option>
                    </select>
                  </div>
                </div>

                {/* Estimates */}
                <div className="bg-white/5 rounded-lg p-4">
                  <div className="flex justify-between text-sm mb-2">
                    <span className="text-gray-400">预估复杂度</span>
                    <span className="text-white font-medium">{estimatedComplexity} / 1000</span>
                  </div>
                  <div className="h-2 bg-white/10 rounded-full overflow-hidden mb-4">
                    <div 
                      className="h-full bg-purple-500 rounded-full"
                      style={{ width: `${estimatedComplexity / 10}%` }}
                    />
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-400">预估费用</span>
                    <span className="text-white font-medium">{formatSol(estimatedCost)} SOL</span>
                  </div>
                </div>

                {/* ZK Info */}
                {(newCriticality === 'High' || newCriticality === 'MissionCritical') && (
                  <div className="bg-purple-500/10 border border-purple-500/30 rounded-lg p-4">
                    <div className="flex items-center gap-2 text-purple-300 text-sm">
                      <span>🔐</span>
                      <span>此任务将生成ZK证明以确保结果可信</span>
                    </div>
                  </div>
                )}
              </div>

              <div className="flex gap-4 mt-6">
                <button
                  onClick={() => setShowCreateModal(false)}
                  className="flex-1 bg-white/10 text-white py-3 rounded-lg font-semibold hover:bg-white/20 transition"
                >
                  取消
                </button>
                <button
                  onClick={handleCreateTask}
                  disabled={!newIntent.trim()}
                  className="flex-1 bg-gradient-to-r from-purple-600 to-pink-600 text-white py-3 rounded-lg font-semibold hover:opacity-90 transition disabled:opacity-50"
                >
                  创建任务
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

