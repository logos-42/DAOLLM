import { useState, useEffect } from 'react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import Link from 'next/link'
import axios from 'axios'

interface GovernanceProposal {
  proposal_id: number
  proposer: string
  proposal_type: string
  description: string
  votes_for: number
  votes_against: number
  status: string
  created_at: number
  voting_ends_at: number
}

export default function Governance() {
  const [proposals, setProposals] = useState<GovernanceProposal[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    fetchProposals()
  }, [])

  const fetchProposals = async () => {
    try {
      const response = await axios.get('http://localhost:8000/api/governance/proposals')
      setProposals(response.data)
    } catch (error) {
      console.error('Failed to fetch proposals:', error)
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

        <h1 className="text-3xl font-bold text-gray-800 mb-8">模型治理</h1>

        <div className="bg-white rounded-lg shadow-lg p-6 mb-8">
          <h2 className="text-xl font-semibold mb-4">当前模型配置</h2>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-gray-600 text-sm">模型版本</p>
              <p className="text-lg font-semibold">v1.0</p>
            </div>
            <div>
              <p className="text-gray-600 text-sm">学习率</p>
              <p className="text-lg font-semibold">0.001</p>
            </div>
            <div>
              <p className="text-gray-600 text-sm">批次大小</p>
              <p className="text-lg font-semibold">32</p>
            </div>
            <div>
              <p className="text-gray-600 text-sm">最小节点信誉</p>
              <p className="text-lg font-semibold">50/100</p>
            </div>
          </div>
        </div>

        {loading ? (
          <div className="text-center py-12">
            <p className="text-gray-600">加载中...</p>
          </div>
        ) : (
          <div className="space-y-4">
            {proposals.map((proposal) => (
              <div key={proposal.proposal_id} className="bg-white rounded-lg shadow-lg p-6">
                <div className="flex justify-between items-start mb-4">
                  <div>
                    <h3 className="text-lg font-semibold">提案 #{proposal.proposal_id}</h3>
                    <p className="text-sm text-gray-600">{proposal.proposal_type}</p>
                  </div>
                  <span className={`px-3 py-1 rounded-full text-sm ${
                    proposal.status === 'active' ? 'bg-blue-100 text-blue-800' :
                    proposal.status === 'passed' ? 'bg-green-100 text-green-800' :
                    'bg-gray-100 text-gray-800'
                  }`}>
                    {proposal.status}
                  </span>
                </div>
                <p className="text-gray-700 mb-4">{proposal.description}</p>
                <div className="flex justify-between items-center">
                  <div className="text-sm text-gray-600">
                    <span>支持: {proposal.votes_for}</span>
                    <span className="mx-2">•</span>
                    <span>反对: {proposal.votes_against}</span>
                  </div>
                  <button className="bg-primary-600 text-white px-4 py-2 rounded-lg hover:bg-primary-700">
                    投票
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

