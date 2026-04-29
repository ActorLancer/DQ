'use client'

import { Copy, ExternalLink, CheckCircle, AlertCircle, Clock, XCircle } from 'lucide-react'
import { useState } from 'react'
import type { ChainProof, ChainStatus, ProjectionStatus } from '@/types'

interface ChainProofCardProps {
  chainProof: ChainProof
}

const CHAIN_STATUS_CONFIG: Record<ChainStatus, { label: string; color: string; icon: any }> = {
  NOT_SUBMITTED: { label: '未提交', color: 'bg-gray-100 text-gray-800', icon: Clock },
  PENDING_SUBMIT: { label: '待提交', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  SUBMITTING: { label: '提交中', color: 'bg-blue-100 text-blue-800', icon: Clock },
  SUBMITTED: { label: '已提交', color: 'bg-blue-100 text-blue-800', icon: CheckCircle },
  CONFIRMED: { label: '已确认', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  FAILED: { label: '失败', color: 'bg-red-100 text-red-800', icon: XCircle },
  RETRYING: { label: '重试中', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  UNKNOWN: { label: '未知', color: 'bg-gray-100 text-gray-800', icon: AlertCircle },
}

const PROJECTION_STATUS_CONFIG: Record<ProjectionStatus, { label: string; color: string }> = {
  PENDING: { label: '待投影', color: 'bg-yellow-100 text-yellow-800' },
  PROJECTED: { label: '已投影', color: 'bg-green-100 text-green-800' },
  OUT_OF_SYNC: { label: '不一致', color: 'bg-red-100 text-red-800' },
  REBUILDING: { label: '重建中', color: 'bg-blue-100 text-blue-800' },
  FAILED: { label: '投影失败', color: 'bg-red-100 text-red-800' },
}

export default function ChainProofCard({ chainProof }: ChainProofCardProps) {
  const [copied, setCopied] = useState<string | null>(null)

  const copyToClipboard = (text: string, type: string) => {
    navigator.clipboard.writeText(text)
    setCopied(type)
    setTimeout(() => setCopied(null), 2000)
  }

  const chainStatusConfig = CHAIN_STATUS_CONFIG[chainProof.chainStatus]
  const projectionStatusConfig = PROJECTION_STATUS_CONFIG[chainProof.projectionStatus]
  const ChainIcon = chainStatusConfig.icon

  return (
    <div className="bg-gradient-to-br from-gray-50 to-gray-100 rounded-xl border-2 border-gray-200 p-6">
      <h3 className="text-lg font-bold text-gray-900 mb-4 flex items-center gap-2">
        <span className="w-2 h-2 bg-primary-600 rounded-full"></span>
        链上凭证
      </h3>

      <div className="space-y-4">
        {/* Request ID */}
        <div>
          <div className="text-xs text-gray-500 mb-1">Request ID</div>
          <div className="flex items-center gap-2">
            <code className="font-hash text-gray-900 flex-1 truncate">
              {chainProof.requestId}
            </code>
            <button
              onClick={() => copyToClipboard(chainProof.requestId, 'requestId')}
              className="p-1.5 hover:bg-gray-200 rounded transition-colors"
              title="复制"
            >
              {copied === 'requestId' ? (
                <CheckCircle className="w-4 h-4 text-success-600" />
              ) : (
                <Copy className="w-4 h-4 text-gray-600" />
              )}
            </button>
          </div>
        </div>

        {/* Tx Hash */}
        {chainProof.txHash && (
          <div>
            <div className="text-xs text-gray-500 mb-1">Tx Hash</div>
            <div className="flex items-center gap-2">
              <code className="font-hash text-gray-900 flex-1 truncate">
                {chainProof.txHash}
              </code>
              <button
                onClick={() => copyToClipboard(chainProof.txHash!, 'txHash')}
                className="p-1.5 hover:bg-gray-200 rounded transition-colors"
                title="复制"
              >
                {copied === 'txHash' ? (
                  <CheckCircle className="w-4 h-4 text-success-600" />
                ) : (
                  <Copy className="w-4 h-4 text-gray-600" />
                )}
              </button>
              <a
                href={`/chain/tx/${chainProof.txHash}`}
                target="_blank"
                rel="noopener noreferrer"
                className="p-1.5 hover:bg-gray-200 rounded transition-colors"
                title="查看详情"
              >
                <ExternalLink className="w-4 h-4 text-gray-600" />
              </a>
            </div>
          </div>
        )}

        {/* 状态 */}
        <div className="grid grid-cols-2 gap-4">
          <div>
            <div className="text-xs text-gray-500 mb-2">链状态</div>
            <div className={`status-tag ${chainStatusConfig.color}`}>
              <ChainIcon className="w-3.5 h-3.5" />
              <span>{chainStatusConfig.label}</span>
            </div>
          </div>
          <div>
            <div className="text-xs text-gray-500 mb-2">投影状态</div>
            <div className={`status-tag ${projectionStatusConfig.color}`}>
              <span>{projectionStatusConfig.label}</span>
            </div>
          </div>
        </div>

        {/* 区块信息 */}
        {chainProof.blockHeight && (
          <div className="grid grid-cols-2 gap-4 pt-4 border-t border-gray-200">
            <div>
              <div className="text-xs text-gray-500 mb-1">区块高度</div>
              <div className="font-hash text-sm text-gray-900">{chainProof.blockHeight}</div>
            </div>
            {chainProof.contractName && (
              <div>
                <div className="text-xs text-gray-500 mb-1">合约名称</div>
                <div className="font-hash text-sm text-gray-900">{chainProof.contractName}</div>
              </div>
            )}
          </div>
        )}

        {/* 时间信息 */}
        <div className="pt-4 border-t border-gray-200 space-y-2">
          {chainProof.submittedAt && (
            <div className="flex justify-between text-xs">
              <span className="text-gray-500">上链时间</span>
              <span className="text-gray-900">
                {new Date(chainProof.submittedAt).toLocaleString('zh-CN')}
              </span>
            </div>
          )}
          {chainProof.lastCheckedAt && (
            <div className="flex justify-between text-xs">
              <span className="text-gray-500">最近检查</span>
              <span className="text-gray-900">
                {new Date(chainProof.lastCheckedAt).toLocaleString('zh-CN')}
              </span>
            </div>
          )}
        </div>

        {/* 错误信息 */}
        {chainProof.errorMessage && (
          <div className="pt-4 border-t border-gray-200">
            <div className="text-xs text-gray-500 mb-1">错误信息</div>
            <div className="text-sm text-red-600 bg-red-50 p-2 rounded">
              {chainProof.errorMessage}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
