import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'

export default function Home() {
  const { connected } = useWallet()

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="container mx-auto px-4 py-8">
        <header className="flex justify-between items-center mb-12">
          <h1 className="text-4xl font-bold text-gray-800">DAO提案分析系统</h1>
          <WalletMultiButton />
        </header>

        <div className="text-center mb-12">
          <h2 className="text-3xl font-semibold text-gray-700 mb-4">
            🤖 让AI帮你理解DAO提案
          </h2>
          <p className="text-lg text-gray-600 mb-8">
            去中心化AI网络自动分析提案，生成摘要、风险评估和决策建议
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-12">
          <div className="bg-white rounded-lg shadow-lg p-6">
            <div className="text-3xl mb-4">📄</div>
            <h3 className="text-xl font-semibold mb-2">活跃提案</h3>
            <p className="text-4xl font-bold text-primary-600">23个</p>
          </div>
          <div className="bg-white rounded-lg shadow-lg p-6">
            <div className="text-3xl mb-4">🌐</div>
            <h3 className="text-xl font-semibold mb-2">网络状态</h3>
            <p className="text-4xl font-bold text-primary-600">15个节点</p>
          </div>
          <div className="bg-white rounded-lg shadow-lg p-6">
            <div className="text-3xl mb-4">⚡</div>
            <h3 className="text-xl font-semibold mb-2">平均响应</h3>
            <p className="text-4xl font-bold text-primary-600">1.8秒</p>
          </div>
        </div>

        {/* TRO Features */}
        <div className="bg-gradient-to-r from-purple-600/20 to-pink-600/20 rounded-2xl p-6 mb-8 border border-purple-500/30">
          <h3 className="text-xl font-semibold text-gray-800 mb-4">🚀 TRO 可信推理中控层</h3>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <Link href="/tro-tasks">
              <div className="bg-white/80 rounded-xl p-4 hover:shadow-lg transition cursor-pointer">
                <div className="text-2xl mb-2">🧠</div>
                <div className="font-semibold text-gray-800">TRO任务</div>
                <div className="text-sm text-gray-600">提交和监控推理任务</div>
              </div>
            </Link>
            <Link href="/node-register">
              <div className="bg-white/80 rounded-xl p-4 hover:shadow-lg transition cursor-pointer">
                <div className="text-2xl mb-2">🖥️</div>
                <div className="font-semibold text-gray-800">推理节点</div>
                <div className="text-sm text-gray-600">注册和管理算力节点</div>
              </div>
            </Link>
            <Link href="/challenge">
              <div className="bg-white/80 rounded-xl p-4 hover:shadow-lg transition cursor-pointer">
                <div className="text-2xl mb-2">⚖️</div>
                <div className="font-semibold text-gray-800">争议解决</div>
                <div className="text-sm text-gray-600">发起和参与DAO仲裁</div>
              </div>
            </Link>
            <div className="bg-white/80 rounded-xl p-4">
              <div className="text-2xl mb-2">🔐</div>
              <div className="font-semibold text-gray-800">ZK证明</div>
              <div className="text-sm text-gray-600">高价值任务强制验证</div>
            </div>
          </div>
        </div>

        <div className="flex gap-4 justify-center mb-12 flex-wrap">
          <Link href="/submit">
            <button className="bg-primary-600 text-white px-8 py-3 rounded-lg text-lg font-semibold hover:bg-primary-700 transition">
              创建提案
            </button>
          </Link>
          <Link href="/proposals">
            <button className="bg-white text-primary-600 border-2 border-primary-600 px-8 py-3 rounded-lg text-lg font-semibold hover:bg-primary-50 transition">
              查看提案
            </button>
          </Link>
          <Link href="/nodes">
            <button className="bg-white text-primary-600 border-2 border-primary-600 px-8 py-3 rounded-lg text-lg font-semibold hover:bg-primary-50 transition">
              节点管理
            </button>
          </Link>
          <Link href="/governance">
            <button className="bg-white text-primary-600 border-2 border-primary-600 px-8 py-3 rounded-lg text-lg font-semibold hover:bg-primary-50 transition">
              模型治理
            </button>
          </Link>
          <Link href="/rewards">
            <button className="bg-white text-primary-600 border-2 border-primary-600 px-8 py-3 rounded-lg text-lg font-semibold hover:bg-primary-50 transition">
              奖励系统
            </button>
          </Link>
        </div>

        {connected && (
          <div className="bg-white rounded-lg shadow-lg p-6">
            <h3 className="text-xl font-semibold mb-4">最新提案</h3>
            <div className="border border-gray-200 rounded-lg p-4">
              <div className="flex justify-between items-start mb-2">
                <h4 className="text-lg font-semibold">📄 降低协议手续费提案</h4>
                <span className="bg-yellow-100 text-yellow-800 px-3 py-1 rounded-full text-sm">
                  🟡 中等风险
                </span>
              </div>
              <p className="text-gray-600 mb-2">建议将协议手续费从0.3%降低到0.25%</p>
              <div className="flex items-center gap-4 text-sm text-gray-500">
                <span>建议: ✅ 支持</span>
                <span>•</span>
                <span>2024-01-15 10:30</span>
              </div>
              <Link href="/proposals/1">
                <button className="mt-4 text-primary-600 hover:underline">
                  查看详情 →
                </button>
              </Link>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

