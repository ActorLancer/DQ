'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Search,
  Filter,
  CheckCircle,
  XCircle,
  Clock,
  AlertTriangle,
  Eye,
  FileText,
  Shield,
  TrendingUp,
  Users,
  DollarSign,
  Calendar,
  Tag
} from 'lucide-react'

interface ListingReview {
  id: string
  listingId: string
  title: string
  supplierName: string
  supplierId: string
  industry: string
  dataType: string
  pricingModel: string
  basePrice: number
  status: 'PENDING' | 'APPROVED' | 'REJECTED' | 'REVISION_REQUIRED'
  submittedAt: string
  reviewedAt: string | null
  reviewedBy: string | null
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
  qualityScore: number
  complianceIssues: string[]
  reviewNotes: string
}

const MOCK_LISTINGS: ListingReview[] = [
  {
    id: 'review_001',
    listingId: 'listing_new_001',
    title: '全国企业工商信息实时查询API',
    supplierName: '天眼数据科技',
    supplierId: 'subject_seller_001',
    industry: '企业征信',
    dataType: 'API',
    pricingModel: '按量计费',
    basePrice: 0.5,
    status: 'PENDING',
    submittedAt: '2026-04-29 09:30:00',
    reviewedAt: null,
    reviewedBy: null,
    riskLevel: 'LOW',
    qualityScore: 92,
    complianceIssues: [],
    reviewNotes: '',
  },
  {
    id: 'review_002',
    listingId: 'listing_new_002',
    title: '个人消费行为画像数据',
    supplierName: '智慧消费研究院',
    supplierId: 'subject_seller_002',
    industry: '消费分析',
    dataType: 'Dataset',
    pricingModel: '订阅制',
    basePrice: 9999,
    status: 'PENDING',
    submittedAt: '2026-04-29 10:15:00',
    reviewedAt: null,
    reviewedBy: null,
    riskLevel: 'HIGH',
    qualityScore: 78,
    complianceIssues: ['缺少隐私保护说明', '数据来源不明确'],
    reviewNotes: '',
  },
  {
    id: 'review_003',
    listingId: 'listing_new_003',
    title: '物流轨迹实时追踪数据',
    supplierName: '快递物流联盟',
    supplierId: 'subject_seller_003',
    industry: '物流',
    dataType: 'Stream',
    pricingModel: '按量计费',
    basePrice: 0.1,
    status: 'REVISION_REQUIRED',
    submittedAt: '2026-04-28 14:20:00',
    reviewedAt: '2026-04-29 08:00:00',
    reviewedBy: '审核员A',
    riskLevel: 'MEDIUM',
    qualityScore: 85,
    complianceIssues: ['API 文档不完整'],
    reviewNotes: '请补充完整的 API 文档和错误码说明',
  },
  {
    id: 'review_004',
    listingId: 'listing_new_004',
    title: '金融市场实时行情数据',
    supplierName: '金融数据服务商',
    supplierId: 'subject_seller_004',
    industry: '金融',
    dataType: 'API',
    pricingModel: '订阅制',
    basePrice: 19999,
    status: 'APPROVED',
    submittedAt: '2026-04-27 11:00:00',
    reviewedAt: '2026-04-28 16:30:00',
    reviewedBy: '审核员B',
    riskLevel: 'LOW',
    qualityScore: 95,
    complianceIssues: [],
    reviewNotes: '数据质量优秀，合规性良好，批准上架',
  },
]

const STATUS_CONFIG = {
  PENDING: { label: '待审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  APPROVED: { label: '已批准', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: XCircle },
  REVISION_REQUIRED: { label: '需修改', color: 'bg-orange-100 text-orange-800', icon: AlertTriangle },
}

const RISK_CONFIG = {
  LOW: { label: '低风险', color: 'text-green-600 bg-green-50' },
  MEDIUM: { label: '中风险', color: 'text-yellow-600 bg-yellow-50' },
  HIGH: { label: '高风险', color: 'text-red-600 bg-red-50' },
}

export default function AdminListingsPage() {
  const [listings, setListings] = useState(MOCK_LISTINGS)
  const [selectedListing, setSelectedListing] = useState<ListingReview | null>(null)
  const [showReviewModal, setShowReviewModal] = useState(false)
  const [reviewAction, setReviewAction] = useState<'approve' | 'reject' | 'revision'>('approve')
  const [reviewNotes, setReviewNotes] = useState('')
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')

  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredListings = listings.filter(listing => {
    const matchesKeyword = listing.title.toLowerCase().includes(searchKeyword.toLowerCase()) ||
                          listing.supplierName.toLowerCase().includes(searchKeyword.toLowerCase())
    const matchesStatus = selectedStatus === 'all' || listing.status === selectedStatus
    return matchesKeyword && matchesStatus
  })

  const handleReview = (listing: ListingReview) => {
    setSelectedListing(listing)
    setReviewNotes(listing.reviewNotes)
    setShowReviewModal(true)
  }

  const handleSubmitReview = () => {
    if (!selectedListing) return

    const newStatus = reviewAction === 'approve' ? 'APPROVED' : 
                     reviewAction === 'reject' ? 'REJECTED' : 'REVISION_REQUIRED'

    setListings(listings.map(listing =>
      listing.id === selectedListing.id
        ? {
            ...listing,
            status: newStatus as any,
            reviewedAt: new Date().toISOString(),
            reviewedBy: '管理员',
            reviewNotes,
          }
        : listing
    ))

    setShowReviewModal(false)
    setSelectedListing(null)
    setReviewNotes('')
  }

  const pendingCount = listings.filter(l => l.status === 'PENDING').length
  const approvedCount = listings.filter(l => l.status === 'APPROVED').length
  const rejectedCount = listings.filter(l => l.status === 'REJECTED').length
  const revisionCount = listings.filter(l => l.status === 'REVISION_REQUIRED').length

  return (
    <>
      <SessionIdentityBar
        subjectName="数据交易平台"
        roleName="平台管理员"
        tenantId="tenant_platform_001"
        scope="admin:listings:approve"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">商品审核</h1>
          <p className="text-gray-600">审核供应商提交的数据商品，确保质量和合规性</p>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-yellow-100 rounded-lg flex items-center justify-center">
                <Clock className="w-6 h-6 text-yellow-600" />
              </div>
              <span className="text-2xl font-bold text-gray-900">{pendingCount}</span>
            </div>
            <h3 className="text-sm font-medium text-gray-600">待审核</h3>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center">
                <CheckCircle className="w-6 h-6 text-green-600" />
              </div>
              <span className="text-2xl font-bold text-gray-900">{approvedCount}</span>
            </div>
            <h3 className="text-sm font-medium text-gray-600">已批准</h3>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-orange-100 rounded-lg flex items-center justify-center">
                <AlertTriangle className="w-6 h-6 text-orange-600" />
              </div>
              <span className="text-2xl font-bold text-gray-900">{revisionCount}</span>
            </div>
            <h3 className="text-sm font-medium text-gray-600">需修改</h3>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-red-100 rounded-lg flex items-center justify-center">
                <XCircle className="w-6 h-6 text-red-600" />
              </div>
              <span className="text-2xl font-bold text-gray-900">{rejectedCount}</span>
            </div>
            <h3 className="text-sm font-medium text-gray-600">已拒绝</h3>
          </div>
        </div>

        {/* 搜索和筛选 */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
          <div className="flex flex-col md:flex-row gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
              <input
                type="text"
                placeholder="搜索商品名称或供应商..."
                value={searchKeyword}
                onChange={(e) => setSearchKeyword(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>

            <select
              value={selectedStatus}
              onChange={(e) => setSelectedStatus(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部状态</option>
              <option value="PENDING">待审核</option>
              <option value="APPROVED">已批准</option>
              <option value="REJECTED">已拒绝</option>
              <option value="REVISION_REQUIRED">需修改</option>
            </select>
          </div>
        </div>

        {/* 商品列表 */}
        <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">商品信息</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">供应商</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">定价模型</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">质量评分</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">风险等级</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">提交时间</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {filteredListings.map((listing) => {
                const statusConfig = STATUS_CONFIG[listing.status]
                const StatusIcon = statusConfig.icon
                const riskConfig = RISK_CONFIG[listing.riskLevel]

                return (
                  <tr key={listing.id} className="hover:bg-gray-50">
                    <td className="py-4 px-6">
                      <div className="font-medium text-gray-900 mb-1">{listing.title}</div>
                      <div className="flex items-center gap-2 text-xs text-gray-500">
                        <Tag className="w-3 h-3" />
                        <span>{listing.industry}</span>
                        <span>•</span>
                        <span>{listing.dataType}</span>
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">{listing.supplierName}</div>
                      <div className="text-xs text-gray-500">{listing.supplierId}</div>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <div className="text-sm text-gray-900">{listing.pricingModel}</div>
                      <div className="text-xs text-gray-500">
                        ¥{listing.basePrice.toLocaleString()}{listing.pricingModel === '按量计费' ? '/次' : '/月'}
                      </div>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <div className="flex items-center justify-center gap-2">
                        <div className="w-16 bg-gray-200 rounded-full h-2">
                          <div
                            className={`h-2 rounded-full ${
                              listing.qualityScore >= 90 ? 'bg-green-500' :
                              listing.qualityScore >= 80 ? 'bg-yellow-500' : 'bg-red-500'
                            }`}
                            style={{ width: `${listing.qualityScore}%` }}
                          />
                        </div>
                        <span className="text-sm font-medium text-gray-900">{listing.qualityScore}</span>
                      </div>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className={`inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium ${riskConfig.color}`}>
                        <Shield className="w-3 h-3" />
                        {riskConfig.label}
                      </span>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className={`status-tag ${statusConfig.color}`}>
                        <StatusIcon className="w-3.5 h-3.5" />
                        <span>{statusConfig.label}</span>
                      </span>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">{listing.submittedAt.split(' ')[0]}</div>
                      <div className="text-xs text-gray-500">{listing.submittedAt.split(' ')[1]}</div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="flex items-center justify-end gap-2">
                        <button
                          onClick={() => handleReview(listing)}
                          className="p-2 text-gray-600 hover:text-primary-600 hover:bg-primary-50 rounded-lg"
                          title="审核"
                        >
                          <Eye className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* 审核 Modal */}
      {showReviewModal && selectedListing && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="bg-white rounded-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto animate-fade-in">
            {/* Modal Header */}
            <div className="sticky top-0 bg-white border-b border-gray-200 px-8 py-6">
              <h2 className="text-2xl font-bold text-gray-900">商品审核</h2>
              <p className="text-sm text-gray-600 mt-1">{selectedListing.title}</p>
            </div>

            {/* Modal Body */}
            <div className="px-8 py-6 space-y-6">
              {/* 基本信息 */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-4">基本信息</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="text-sm text-gray-600">商品名称</label>
                    <p className="text-sm font-medium text-gray-900 mt-1">{selectedListing.title}</p>
                  </div>
                  <div>
                    <label className="text-sm text-gray-600">供应商</label>
                    <p className="text-sm font-medium text-gray-900 mt-1">{selectedListing.supplierName}</p>
                  </div>
                  <div>
                    <label className="text-sm text-gray-600">行业分类</label>
                    <p className="text-sm font-medium text-gray-900 mt-1">{selectedListing.industry}</p>
                  </div>
                  <div>
                    <label className="text-sm text-gray-600">数据类型</label>
                    <p className="text-sm font-medium text-gray-900 mt-1">{selectedListing.dataType}</p>
                  </div>
                  <div>
                    <label className="text-sm text-gray-600">定价模型</label>
                    <p className="text-sm font-medium text-gray-900 mt-1">{selectedListing.pricingModel}</p>
                  </div>
                  <div>
                    <label className="text-sm text-gray-600">基础价格</label>
                    <p className="text-sm font-medium text-gray-900 mt-1">
                      ¥{selectedListing.basePrice.toLocaleString()}{selectedListing.pricingModel === '按量计费' ? '/次' : '/月'}
                    </p>
                  </div>
                </div>
              </div>

              {/* 质量评估 */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-4">质量评估</h3>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="text-sm text-gray-600">质量评分</label>
                    <div className="flex items-center gap-3 mt-2">
                      <div className="flex-1 bg-gray-200 rounded-full h-3">
                        <div
                          className={`h-3 rounded-full ${
                            selectedListing.qualityScore >= 90 ? 'bg-green-500' :
                            selectedListing.qualityScore >= 80 ? 'bg-yellow-500' : 'bg-red-500'
                          }`}
                          style={{ width: `${selectedListing.qualityScore}%` }}
                        />
                      </div>
                      <span className="text-lg font-bold text-gray-900">{selectedListing.qualityScore}</span>
                    </div>
                  </div>
                  <div>
                    <label className="text-sm text-gray-600">风险等级</label>
                    <div className="mt-2">
                      <span className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm font-medium ${RISK_CONFIG[selectedListing.riskLevel].color}`}>
                        <Shield className="w-4 h-4" />
                        {RISK_CONFIG[selectedListing.riskLevel].label}
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              {/* 合规问题 */}
              {selectedListing.complianceIssues.length > 0 && (
                <div>
                  <h3 className="text-lg font-bold text-gray-900 mb-4">合规问题</h3>
                  <div className="bg-red-50 border border-red-200 rounded-lg p-4">
                    <ul className="space-y-2">
                      {selectedListing.complianceIssues.map((issue, index) => (
                        <li key={index} className="flex items-start gap-2 text-sm text-red-800">
                          <AlertTriangle className="w-4 h-4 flex-shrink-0 mt-0.5" />
                          <span>{issue}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
              )}

              {/* 审核操作 */}
              <div>
                <h3 className="text-lg font-bold text-gray-900 mb-4">审核操作</h3>
                <div className="space-y-4">
                  <div>
                    <label className="text-sm font-medium text-gray-700 mb-2 block">审核结果</label>
                    <div className="flex gap-3">
                      <button
                        onClick={() => setReviewAction('approve')}
                        className={`flex-1 px-4 py-3 rounded-lg border-2 font-medium transition-all ${
                          reviewAction === 'approve'
                            ? 'border-green-500 bg-green-50 text-green-700'
                            : 'border-gray-200 text-gray-700 hover:border-gray-300'
                        }`}
                      >
                        <CheckCircle className="w-5 h-5 mx-auto mb-1" />
                        批准上架
                      </button>
                      <button
                        onClick={() => setReviewAction('revision')}
                        className={`flex-1 px-4 py-3 rounded-lg border-2 font-medium transition-all ${
                          reviewAction === 'revision'
                            ? 'border-orange-500 bg-orange-50 text-orange-700'
                            : 'border-gray-200 text-gray-700 hover:border-gray-300'
                        }`}
                      >
                        <AlertTriangle className="w-5 h-5 mx-auto mb-1" />
                        要求修改
                      </button>
                      <button
                        onClick={() => setReviewAction('reject')}
                        className={`flex-1 px-4 py-3 rounded-lg border-2 font-medium transition-all ${
                          reviewAction === 'reject'
                            ? 'border-red-500 bg-red-50 text-red-700'
                            : 'border-gray-200 text-gray-700 hover:border-gray-300'
                        }`}
                      >
                        <XCircle className="w-5 h-5 mx-auto mb-1" />
                        拒绝上架
                      </button>
                    </div>
                  </div>

                  <div>
                    <label className="text-sm font-medium text-gray-700 mb-2 block">
                      审核意见 <span className="text-red-500">*</span>
                    </label>
                    <textarea
                      value={reviewNotes}
                      onChange={(e) => setReviewNotes(e.target.value)}
                      placeholder={
                        reviewAction === 'approve' ? '请说明批准理由...' :
                        reviewAction === 'revision' ? '请详细说明需要修改的内容...' :
                        '请说明拒绝理由...'
                      }
                      className="w-full px-4 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 min-h-[120px]"
                      required
                    />
                  </div>
                </div>
              </div>
            </div>

            {/* Modal Footer */}
            <div className="sticky bottom-0 bg-gray-50 border-t border-gray-200 px-8 py-4 flex gap-3">
              <button
                onClick={() => {
                  setShowReviewModal(false)
                  setSelectedListing(null)
                  setReviewNotes('')
                }}
                className="flex-1 px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-100 font-medium"
              >
                取消
              </button>
              <button
                onClick={handleSubmitReview}
                disabled={!reviewNotes.trim()}
                className={`flex-1 px-6 py-3 rounded-lg font-medium ${
                  reviewAction === 'approve' ? 'bg-green-600 hover:bg-green-700' :
                  reviewAction === 'revision' ? 'bg-orange-600 hover:bg-orange-700' :
                  'bg-red-600 hover:bg-red-700'
                } text-white disabled:opacity-50 disabled:cursor-not-allowed`}
              >
                提交审核
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
