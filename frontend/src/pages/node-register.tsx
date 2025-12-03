import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'
import { useState, useEffect } from 'react'

// Types matching on-chain enums
type ModelCapability = 'Local7B' | 'Local13B' | 'Local70B' | 'CloudAPI'
type NodeStatus = 'Pending' | 'Active' | 'Suspended' | 'Slashed' | 'Exiting'

interface ReasoningNode {
  owner: string
  modelCapability: ModelCapability
  stakeAmount: number
  dynamicStakeMin: number
  reputationScore: number
  dynamicMultiplier: number
  totalTasks: number
  successfulTasks: number
  cacheHitRate: number
  averageLatency: number
  pendingRewards: number
  status: NodeStatus
  benchmarkScore: number
  lastBenchmarkTs: string
}

// Mock nodes data
const mockNodes: ReasoningNode[] = [
  {
    owner: '7xKX...3fYz',
    modelCapability: 'Local7B',
    stakeAmount: 10_000_000_000,
    dynamicStakeMin: 8_000_000_000,
    reputationScore: 9200,
    dynamicMultiplier: 1150,
    totalTasks: 1234,
    successfulTasks: 1198,
    cacheHitRate: 6800,
    averageLatency: 1200,
    pendingRewards: 500_000_000,
    status: 'Active',
    benchmarkScore: 8500,
    lastBenchmarkTs: '2024-12-02T12:00:00Z',
  },
  {
    owner: '9aBC...xY12',
    modelCapability: 'Local13B',
    stakeAmount: 25_000_000_000,
    dynamicStakeMin: 20_000_000_000,
    reputationScore: 9500,
    dynamicMultiplier: 1200,
    totalTasks: 856,
    successfulTasks: 842,
    cacheHitRate: 4500,
    averageLatency: 2100,
    pendingRewards: 1_200_000_000,
    status: 'Active',
    benchmarkScore: 9200,
    lastBenchmarkTs: '2024-12-02T14:30:00Z',
  },
  {
    owner: '3dEF...pQ78',
    modelCapability: 'CloudAPI',
    stakeAmount: 50_000_000_000,
    dynamicStakeMin: 45_000_000_000,
    reputationScore: 9800,
    dynamicMultiplier: 1250,
    totalTasks: 2341,
    successfulTasks: 2339,
    cacheHitRate: 2300,
    averageLatency: 800,
    pendingRewards: 3_500_000_000,
    status: 'Active',
    benchmarkScore: 9800,
    lastBenchmarkTs: '2024-12-03T08:00:00Z',
  },
]

const capabilityInfo: Record<ModelCapability, { name: string; minStake: number; color: string }> = {
  Local7B: { name: '本地 7B 模型', minStake: 10_000_000_000, color: 'text-green-400' },
  Local13B: { name: '本地 13B 模型', minStake: 20_000_000_000, color: 'text-blue-400' },
  Local70B: { name: '本地 70B 模型', minStake: 35_000_000_000, color: 'text-purple-400' },
  CloudAPI: { name: '云端 API', minStake: 50_000_000_000, color: 'text-pink-400' },
}

const statusColors: Record<NodeStatus, string> = {
  Pending: 'bg-gray-100 text-gray-800',
  Active: 'bg-green-100 text-green-800',
  Suspended: 'bg-yellow-100 text-yellow-800',
  Slashed: 'bg-red-100 text-red-800',
  Exiting: 'bg-orange-100 text-orange-800',
}

export default function NodeRegister() {
  const { connected, publicKey } = useWallet()
  const [nodes] = useState<ReasoningNode[]>(mockNodes)
  const [showRegisterModal, setShowRegisterModal] = useState(false)
  const [myNode, setMyNode] = useState<ReasoningNode | null>(null)

  // Registration form state
  const [selectedCapability, setSelectedCapability] = useState<ModelCapability>('Local7B')
  const [stakeAmount, setStakeAmount] = useState('')
  const [ollamaEndpoint, setOllamaEndpoint] = useState('http://localhost:11434')
  const [modelName, setModelName] = useState('llama3.1:8b-instruct-q4_K_M')

  // Check if user has a node
  useEffect(() => {
    if (connected && publicKey) {
      // In production, query from Solana
      const existingNode = nodes.find(n => n.owner.includes(publicKey.toBase58().slice(0, 4)))
      setMyNode(existingNode || null)
    }
  }, [connected, publicKey, nodes])

  const handleRegister = async () => {
    if (!connected || !publicKey) {
      alert('请先连接钱包')
      return
    }

    const minStake = capabilityInfo[selectedCapability].minStake
    const stake = parseFloat(stakeAmount) * 1e9

    if (stake < minStake) {
      alert(`${capabilityInfo[selectedCapability].name} 最低质押: ${minStake / 1e9} SOL`)
      return
    }

    // In production, call Solana program
    alert('节点注册请求已提交！(演示模式)')
    setShowRegisterModal(false)
  }

  const handleWithdraw = async () => {
    if (!myNode) return
    // In production, call Solana program
    alert('提款请求已提交！(演示模式)')
  }

  const handleClaimRewards = async () => {
    if (!myNode || myNode.pendingRewards === 0) return
    // In production, call Solana program
    alert('奖励领取请求已提交！(演示模式)')
  }

  const formatSol = (lamports: number) => (lamports / 1e9).toFixed(2)
  const formatPercent = (bps: number) => (bps / 100).toFixed(1)
  const formatDate = (iso: string) => new Date(iso).toLocaleString('zh-CN')

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-indigo-900 to-slate-900">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <header className="flex justify-between items-center mb-8">
          <div className="flex items-center gap-4">
            <Link href="/">
              <span className="text-2xl font-bold text-white cursor-pointer hover:text-indigo-300 transition">
                ← DAOLLM
              </span>
            </Link>
            <h1 className="text-3xl font-bold text-white">推理节点管理</h1>
          </div>
          <WalletMultiButton />
        </header>

        {/* My Node Card */}
        {connected && (
          <div className="mb-8">
            <h2 className="text-xl font-semibold text-white mb-4">我的节点</h2>
            {myNode ? (
              <div className="bg-gradient-to-r from-indigo-500/20 to-purple-500/20 backdrop-blur rounded-2xl p-6 border border-indigo-500/30">
                <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
                  {/* Basic Info */}
                  <div className="space-y-3">
                    <div className="flex items-center gap-2">
                      <span className={`px-3 py-1 rounded-full text-xs font-medium ${statusColors[myNode.status]}`}>
                        {myNode.status}
                      </span>
                      <span className={`text-sm font-medium ${capabilityInfo[myNode.modelCapability].color}`}>
                        {capabilityInfo[myNode.modelCapability].name}
                      </span>
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">质押金额</div>
                      <div className="text-2xl font-bold text-white">{formatSol(myNode.stakeAmount)} SOL</div>
                      <div className="text-xs text-gray-500">
                        最低要求: {formatSol(myNode.dynamicStakeMin)} SOL
                      </div>
                    </div>
                  </div>

                  {/* Performance */}
                  <div className="space-y-3">
                    <div>
                      <div className="text-sm text-gray-400">信誉评分</div>
                      <div className="text-2xl font-bold text-green-400">{formatPercent(myNode.reputationScore)}%</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-400">动态倍率</div>
                      <div className="text-lg font-semibold text-purple-400">{formatPercent(myNode.dynamicMultiplier)}x</div>
                    </div>
                  </div>

                  {/* Stats */}
                  <div className="space-y-3">
                    <div>
                      <div className="text-sm text-gray-400">任务统计</div>
                      <div className="text-lg font-semibold text-white">
                        {myNode.successfulTasks} / {myNode.totalTasks}
                        <span className="text-sm text-gray-400 ml-2">
                          ({((myNode.successfulTasks / myNode.totalTasks) * 100).toFixed(1)}%)
                        </span>
                      </div>
                    </div>
                    <div className="flex gap-4">
                      <div>
                        <div className="text-xs text-gray-400">缓存命中率</div>
                        <div className="text-sm font-medium text-white">{formatPercent(myNode.cacheHitRate)}%</div>
                      </div>
                      <div>
                        <div className="text-xs text-gray-400">平均延迟</div>
                        <div className="text-sm font-medium text-white">{myNode.averageLatency}ms</div>
                      </div>
                    </div>
                  </div>

                  {/* Rewards & Actions */}
                  <div className="space-y-3">
                    <div>
                      <div className="text-sm text-gray-400">待领取奖励</div>
                      <div className="text-2xl font-bold text-yellow-400">{formatSol(myNode.pendingRewards)} SOL</div>
                    </div>
                    <div className="flex gap-2">
                      <button
                        onClick={handleClaimRewards}
                        disabled={myNode.pendingRewards === 0}
                        className="flex-1 bg-yellow-500 text-black py-2 rounded-lg font-semibold hover:bg-yellow-400 transition disabled:opacity-50"
                      >
                        领取奖励
                      </button>
                      <button
                        onClick={handleWithdraw}
                        className="flex-1 bg-white/10 text-white py-2 rounded-lg font-semibold hover:bg-white/20 transition"
                      >
                        提取质押
                      </button>
                    </div>
                  </div>
                </div>

                {/* Benchmark Info */}
                <div className="mt-4 pt-4 border-t border-white/10 flex justify-between items-center">
                  <div className="text-sm text-gray-400">
                    基准测试分数: <span className="text-white font-medium">{formatPercent(myNode.benchmarkScore)}%</span>
                    <span className="ml-4">上次测试: {formatDate(myNode.lastBenchmarkTs)}</span>
                  </div>
                  <button className="text-indigo-400 hover:text-indigo-300 text-sm">
                    运行基准测试 →
                  </button>
                </div>
              </div>
            ) : (
              <div className="bg-white/5 backdrop-blur rounded-2xl p-8 border border-white/10 text-center">
                <div className="text-5xl mb-4">🖥️</div>
                <h3 className="text-xl font-semibold text-white mb-2">尚未注册节点</h3>
                <p className="text-gray-400 mb-6">注册成为推理节点，为TRO网络提供算力并获得奖励</p>
                <button
                  onClick={() => setShowRegisterModal(true)}
                  className="bg-gradient-to-r from-indigo-600 to-purple-600 text-white px-8 py-3 rounded-lg font-semibold hover:opacity-90 transition"
                >
                  注册节点
                </button>
              </div>
            )}
          </div>
        )}

        {/* Network Stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">活跃节点</div>
            <div className="text-2xl font-bold text-white">{nodes.filter(n => n.status === 'Active').length}</div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">总质押量</div>
            <div className="text-2xl font-bold text-indigo-400">
              {formatSol(nodes.reduce((a, n) => a + n.stakeAmount, 0))} SOL
            </div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">网络平均信誉</div>
            <div className="text-2xl font-bold text-green-400">
              {formatPercent(nodes.reduce((a, n) => a + n.reputationScore, 0) / nodes.length)}%
            </div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">待分发奖励</div>
            <div className="text-2xl font-bold text-yellow-400">
              {formatSol(nodes.reduce((a, n) => a + n.pendingRewards, 0))} SOL
            </div>
          </div>
        </div>

        {/* All Nodes */}
        <div>
          <h2 className="text-xl font-semibold text-white mb-4">网络节点</h2>
          <div className="bg-white/5 backdrop-blur rounded-xl border border-white/10 overflow-hidden">
            <table className="w-full">
              <thead>
                <tr className="border-b border-white/10">
                  <th className="text-left text-sm text-gray-400 px-6 py-4">节点</th>
                  <th className="text-left text-sm text-gray-400 px-6 py-4">能力</th>
                  <th className="text-left text-sm text-gray-400 px-6 py-4">质押</th>
                  <th className="text-left text-sm text-gray-400 px-6 py-4">信誉</th>
                  <th className="text-left text-sm text-gray-400 px-6 py-4">任务成功率</th>
                  <th className="text-left text-sm text-gray-400 px-6 py-4">基准分数</th>
                  <th className="text-left text-sm text-gray-400 px-6 py-4">状态</th>
                </tr>
              </thead>
              <tbody>
                {nodes.map((node, i) => (
                  <tr key={i} className="border-b border-white/5 hover:bg-white/5 transition">
                    <td className="px-6 py-4">
                      <div className="font-mono text-white">{node.owner}</div>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`text-sm font-medium ${capabilityInfo[node.modelCapability].color}`}>
                        {capabilityInfo[node.modelCapability].name}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-white font-medium">{formatSol(node.stakeAmount)} SOL</div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-2">
                        <div className="w-24 h-2 bg-white/10 rounded-full overflow-hidden">
                          <div 
                            className="h-full bg-green-500 rounded-full"
                            style={{ width: `${node.reputationScore / 100}%` }}
                          />
                        </div>
                        <span className="text-white text-sm">{formatPercent(node.reputationScore)}%</span>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-white">
                        {((node.successfulTasks / node.totalTasks) * 100).toFixed(1)}%
                        <span className="text-gray-500 text-sm ml-1">({node.totalTasks})</span>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <div className="text-white">{formatPercent(node.benchmarkScore)}%</div>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`px-3 py-1 rounded-full text-xs font-medium ${statusColors[node.status]}`}>
                        {node.status}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>

        {/* Register Modal */}
        {showRegisterModal && (
          <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
            <div className="bg-slate-800 rounded-2xl p-8 max-w-lg w-full mx-4 border border-indigo-500/30">
              <h2 className="text-2xl font-bold text-white mb-6">注册推理节点</h2>
              
              <div className="space-y-4">
                {/* Capability Selection */}
                <div>
                  <label className="block text-sm text-gray-300 mb-2">模型能力</label>
                  <div className="grid grid-cols-2 gap-3">
                    {(Object.keys(capabilityInfo) as ModelCapability[]).map(cap => (
                      <button
                        key={cap}
                        onClick={() => setSelectedCapability(cap)}
                        className={`p-4 rounded-xl border text-left transition ${
                          selectedCapability === cap 
                            ? 'border-indigo-500 bg-indigo-500/20' 
                            : 'border-white/20 bg-white/5 hover:bg-white/10'
                        }`}
                      >
                        <div className={`font-medium ${capabilityInfo[cap].color}`}>
                          {capabilityInfo[cap].name}
                        </div>
                        <div className="text-xs text-gray-400 mt-1">
                          最低质押: {formatSol(capabilityInfo[cap].minStake)} SOL
                        </div>
                      </button>
                    ))}
                  </div>
                </div>

                {/* Stake Amount */}
                <div>
                  <label className="block text-sm text-gray-300 mb-2">质押金额 (SOL)</label>
                  <input
                    type="number"
                    value={stakeAmount}
                    onChange={(e) => setStakeAmount(e.target.value)}
                    placeholder={`最低 ${formatSol(capabilityInfo[selectedCapability].minStake)}`}
                    className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    质押越多，信誉提升越快，获得任务优先级越高
                  </p>
                </div>

                {/* Local Model Config */}
                {selectedCapability !== 'CloudAPI' && (
                  <>
                    <div>
                      <label className="block text-sm text-gray-300 mb-2">Ollama 端点</label>
                      <input
                        type="text"
                        value={ollamaEndpoint}
                        onChange={(e) => setOllamaEndpoint(e.target.value)}
                        placeholder="http://localhost:11434"
                        className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-gray-300 mb-2">模型名称</label>
                      <input
                        type="text"
                        value={modelName}
                        onChange={(e) => setModelName(e.target.value)}
                        placeholder="llama3.1:8b-instruct-q4_K_M"
                        className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500"
                      />
                    </div>
                  </>
                )}

                {/* Info */}
                <div className="bg-indigo-500/10 border border-indigo-500/30 rounded-lg p-4">
                  <div className="flex items-start gap-2 text-indigo-300 text-sm">
                    <span>ℹ️</span>
                    <div>
                      <p>注册后需要通过基准测试才能接收任务。</p>
                      <p className="mt-1">信誉评分会影响动态质押要求和奖励倍率。</p>
                    </div>
                  </div>
                </div>
              </div>

              <div className="flex gap-4 mt-6">
                <button
                  onClick={() => setShowRegisterModal(false)}
                  className="flex-1 bg-white/10 text-white py-3 rounded-lg font-semibold hover:bg-white/20 transition"
                >
                  取消
                </button>
                <button
                  onClick={handleRegister}
                  disabled={!stakeAmount || parseFloat(stakeAmount) < capabilityInfo[selectedCapability].minStake / 1e9}
                  className="flex-1 bg-gradient-to-r from-indigo-600 to-purple-600 text-white py-3 rounded-lg font-semibold hover:opacity-90 transition disabled:opacity-50"
                >
                  注册节点
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

