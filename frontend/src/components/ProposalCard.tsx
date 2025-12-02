import Link from 'next/link'

interface ProposalCardProps {
  proposalId: string
  title?: string
  status: string
  timestamp: number
  ipfsHash: string
}

export default function ProposalCard({
  proposalId,
  title,
  status,
  timestamp,
  ipfsHash,
}: ProposalCardProps) {
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
    <Link href={`/proposals/${proposalId}`}>
      <div className="bg-white rounded-lg shadow-lg p-6 hover:shadow-xl transition cursor-pointer">
        <div className="flex justify-between items-start mb-4">
          <h3 className="text-lg font-semibold text-gray-800">
            📄 {title || `提案 #${proposalId.slice(-8)}`}
          </h3>
          {getStatusBadge(status)}
        </div>
        <p className="text-gray-600 text-sm mb-4">
          IPFS: {ipfsHash.slice(0, 20)}...
        </p>
        <div className="text-xs text-gray-500">
          {new Date(timestamp * 1000).toLocaleString('zh-CN')}
        </div>
      </div>
    </Link>
  )
}

