'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Search, 
  Filter, 
  CheckCircle, 
  XCircle, 
  AlertCircle,
  Clock,
  Eye,
  FileText,
  User,
  Building2,
  Calendar,
  TrendingUp
} from 'lucide-react'

interface AccessRequestItem {
  id: string
  requestId: string
  buyerName: string
  buyerSubject: string
  listingTitle: string
  planName: string
  usagePurpose: string
  expectedUsage: string
  status: string
  workflowStatus: string
  chainStatus: string
  projectionStatus: string
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
  createdAt: string
  updatedAt: string
}

const MOCK_REQUESTS: AccessRequestItem[] = [
  {
    id: 'ar_001',
    requestId: 'req_20260428_000001',
    buyerName: '某某科技有限公司',
    buyerSubject: '买方主体',
    listingTitle: '企业工商风险数据',
    planName: '标准版',
    usagePurpose: '用于供应链合作伙伴风险评估，预计每月查询 5000 次',
    expectedUsage: '5000/月',
    status: 'PENDING_SUPPLIER_REVIEW',
    workflowStatus: '待供应商审核',
    chainStatus: 'NOT_SUBMITTED',
    projectionStatus: 'PROJECTED',
    riskLevel: 'LOW',
    createdAt: '2026-04-28 14:30:00',
    updatedAt: '2026-04-28 14:30:00',
  },
  {
    id: 'ar_002',
    requestId: 'req_20260428_000002',
    buyerName: '智慧消费研究院',
    buyerSubject: '买方主体',
    listingTitle: '企业工商风险数据',
    planName: '企业版',
    usagePurpose: '用于信贷审批与风险控制，预计每月查询 15000 次',
    expectedUsage: '15000/月',
    status: 'PENDING_SUPPLIER_REVIEW',
    workflowStatus: '待供应商审核',
    chainStatus: 'NOT_SUBMITTED',
    projectionStatus: 'PROJECTED',
    riskLevel: 'MEDIUM',
    createdAt: '2026-04-28 10:15:00',
    updatedAt: '2026-04-28 10:15:00',
  },
  {
    id: 'ar_003',
    requestId: 'req_20260427_000003',
    buyerName: '金融数据服务',
    buyerSubject: '买方主体',
    listingTitle: '企业工商风险数据',
    planName: '企业版',
    usagePurpose: '用于投资决策支持，预计每月查询 20000 次',
    expectedUsage: '20000/月',
    status: 'NEED_MORE_INFO',
    workflowStatus: '需补充材料',
    chainStatus: 'NOT_SUBMITTED',
    projectionStatus: 'PROJECTED',
    riskLevel: 'HIGH',
    createdAt: '2026-04-27 16:45:00',
    updatedAt: '2026-04-28 09:20:00',
  },
]

const STATUS_CONFIG: Record<string, { label: string; color: string; icon: any }> = {
  PENDING_SUPPLIER_REVIEW: { label: '待审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  NEED_MORE_INFO: { label: '需补充', color: 'bg-orange-100 text-orange-800', icon: AlertCircle },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: XCircle },
}

const RISK_CONFIG = {
  LOW: { label: '低风险', color: 'text-green-600 bg-green-50' },
  MEDIUM: { label: '中风险', color: 'text-yellow-600 bg-yellow-50' },
  HIGH: { label: '高风险', color: 'text-red-600 bg-red-50' },
}

export default function SellerRequestsPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedRequest, setSelectedRequest] = useState<AccessRequestItem | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredRequests = MOCK_REQUESTS.filter((request) => {
    const matchesKeyword = 
      request.buyerName.toLowerCase().includes(searchKeyword.toLowerCase()) ||
      request.listingTitle.toLowerCase().includes(searchKeyword.toLowerCase())
    const matchesStatus = selectedStatus === 'all' || request.status === selectedStatus
    return matchesKeyword && matchesStatus
  })

  const handleApprove = (request: AccessRequestItem) => {
    alert(`通过申请: ${request.requestId}`)
  }

  const handleReject = (request: AccessRequestItem) => {
    alert(`拒绝申请: ${request.requestId}`)
  }

  const handleRequestMoreInfo = (request: AccessRequestItem) => {
    alert(`要求补充材料: ${request.requestId}`)
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:listings:write"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">申请审批</h1>
          <p className="text-gray-600">处理买方的数据访问申请</p>
        </div>

        {/* 筛选和搜索 */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
              <input
                type="text"
                value={searchKeyword}
                onChange={(e) => setSearchKeyword(e.target.value)}
                placeholder="搜索买方名称或商品名称..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>

            <select
              value={selectedStatus}
              onChange={(e) => setSelectedStatus(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部状态</option>
              <option value="PENDING_SUPPLIER_REVIEW">待审核</option>
              <option value="NEED_MORE_INFO">需补充</option>
              <option value="APPROVED">已通过</option>
              <option value="REJECTED">已拒绝</option>
            </select>

            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>更多筛选</span>
            </button>
          </div>
        </div>

        {/* 申请列表 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 左侧列表 */}
          <div className="lg:col-span-2 space-y-4">
            {filteredRequests.map((request) => {
              const statusConfig = STATUS_CONFIG[request.status]
              const riskConfig = RISK_CONFIG[request.riskLevel]
              const StatusIcon = statusConfig.icon
              const isSelected = selectedRequest?.id === request.id

              return (
                <div
                  key={request.id}
                  onClick={() => setSelectedRequest(request)}
                  className={`bg-white rounded-xl border-2 p-6 cursor-pointer transition-all ${
                    isSelected
                      ? 'border-primary-500 shadow-lg'
                      : 'border-gray-200 hover:border-primary-300 hover:shadow-md'
                  }`}
                >
                  {/* 头部 */}
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex-1">
                      <div className="flex items-center gap-3 mb-2">
                        <h3 className="text-lg font-bold text-gray-900">{request.buyerName}</h3>
                        <span className={`status-tag ${statusConfig.color}`}>
                          <StatusIcon className="w-3.5 h-3.5" />
                          <span>{statusConfig.label}</span>
                        </span>
                        <span className={`text-xs px-2 py-1 rounded-full font-medium ${riskConfig.color}`}>
                          {riskConfig.label}
                        </span>
                      </div>
                      <div className="text-sm text-gray-600">
                        申请商品: <span className="font-medium text-gray-900">{request.listingTitle}</span> · {request.planName}
                      </div>
                    </div>
                  </div>

                  {/* 用途 */}
                  <div className="mb-4">
                    <div className="text-xs text-gray-500 mb-1">使用用途</div>
                    <p className="text-sm text-gray-700 line-clamp-2">{request.usagePurpose}</p>
                  </div>

                  {/* 底部信息 */}
                  <div className="flex items-center justify-between pt-4 border-t border-gray-100">
                    <div className="flex items-center gap-4 text-xs text-gray-500">
                      <div className="flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        <span>{request.createdAt}</span>
                      </div>
                      <div className="flex items-center gap-1">
                        <TrendingUp className="w-3 h-3" />
                        <span>{request.expectedUsage}</span>
                      </div>
                    </div>

                    {request.status === 'PENDING_SUPPLIER_REVIEW' && (
                      <div className="flex gap-2">
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            handleApprove(request)
                          }}
                          className="px-3 py-1.5 bg-success-600 text-white rounded-lg hover:bg-success-700 text-sm font-medium"
                        >
                          通过
                        </button>
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            handleReject(request)
                          }}
                          className="px-3 py-1.5 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium"
                        >
                          拒绝
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              )
            })}

            {filteredRequests.length === 0 && (
              <div className="bg-white rounded-xl border border-gray-200 p-12 text-center">
                <FileText className="w-16 h-16 mx-auto text-gray-300 mb-4" />
                <h3 className="text-lg font-medium text-gray-900 mb-2">暂无申请</h3>
                <p className="text-gray-600">当前没有符合条件的访问申请</p>
              </div>
            )}
          </div>

          {/* 右侧详情 */}
          <div className="lg:col-span-1">
            {selectedRequest ? (
              <div className="bg-white rounded-xl border border-gray-200 p-6 sticky top-28">
                <h3 className="text-lg font-bold text-gray-900 mb-6">申请详情</h3>

                <div className="space-y-6">
                  {/* Request ID */}
                  <div>
                    <div className="text-xs text-gray-500 mb-1">Request ID</div>
                    <code className="font-mono text-sm text-gray-900 bg-gray-50 px-2 py-1 rounded">
                      {selectedRequest.requestId}
                    </code>
                  </div>

                  {/* 买方信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">买方信息</div>
                    <div className="flex items-start gap-3 p-3 bg-gray-50 rounded-lg">
                      <Building2 className="w-5 h-5 text-gray-600 flex-shrink-0 mt-0.5" />
                      <div>
                        <div className="font-medium text-gray-900 mb-1">{selectedRequest.buyerName}</div>
                        <div className="text-xs text-gray-600">{selectedRequest.buyerSubject}</div>
                      </div>
                    </div>
                  </div>

                  {/* 申请内容 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">申请内容</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">商品:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.listingTitle}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">套餐:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.planName}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">预计用量:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.expectedUsage}</span>
                      </div>
                    </div>
                  </div>

                  {/* 使用用途 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">使用用途</div>
                    <p className="text-sm text-gray-700 bg-gray-50 p-3 rounded-lg">
                      {selectedRequest.usagePurpose}
                    </p>
                  </div>

                  {/* 状态信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">状态信息</div>
                    <div className="space-y-2">
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-600">工作流状态:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.workflowStatus}</span>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-600">链状态:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.chainStatus}</span>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-600">投影状态:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.projectionStatus}</span>
                      </div>
                      <div className="flex justify-between text-sm">
                        <span className="text-gray-600">风险等级:</span>
                        <span className={`text-xs px-2 py-0.5 rounded-full font-medium ${RISK_CONFIG[selectedRequest.riskLevel].color}`}>
                          {RISK_CONFIG[selectedRequest.riskLevel].label}
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* 操作按钮 */}
                  {selectedRequest.status === 'PENDING_SUPPLIER_REVIEW' && (
                    <div className="space-y-2 pt-4 border-t border-gray-200">
                      <button
                        onClick={() => handleApprove(selectedRequest)}
                        className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-success-600 text-white rounded-lg hover:bg-success-700 font-medium"
                      >
                        <CheckCircle className="w-4 h-4" />
                        <span>通过申请</span>
                      </button>
                      <button
                        onClick={() => handleRequestMoreInfo(selectedRequest)}
                        className="w-full flex items-center justify-center gap-2 px-4 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium"
                      >
                        <AlertCircle className="w-4 h-4" />
                        <span>要求补充材料</span>
                      </button>
                      <button
                        onClick={() => handleReject(selectedRequest)}
                        className="w-full flex items-center justify-center gap-2 px-4 py-3 border border-red-300 text-red-600 rounded-lg hover:bg-red-50 font-medium"
                      >
                        <XCircle className="w-4 h-4" />
                        <span>拒绝申请</span>
                      </button>
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <div className="bg-white rounded-xl border border-gray-200 p-12 text-center sticky top-28">
                <Eye className="w-12 h-12 mx-auto text-gray-300 mb-4" />
                <p className="text-gray-600">选择一个申请查看详情</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
