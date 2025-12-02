import { useState, useEffect } from 'react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'
import axios from 'axios'

interface RewardHistory {
  recipient: string
  amount: number
  reward_type: string
  timestamp: number
}

export default function Rewards() {
  const [history, setHistory] = useState<RewardHistory[]>([])
  const [balance, setBalance] = useState<any>({})
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchData()
  }, [])

  const fetchData = async () => {
    try {
      const [historyRes, balanceRes] = await Promise.all([
        axios.get('http://localhost:8000/api/rewards/history'),
        axios.get('http://localhost:8000/api/rewards/balance')
      ])
      setHistory(historyRes.data)
      setBalance(balanceRes.data)
    } catch (error) {
      console.error('Failed to fetch rewards:', error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
      <div className="container mx-auto px-4 py-8">
        <header className="flex justify-between items-center mb-8">
          <Link href="/">
            <button className="text-primary-600 hover:underline">← 返回首页</button>
          </Link>
          <WalletMultiButton />
        </header>

        <h1 className="text-3xl font-bold text-gray-800 mb-8">奖励系统</h1>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
          <div className="bg-white rounded-lg shadow-lg p-6">
            <p className="text-gray-600 text-sm mb-2">数据贡献奖励</p>
            <p className="text-2xl font-bold text-primary-600">{balance.data_contribution || 0} $PROPOSAL</p>
          </div>
          <div className="bg-white rounded-lg shadow-lg p-6">
            <p className="text-gray-600 text-sm mb-2">推理奖励</p>
            <p className="text-2xl font-bold text-primary-600">{balance.inference || 0} $PROPOSAL</p>
          </div>
          <div className="bg-white rounded-lg shadow-lg p-6">
            <p className="text-gray-600 text-sm mb-2">训练奖励</p>
            <p className="text-2xl font-bold text-primary-600">{balance.training || 0} $PROPOSAL</p>
          </div>
          <div className="bg-white rounded-lg shadow-lg p-6">
            <p className="text-gray-600 text-sm mb-2">治理奖励</p>
            <p className="text-2xl font-bold text-primary-600">{balance.governance || 0} $PROPOSAL</p>
          </div>
        </div>

        <div className="bg-white rounded-lg shadow-lg p-6">
          <h2 className="text-xl font-semibold mb-4">奖励历史</h2>
          {loading ? (
            <p className="text-gray-600">加载中...</p>
          ) : history.length === 0 ? (
            <p className="text-gray-600">暂无奖励记录</p>
          ) : (
            <div className="space-y-2">
              {history.map((reward, index) => (
                <div key={index} className="flex justify-between items-center border-b pb-2">
                  <div>
                    <p className="font-semibold">{reward.reward_type}</p>
                    <p className="text-sm text-gray-600">{reward.recipient.slice(0, 8)}...</p>
                  </div>
                  <div className="text-right">
                    <p className="font-semibold text-primary-600">+{reward.amount} $PROPOSAL</p>
                    <p className="text-sm text-gray-600">
                      {new Date(reward.timestamp * 1000).toLocaleDateString()}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

