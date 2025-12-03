import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'
import { useState } from 'react'
import { useRouter } from 'next/router'

// Types
type ChallengeStatus = 'Active' | 'VotingInProgress' | 'Resolved' | 'Rejected'
type ResolutionOutcome = 'ChallengerWins' | 'DefenderWins' | 'Draw' | 'Pending'

interface Challenge {
  challengeId: string
  taskId: string
  taskIntent: string
  challenger: string
  challengerStake: number
  reason: string
  evidenceHash: string
  counterResult?: string
  status: ChallengeStatus
  resolution: ResolutionOutcome
  votesFor: number
  votesAgainst: number
  createdAt: string
  votingDeadline?: string
  resolvedAt?: string
}

// Mock data
const mockChallenges: Challenge[] = [
  {
    challengeId: '1',
    taskId: '5',
    taskIntent: '分析Aave V3的清算机制风险',
    challenger: '8xYZ...7abc',
    challengerStake: 2_000_000_000,
    reason: '原结果遗漏了闪电贷攻击向量的关键风险分析',
    evidenceHash: 'QmEvidence1...',
    counterResult: '补充了闪电贷攻击的详细分析和历史案例',
    status: 'VotingInProgress',
    resolution: 'Pending',
    votesFor: 45,
    votesAgainst: 32,
    createdAt: '2024-12-02T10:00:00Z',
    votingDeadline: '2024-12-05T10:00:00Z',
  },
  {
    challengeId: '2',
    taskId: '8',
    taskIntent: 'ETH/USDC 当前价格查询',
    challenger: '3dEF...pQ78',
    challengerStake: 500_000_000,
    reason: '返回的价格数据与多个可信来源不符',
    evidenceHash: 'QmEvidence2...',
    status: 'Resolved',
    resolution: 'ChallengerWins',
    votesFor: 89,
    votesAgainst: 11,
    createdAt: '2024-12-01T08:00:00Z',
    resolvedAt: '2024-12-02T08:00:00Z',
  },
  {
    challengeId: '3',
    taskId: '12',
    taskIntent: '评估某新DeFi协议的安全性',
    challenger: '9aBC...xY12',
    challengerStake: 1_500_000_000,
    reason: '模型产生了明显的幻觉内容，引用了不存在的审计报告',
    evidenceHash: 'QmEvidence3...',
    status: 'Resolved',
    resolution: 'DefenderWins',
    votesFor: 23,
    votesAgainst: 77,
    createdAt: '2024-11-30T15:00:00Z',
    resolvedAt: '2024-12-01T15:00:00Z',
  },
]

const statusColors: Record<ChallengeStatus, string> = {
  Active: 'bg-blue-100 text-blue-800',
  VotingInProgress: 'bg-yellow-100 text-yellow-800',
  Resolved: 'bg-green-100 text-green-800',
  Rejected: 'bg-gray-100 text-gray-800',
}

const outcomeColors: Record<ResolutionOutcome, string> = {
  ChallengerWins: 'text-green-400',
  DefenderWins: 'text-red-400',
  Draw: 'text-yellow-400',
  Pending: 'text-gray-400',
}

const outcomeLabels: Record<ResolutionOutcome, string> = {
  ChallengerWins: '挑战者胜',
  DefenderWins: '原结果维持',
  Draw: '平局',
  Pending: '待定',
}

export default function ChallengePage() {
  const { connected, publicKey } = useWallet()
  const router = useRouter()
  const { id: targetTaskId } = router.query
  
  const [challenges] = useState<Challenge[]>(mockChallenges)
  const [showChallengeModal, setShowChallengeModal] = useState(!!targetTaskId)
  const [filter, setFilter] = useState<ChallengeStatus | 'all'>('all')

  // Challenge form state
  const [taskId, setTaskId] = useState(targetTaskId as string || '')
  const [reason, setReason] = useState('')
  const [evidence, setEvidence] = useState('')
  const [stakeAmount, setStakeAmount] = useState('')

  const handleSubmitChallenge = async () => {
    if (!connected || !publicKey) {
      alert('请先连接钱包')
      return
    }

    if (!taskId || !reason || !stakeAmount) {
      alert('请填写完整信息')
      return
    }

    // In production, call Solana program
    alert('挑战已提交！(演示模式)')
    setShowChallengeModal(false)
    setTaskId('')
    setReason('')
    setEvidence('')
    setStakeAmount('')
  }

  const handleVote = async (challengeId: string, support: boolean) => {
    if (!connected || !publicKey) {
      alert('请先连接钱包')
      return
    }

    // In production, call governance contract
    alert(`投票已提交: ${support ? '支持挑战' : '反对挑战'} (演示模式)`)
  }

  const filteredChallenges = filter === 'all' ? challenges : challenges.filter(c => c.status === filter)

  const formatSol = (lamports: number) => (lamports / 1e9).toFixed(2)
  const formatDate = (iso: string) => new Date(iso).toLocaleString('zh-CN')

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-red-900 to-slate-900">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <header className="flex justify-between items-center mb-8">
          <div className="flex items-center gap-4">
            <Link href="/">
              <span className="text-2xl font-bold text-white cursor-pointer hover:text-red-300 transition">
                ← DAOLLM
              </span>
            </Link>
            <h1 className="text-3xl font-bold text-white">争议解决中心</h1>
          </div>
          <WalletMultiButton />
        </header>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">活跃争议</div>
            <div className="text-2xl font-bold text-yellow-400">
              {challenges.filter(c => c.status === 'Active' || c.status === 'VotingInProgress').length}
            </div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">已解决</div>
            <div className="text-2xl font-bold text-green-400">
              {challenges.filter(c => c.status === 'Resolved').length}
            </div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">挑战者胜率</div>
            <div className="text-2xl font-bold text-white">
              {Math.round(
                (challenges.filter(c => c.resolution === 'ChallengerWins').length / 
                 challenges.filter(c => c.status === 'Resolved').length) * 100
              )}%
            </div>
          </div>
          <div className="bg-white/10 backdrop-blur rounded-xl p-4 border border-white/20">
            <div className="text-sm text-gray-300">总质押池</div>
            <div className="text-2xl font-bold text-red-400">
              {formatSol(challenges.reduce((a, c) => a + c.challengerStake, 0))} SOL
            </div>
          </div>
        </div>

        {/* Actions & Filter */}
        <div className="flex justify-between items-center mb-6">
          <div className="flex gap-2">
            <button
              onClick={() => setFilter('all')}
              className={`px-4 py-2 rounded-lg transition ${filter === 'all' ? 'bg-red-600 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
            >
              全部
            </button>
            <button
              onClick={() => setFilter('VotingInProgress')}
              className={`px-4 py-2 rounded-lg transition ${filter === 'VotingInProgress' ? 'bg-yellow-600 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
            >
              投票中
            </button>
            <button
              onClick={() => setFilter('Resolved')}
              className={`px-4 py-2 rounded-lg transition ${filter === 'Resolved' ? 'bg-green-600 text-white' : 'bg-white/10 text-gray-300 hover:bg-white/20'}`}
            >
              已解决
            </button>
          </div>
          <button
            onClick={() => setShowChallengeModal(true)}
            className="bg-gradient-to-r from-red-600 to-orange-600 text-white px-6 py-2 rounded-lg font-semibold hover:opacity-90 transition"
          >
            + 发起挑战
          </button>
        </div>

        {/* Challenge List */}
        <div className="space-y-4">
          {filteredChallenges.map(challenge => (
            <div key={challenge.challengeId} className="bg-white/10 backdrop-blur rounded-xl p-6 border border-white/20">
              {/* Header */}
              <div className="flex justify-between items-start mb-4">
                <div className="flex-1">
                  <div className="flex items-center gap-3 mb-2">
                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${statusColors[challenge.status]}`}>
                      {challenge.status === 'VotingInProgress' ? '投票中' : 
                       challenge.status === 'Resolved' ? '已解决' : 
                       challenge.status === 'Active' ? '活跃' : '已拒绝'}
                    </span>
                    {challenge.status === 'Resolved' && (
                      <span className={`text-sm font-medium ${outcomeColors[challenge.resolution]}`}>
                        {outcomeLabels[challenge.resolution]}
                      </span>
                    )}
                    <span className="text-xs text-gray-400">任务 #{challenge.taskId}</span>
                  </div>
                  <h3 className="text-lg font-semibold text-white mb-2">{challenge.taskIntent}</h3>
                  <div className="flex items-center gap-4 text-sm text-gray-400">
                    <span>挑战者: {challenge.challenger}</span>
                    <span>•</span>
                    <span>质押: {formatSol(challenge.challengerStake)} SOL</span>
                  </div>
                </div>
              </div>

              {/* Reason */}
              <div className="bg-white/5 rounded-lg p-4 mb-4">
                <div className="text-sm text-gray-400 mb-1">挑战理由</div>
                <div className="text-white">{challenge.reason}</div>
              </div>

              {/* Counter Result */}
              {challenge.counterResult && (
                <div className="bg-green-500/10 border border-green-500/30 rounded-lg p-4 mb-4">
                  <div className="text-sm text-green-400 mb-1">补充证据/结论</div>
                  <div className="text-white">{challenge.counterResult}</div>
                </div>
              )}

              {/* Voting Progress */}
              {(challenge.status === 'VotingInProgress' || challenge.status === 'Resolved') && (
                <div className="mb-4">
                  <div className="flex justify-between text-sm text-gray-400 mb-2">
                    <span>DAO投票进度</span>
                    <span>{challenge.votesFor + challenge.votesAgainst} 票</span>
                  </div>
                  <div className="flex h-4 rounded-full overflow-hidden">
                    <div 
                      className="bg-green-500"
                      style={{ width: `${(challenge.votesFor / (challenge.votesFor + challenge.votesAgainst)) * 100}%` }}
                    />
                    <div 
                      className="bg-red-500"
                      style={{ width: `${(challenge.votesAgainst / (challenge.votesFor + challenge.votesAgainst)) * 100}%` }}
                    />
                  </div>
                  <div className="flex justify-between text-xs mt-1">
                    <span className="text-green-400">支持: {challenge.votesFor}</span>
                    <span className="text-red-400">反对: {challenge.votesAgainst}</span>
                  </div>
                </div>
              )}

              {/* Footer */}
              <div className="flex justify-between items-center pt-4 border-t border-white/10">
                <div className="text-sm text-gray-400">
                  创建于 {formatDate(challenge.createdAt)}
                  {challenge.votingDeadline && (
                    <span className="ml-4">投票截止: {formatDate(challenge.votingDeadline)}</span>
                  )}
                  {challenge.resolvedAt && (
                    <span className="ml-4">解决于: {formatDate(challenge.resolvedAt)}</span>
                  )}
                </div>
                
                {/* Voting Buttons */}
                {challenge.status === 'VotingInProgress' && connected && (
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleVote(challenge.challengeId, true)}
                      className="bg-green-500/20 border border-green-500/50 text-green-400 px-4 py-2 rounded-lg text-sm font-medium hover:bg-green-500/30 transition"
                    >
                      👍 支持挑战
                    </button>
                    <button
                      onClick={() => handleVote(challenge.challengeId, false)}
                      className="bg-red-500/20 border border-red-500/50 text-red-400 px-4 py-2 rounded-lg text-sm font-medium hover:bg-red-500/30 transition"
                    >
                      👎 反对挑战
                    </button>
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>

        {/* How it Works */}
        <div className="mt-12 bg-white/5 backdrop-blur rounded-2xl p-8 border border-white/10">
          <h2 className="text-xl font-semibold text-white mb-6">争议解决机制</h2>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="text-center">
              <div className="w-12 h-12 bg-red-500/20 rounded-full flex items-center justify-center mx-auto mb-3">
                <span className="text-2xl">1️⃣</span>
              </div>
              <h3 className="font-semibold text-white mb-2">发起挑战</h3>
              <p className="text-sm text-gray-400">质押任务奖励的20%，提交争议理由和反证</p>
            </div>
            <div className="text-center">
              <div className="w-12 h-12 bg-yellow-500/20 rounded-full flex items-center justify-center mx-auto mb-3">
                <span className="text-2xl">2️⃣</span>
              </div>
              <h3 className="font-semibold text-white mb-2">DAO投票</h3>
              <p className="text-sm text-gray-400">社区成员投票决定争议结果，持有更多代币权重更大</p>
            </div>
            <div className="text-center">
              <div className="w-12 h-12 bg-green-500/20 rounded-full flex items-center justify-center mx-auto mb-3">
                <span className="text-2xl">3️⃣</span>
              </div>
              <h3 className="font-semibold text-white mb-2">结果执行</h3>
              <p className="text-sm text-gray-400">根据投票结果，胜方获得败方押金，更新节点信誉</p>
            </div>
            <div className="text-center">
              <div className="w-12 h-12 bg-purple-500/20 rounded-full flex items-center justify-center mx-auto mb-3">
                <span className="text-2xl">4️⃣</span>
              </div>
              <h3 className="font-semibold text-white mb-2">系统学习</h3>
              <p className="text-sm text-gray-400">争议案例纳入训练数据，持续优化模型质量</p>
            </div>
          </div>
        </div>

        {/* Challenge Modal */}
        {showChallengeModal && (
          <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
            <div className="bg-slate-800 rounded-2xl p-8 max-w-lg w-full mx-4 border border-red-500/30">
              <h2 className="text-2xl font-bold text-white mb-6">发起挑战</h2>
              
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-300 mb-2">任务 ID</label>
                  <input
                    type="text"
                    value={taskId}
                    onChange={(e) => setTaskId(e.target.value)}
                    placeholder="输入要挑战的任务ID"
                    className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-red-500"
                  />
                </div>

                <div>
                  <label className="block text-sm text-gray-300 mb-2">挑战理由</label>
                  <textarea
                    value={reason}
                    onChange={(e) => setReason(e.target.value)}
                    placeholder="详细说明为什么认为原结果有问题..."
                    className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-red-500"
                    rows={4}
                  />
                </div>

                <div>
                  <label className="block text-sm text-gray-300 mb-2">证据/反证 (IPFS Hash 或 URL)</label>
                  <input
                    type="text"
                    value={evidence}
                    onChange={(e) => setEvidence(e.target.value)}
                    placeholder="Qm... 或 https://..."
                    className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-red-500"
                  />
                </div>

                <div>
                  <label className="block text-sm text-gray-300 mb-2">质押金额 (SOL)</label>
                  <input
                    type="number"
                    value={stakeAmount}
                    onChange={(e) => setStakeAmount(e.target.value)}
                    placeholder="最低为任务奖励的 20%"
                    className="w-full bg-white/10 border border-white/20 rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:border-red-500"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    挑战失败将损失质押金额
                  </p>
                </div>

                {/* Warning */}
                <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4">
                  <div className="flex items-start gap-2 text-red-300 text-sm">
                    <span>⚠️</span>
                    <div>
                      <p>发起挑战需要质押 SOL，如果DAO投票认定原结果正确，您将损失全部质押。</p>
                      <p className="mt-1">请确保您有充分的证据支持您的挑战。</p>
                    </div>
                  </div>
                </div>
              </div>

              <div className="flex gap-4 mt-6">
                <button
                  onClick={() => setShowChallengeModal(false)}
                  className="flex-1 bg-white/10 text-white py-3 rounded-lg font-semibold hover:bg-white/20 transition"
                >
                  取消
                </button>
                <button
                  onClick={handleSubmitChallenge}
                  disabled={!taskId || !reason || !stakeAmount}
                  className="flex-1 bg-gradient-to-r from-red-600 to-orange-600 text-white py-3 rounded-lg font-semibold hover:opacity-90 transition disabled:opacity-50"
                >
                  提交挑战
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

