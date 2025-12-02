interface NodeStatusProps {
  nodeId: string
  owner: string
  stakeAmount: number
  reputationScore: number
  isActive: boolean
}

export default function NodeStatus({
  nodeId,
  owner,
  stakeAmount,
  reputationScore,
  isActive,
}: NodeStatusProps) {
  return (
    <div className="bg-white rounded-lg shadow-lg p-6">
      <div className="flex justify-between items-start mb-4">
        <h3 className="text-lg font-semibold">{nodeId}</h3>
        <span className={`px-3 py-1 rounded-full text-sm ${
          isActive 
            ? 'bg-green-100 text-green-800' 
            : 'bg-gray-100 text-gray-800'
        }`}>
          {isActive ? '🟢 在线' : '⚫ 离线'}
        </span>
      </div>
      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-gray-600">所有者:</span>
          <span className="font-mono text-xs">{owner.slice(0, 8)}...</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">质押金额:</span>
          <span className="font-semibold">{stakeAmount} SOL</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">信誉评分:</span>
          <span className="font-semibold">{reputationScore}/100</span>
        </div>
      </div>
    </div>
  )
}

