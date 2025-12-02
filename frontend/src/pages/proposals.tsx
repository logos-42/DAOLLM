import { useState, useEffect } from 'react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'
import axios from 'axios'

interface Proposal {
  proposal_id: string
  ipfs_hash: string
  submitter: string
  timestamp: number
  status: string
}

export default function Proposals() {
  const [proposals, setProposals] = useState<Proposal[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchProposals()
  }, [])

  const fetchProposals = async () => {
    try {
      const response = await axios.get('http://localhost:8000/api/proposals')
      setProposals(response.data)
    } catch (error) {
      console.error('Failed to fetch proposals:', error)
    } finally {
      setLoading(false)
    }
  }

  const getStatusBadge = (status: string) => {
    const statusMap: Record<string, { color: string; text: string }> = {
      submitted: { color: 'bg-blue-100 text-blue-800', text: '已提交' },
      analyzing: { color: 'bg-yellow-100 text-yellow-800', text: '分析中' },
      completed: { color: 'bg-green-100 text-green-800', text: '已完成' },
    }
    const statusInfo = statusMap[status] || { color: 'bg-gray-100 text-gray-800', text: status }
    return (
      <span className={`px-3 py-1 rounded-full text-sm ${statusInfo.color}`}>
        {statusInfo.text}
      </span>
    )
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

        <div className="flex justify-between items-center mb-8">
          <h1 className="text-3xl font-bold text-gray-800">提案列表</h1>
          <Link href="/submit">
            <button className="bg-primary-600 text-white px-6 py-2 rounded-lg font-semibold hover:bg-primary-700 transition">
              创建提案
            </button>
          </Link>
        </div>

        {loading ? (
          <div className="text-center py-12">
            <p className="text-gray-600">加载中...</p>
          </div>
        ) : proposals.length === 0 ? (
          <div className="bg-white rounded-lg shadow-lg p-12 text-center">
            <p className="text-gray-600 mb-4">暂无提案</p>
            <Link href="/submit">
              <button className="bg-primary-600 text-white px-6 py-2 rounded-lg font-semibold hover:bg-primary-700 transition">
                创建第一个提案
              </button>
            </Link>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {proposals.map((proposal) => (
              <Link key={proposal.proposal_id} href={`/proposals/${proposal.proposal_id}`}>
                <div className="bg-white rounded-lg shadow-lg p-6 hover:shadow-xl transition cursor-pointer">
                  <div className="flex justify-between items-start mb-4">
                    <h3 className="text-lg font-semibold text-gray-800">
                      📄 提案 #{proposal.proposal_id.slice(-8)}
                    </h3>
                    {getStatusBadge(proposal.status)}
                  </div>
                  <p className="text-gray-600 text-sm mb-4">
                    IPFS: {proposal.ipfs_hash.slice(0, 20)}...
                  </p>
                  <div className="text-xs text-gray-500">
                    {new Date(proposal.timestamp * 1000).toLocaleString('zh-CN')}
                  </div>
                </div>
              </Link>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

