import { useState, useEffect } from 'react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'
import axios from 'axios'

interface Node {
  id: string
  owner: string
  stake_amount: number
  reputation_score: number
  is_active: boolean
}

export default function Nodes() {
  const [nodes, setNodes] = useState<Node[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchNodes()
  }, [])

  const fetchNodes = async () => {
    try {
      const response = await axios.get('http://localhost:8000/api/inference/nodes')
      setNodes(response.data)
    } catch (error) {
      console.error('Failed to fetch nodes:', error)
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

        <h1 className="text-3xl font-bold text-gray-800 mb-8">推理节点管理</h1>

        <div className="bg-white rounded-lg shadow-lg p-6 mb-8">
          <h2 className="text-xl font-semibold mb-4">网络统计</h2>
          <div className="grid grid-cols-3 gap-4">
            <div>
              <p className="text-gray-600 text-sm">活跃节点</p>
              <p className="text-2xl font-bold text-primary-600">{nodes.filter(n => n.is_active).length}个</p>
            </div>
            <div>
              <p className="text-gray-600 text-sm">总质押</p>
              <p className="text-2xl font-bold text-primary-600">
                {nodes.reduce((sum, n) => sum + n.stake_amount, 0).toLocaleString()} SOL
              </p>
            </div>
            <div>
              <p className="text-gray-600 text-sm">平均评分</p>
              <p className="text-2xl font-bold text-primary-600">
                {nodes.length > 0 
                  ? Math.round(nodes.reduce((sum, n) => sum + n.reputation_score, 0) / nodes.length)
                  : 0}/100
              </p>
            </div>
          </div>
        </div>

        {loading ? (
          <div className="text-center py-12">
            <p className="text-gray-600">加载中...</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {nodes.map((node) => (
              <div key={node.id} className="bg-white rounded-lg shadow-lg p-6">
                <div className="flex justify-between items-start mb-4">
                  <h3 className="text-lg font-semibold">{node.id}</h3>
                  <span className={`px-3 py-1 rounded-full text-sm ${
                    node.is_active 
                      ? 'bg-green-100 text-green-800' 
                      : 'bg-gray-100 text-gray-800'
                  }`}>
                    {node.is_active ? '🟢 在线' : '⚫ 离线'}
                  </span>
                </div>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span className="text-gray-600">所有者:</span>
                    <span className="font-mono text-xs">{node.owner.slice(0, 8)}...</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600">质押金额:</span>
                    <span className="font-semibold">{node.stake_amount} SOL</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-gray-600">信誉评分:</span>
                    <span className="font-semibold">{node.reputation_score}/100</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

