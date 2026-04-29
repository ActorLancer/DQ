'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Search, 
  Filter,
  FileText,
  Clock,
  CheckCircle,
  XCircle,
  AlertCircle,
  Eye,
  MessageSquare,
  Calendar,
  Shield,
  TrendingUp
} from 'lucide-react'

interface AccessRequest {
  id: string
  requestId: string
  listingTitle: string
  supplierName: string
  plan: string
  status: 'DRAFT' | 'SUBMITTED' | 'PENDING_SUPPLIER_REVIEW' | 'PENDING_PLATFORM_REVIEW' | 'NEED_MORE_INFO' | 'APPROVED' | 'REJECTED' | 'CANCELLED'
  usagePurpose: string
  expectedUsage: string
  createdAt: string
  updatedAt: string
  reviewNotes?: string
  chainStatus: 'NOT_SUBMITTED' | 'SUBMITTED' | 'CONFIRMED'
  projectionStatus: 'PENDING' | 'PROJECTED'
}

const MOCK_REQUESTS: AccessRequest[] = [
  {
    id: 'req_001',
    requestId: 'request_20260428_001',
    listingTitle: '金融市场行情数据',
    supplierName: '金融数据服务',
    plan: '专业版',
    status: 'PENDING_SUPPLIER_REVIEW',
    usagePurpose: '用于量化交易策略回测和实时行情分析',
    expectedUsage: '预计每日调用 5000 次',
    createdAt: '2026-04-28 10:30:00',
    updatedAt: '2026-04-28 10:30:00',
    chainStatus: 'SUBMITTED',
    projectionStatus: 'PENDING',
  },
  {
    id: 'req_002',
    requestId: 'request_20260427_002',
    listingTitle: '医疗健康知识图谱',
    supplierName: '医疗大数据中心',
    plan: '定制版',
    status: 'NEED_MORE_INFO',
    usagePurpose: '用于智能诊断辅助系统开发',
    expectedUsage: '预计每月调用 20000 次',
    createdAt: '2026-04-27 14:20:00',
    updatedAt: '2026-04-28 09:15:00',
    reviewNotes: '请补充：1. 数据使用场景详细说明 2. 数据安全保障措施 3. 相关医疗资质证明',
    chainStatus: 'SUBMITTED',
    projectionStatus: 'PROJECTED',
  },
  {
    id: 'req_003',
    requestId: 'request_20260426_003',
    listingTitle: '供应链物流数据',
    supplierName: '智运物流数据',
    plan: '企业版',
    status: 'APPROVED',
    usagePurpose: '用于供应链优化和物流路径规划',
    expectedUsage: '预计每日调用 3000 次',
    createdAt: '2026-04-26 16:00:00',
    updatedAt: '2026-04-27 10:30:00',
    reviewNotes: '申请已通过，请前往"我的订阅"查看 API Key',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
  },
  {
    id: 'req_004',
    requestId: 'request_20260425_004',
    listingTitle: '社交媒体舆情数据',
    supplierName: '舆情监测平台',
    plan: '标准版',
    status: 'REJECTED',
    usagePurpose: '用于品牌舆情监测和分析',
    expectedUsage: '预计每日调用 2000 次',
    createdAt: '2026-04-25 11:00:00',
    updatedAt: '2026-04-26 15:20:00',
    reviewNotes: '拒绝原因：使用用途描述不够详细，且未提供相关业务资质证明',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
  },
  {
    id: 'req_005',
    requestId: 'request_20260424_005',
    listingTitle: '房地产市场数据',
    supplierName: '房产数据研究院',
    plan: '按量计费',
    status: 'PENDING_PLATFORM_REVIEW',
    usagePurpose: '用于房地产市场分析和投资决策',
    expectedUsage: '预计每月调用 10000 次',
    createdAt: '2026-04-24 09:30:00',
    updatedAt: '2026-04-25 14:00:00',
    chainStatus: 'SUBMITTED',
    projectionStatus: 'PROJECTED',
  },
]

const STATUS_CONFIG = {
  DRAFT: { label: '草稿', color: 'bg-gray-100 text-gray-800', icon: FileText },
  SUBMITTED: { label: '已提交', color: 'bg-blue-100 text-blue-800', icon: Clock },
  PENDING_SUPPLIER_REVIEW: { label: '待供应商审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  PENDING_PLATFORM_REVIEW: { label: '待平台审核', color: 'bg-purple-100 text-purple-800', icon: Clock },
  NEED_MORE_INFO: { label: '需补充材料', color: 'bg-orange-100 text-orange-800', icon: AlertCircle },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: XCircle },
  CANCELLED: { label: '已取消', color: 'bg-gray-100 text-gray-800', icon: XCircle },
}

export default function BuyerRequestsPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedRequest, setSelectedRequest] = useState<AccessRequest | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredRequests = MOCK_REQUESTS.filter((req) => {
    const matchesKeyword = 
      req.listingTitle.toLowerCase().includes(searchKeyword.toLowerCase()) ||
      req.supplierName.toLowerCase().includes(searchKeyword.toLowerCase())
    const matchesStatus = selectedStatus === 'all' || req.status === selectedStatus
    return matchesKeyword && matchesStatus
  })

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:requests:read"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">我的申请</h1>
          <p className="text-gray-600">查看和管理您的数据访问申请</p>
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
                placeholder="搜索商品名称或供应商..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>

            <select
              value={selectedStatus}
              onChange={(e) => setSelectedStatus(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部状态</option>
              <option value="PENDING_SUPPLIER_REVIEW">待供应商审核</option>
              <option value="PENDING_PLATFORM_REVIEW">待平台审核</option>
              <option value="NEED_MORE_INFO">需补充材料</option>
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
                      <h3 className="text-lg font-bold text-gray-900 mb-2">{request.listingTitle}</h3>
                      <div className="flex items-center gap-3 mb-2">
                        <span className="text-sm text-gray-600">{request.supplierName}</span>
                        <span className="text-sm text-gray-400">·</span>
                        <span className="text-sm font-medium text-gray-900">{request.plan}</span>
                      </div>
                    </div>
                    <span className={`status-tag ${statusConfig.color}`}>
                      <StatusIcon className="w-3.5 h-3.5" />
                      <span>{statusConfig.label}</span>
                    </span>
                  </div>

                  {/* 使用用途 */}
                  <div className="mb-4 pb-4 border-b border-gray-100">
                    <div className="text-xs text-gray-500 mb-1">使用用途</div>
                    <p className="text-sm text-gray-900 line-clamp-2">{request.usagePurpose}</p>
                  </div>

                  {/* 底部信息 */}
                  <div className="flex items-center justify-between text-xs text-gray-500">
                    <div className="flex items-center gap-4">
                      <div className="flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        <span>申请时间: {request.createdAt.split(' ')[0]}</span>
                      </div>
                    </div>
                    <button className="text-primary-600 hover:text-primary-700 font-medium">
                      查看详情 →
                    </button>
                  </div>

                  {/* 审核备注提示 */}
                  {request.reviewNotes && (
                    <div className={`mt-4 p-3 rounded-lg border ${
                      request.status === 'NEED_MORE_INFO'
                        ? 'bg-orange-50 border-orange-200'
                        : request.status === 'APPROVED'
                        ? 'bg-green-50 border-green-200'
                        : 'bg-red-50 border-red-200'
                    }`}>
                      <div className="flex items-start gap-2">
                        <MessageSquare className={`w-4 h-4 flex-shrink-0 mt-0.5 ${
                          request.status === 'NEED_MORE_INFO'
                            ? 'text-orange-600'
                            : request.status === 'APPROVED'
                            ? 'text-green-600'
                            : 'text-red-600'
                        }`} />
                        <p className={`text-xs ${
                          request.status === 'NEED_MORE_INFO'
                            ? 'text-orange-800'
                            : request.status === 'APPROVED'
                            ? 'text-green-800'
                            : 'text-red-800'
                        }`}>
                          {request.reviewNotes}
                        </p>
                      </div>
                    </div>
                  )}
                </div>
              )
            })}
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
                    <code className="font-mono text-xs text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                      {selectedRequest.requestId}
                    </code>
                  </div>

                  {/* 商品信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">商品信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">商品:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.listingTitle}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">供应商:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.supplierName}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">套餐:</span>
                        <span className="font-medium text-gray-900">{selectedRequest.plan}</span>
                      </div>
                    </div>
                  </div>

                  {/* 申请内容 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">申请内容</div>
                    <div className="space-y-3">
                      <div>
                        <div className="text-xs text-gray-600 mb-1">使用用途</div>
                        <p className="text-sm text-gray-900">{selectedRequest.usagePurpose}</p>
                      </div>
                      <div>
                        <div className="text-xs text-gray-600 mb-1">预计用量</div>
                        <p className="text-sm text-gray-900">{selectedRequest.expectedUsage}</p>
                      </div>
                    </div>
                  </div>

                  {/* 状态信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">状态信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">申请状态:</span>
                        <span className={`status-tag text-xs ${STATUS_CONFIG[selectedRequest.status].color}`}>
                          {STATUS_CONFIG[selectedRequest.status].label}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">链状态:</span>
                        <span className={`status-tag text-xs ${
                          selectedRequest.chainStatus === 'CONFIRMED'
                            ? 'bg-green-100 text-green-800'
                            : 'bg-yellow-100 text-yellow-800'
                        }`}>
                          {selectedRequest.chainStatus === 'CONFIRMED' ? '已确认' : '已提交'}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">投影状态:</span>
                        <span className={`status-tag text-xs ${
                          selectedRequest.projectionStatus === 'PROJECTED'
                            ? 'bg-green-100 text-green-800'
                            : 'bg-gray-100 text-gray-800'
                        }`}>
                          {selectedRequest.projectionStatus === 'PROJECTED' ? '已投影' : '待投影'}
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* 时间信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">时间信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">创建时间:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedRequest.createdAt}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">更新时间:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedRequest.updatedAt}</span>
                      </div>
                    </div>
                  </div>

                  {/* 审核备注 */}
                  {selectedRequest.reviewNotes && (
                    <div>
                      <div className="text-xs text-gray-500 mb-2">审核备注</div>
                      <div className={`p-3 rounded-lg border text-sm ${
                        selectedRequest.status === 'NEED_MORE_INFO'
                          ? 'bg-orange-50 border-orange-200 text-orange-800'
                          : selectedRequest.status === 'APPROVED'
                          ? 'bg-green-50 border-green-200 text-green-800'
                          : 'bg-red-50 border-red-200 text-red-800'
                      }`}>
                        {selectedRequest.reviewNotes}
                      </div>
                    </div>
                  )}

                  {/* 链上凭证 */}
                  {selectedRequest.chainStatus === 'CONFIRMED' && (
                    <div>
                      <div className="text-xs text-gray-500 mb-2">链上凭证</div>
                      <div className="flex items-center gap-2 p-3 bg-green-50 border border-green-200 rounded-lg">
                        <Shield className="w-4 h-4 text-success-600" />
                        <span className="text-xs text-success-800 font-medium">已链上存证</span>
                      </div>
                    </div>
                  )}

                  {/* 操作按钮 */}
                  {selectedRequest.status === 'NEED_MORE_INFO' && (
                    <div className="pt-4 border-t border-gray-200">
                      <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                        <FileText className="w-4 h-4" />
                        <span>补充材料</span>
                      </button>
                    </div>
                  )}

                  {selectedRequest.status === 'APPROVED' && (
                    <div className="pt-4 border-t border-gray-200">
                      <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                        <TrendingUp className="w-4 h-4" />
                        <span>前往订阅管理</span>
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
