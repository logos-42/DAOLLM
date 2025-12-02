import { useState } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import { useRouter } from 'next/router'
import Link from 'next/link'
import axios from 'axios'

export default function SubmitProposal() {
  const { publicKey, connected } = useWallet()
  const router = useRouter()
  const [title, setTitle] = useState('')
  const [content, setContent] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    
    if (!connected || !publicKey) {
      setError('请先连接钱包')
      return
    }

    if (!content.trim()) {
      setError('请输入提案内容')
      return
    }

    setLoading(true)
    setError('')

    try {
      const response = await axios.post('http://localhost:8000/api/proposals', {
        title: title || undefined,
        content,
      })

      if (response.data) {
        router.push(`/proposals/${response.data.proposal_id}`)
      }
    } catch (err: any) {
      setError(err.response?.data?.detail || '提交失败，请重试')
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

        <div className="max-w-3xl mx-auto">
          <h1 className="text-3xl font-bold text-gray-800 mb-8">创建新提案</h1>

          {!connected && (
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
              <p className="text-yellow-800">请先连接钱包以提交提案</p>
            </div>
          )}

          <form onSubmit={handleSubmit} className="bg-white rounded-lg shadow-lg p-8">
            <div className="mb-6">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                提案标题（可选）
              </label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                placeholder="例如：降低协议手续费"
              />
            </div>

            <div className="mb-6">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                提案内容 *
              </label>
              <textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                rows={12}
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                placeholder="请输入你的提案内容..."
                required
              />
            </div>

            {error && (
              <div className="mb-6 bg-red-50 border border-red-200 rounded-lg p-4">
                <p className="text-red-800">{error}</p>
              </div>
            )}

            <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
              <p className="text-blue-800 text-sm">
                💡 提示：提交后AI将自动分析你的提案
              </p>
              <p className="text-blue-800 text-sm mt-1">
                💰 提交奖励：10 $PROPOSAL
              </p>
            </div>

            <button
              type="submit"
              disabled={loading || !connected}
              className="w-full bg-primary-600 text-white px-6 py-3 rounded-lg font-semibold hover:bg-primary-700 transition disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {loading ? '提交中...' : '提交提案'}
            </button>
          </form>
        </div>
      </div>
    </div>
  )
}

