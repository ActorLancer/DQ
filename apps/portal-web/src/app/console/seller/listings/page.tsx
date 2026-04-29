'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import Link from 'next/link'
import { 
  Plus, 
  Package,
  Search, 
  Filter, 
  MoreVertical,
  Eye,
  Edit,
  Copy,
  Trash2,
  TrendingUp,
  Users,
  Shield,
  AlertCircle,
  CheckCircle,
  Clock
} from 'lucide-react'
import type { ListingStatus } from '@/types'

interface ListingItem {
  id: string
  title: string
  industry: string
  status: ListingStatus
  pricingModel: string
  deliveryMethods: string[]
  qualityScore: number
  requestCount: number
  subscriberCount: number
  updatedAt: string
  chainStatus: string
  projectionStatus: string
}

const MOCK_LISTINGS: ListingItem[] = [
  {
    id: 'listing_001',
    title: '企业工商风险数据',
    industry: '金融',
    status: 'LISTED',
    pricingModel: '月订阅',
    deliveryMethods: ['API', 'FILE'],
    qualityScore: 9.2,
    requestCount: 45,
    subscriberCount: 28,
    updatedAt: '2026-04-20',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
  },
  {
    id: 'listing_002',
    title: '消费者行为分析数据',
    industry: '消费',
    status: 'LISTED',
    pricingModel: '年订阅',
    deliveryMethods: ['API'],
    qualityScore: 8.8,
    requestCount: 32,
    subscriberCount: 18,
    updatedAt: '2026-04-18',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
  },
  {
    id: 'listing_003',
    title: '物流轨迹实时数据',
    industry: '交通',
    status: 'PENDING_REVIEW',
    pricingModel: '按量计费',
    deliveryMethods: ['API'],
    qualityScore: 9.5,
    requestCount: 0,
    subscriberCount: 0,
    updatedAt: '2026-04-25',
    chainStatus: 'NOT_SUBMITTED',
    projectionStatus: 'PENDING',
  },
  {
    id: 'listing_004',
    title: '医疗健康知识图谱',
    industry: '医疗',
    status: 'DRAFT',
    pricingModel: '定制',
    deliveryMethods: ['API', 'FILE'],
    qualityScore: 0,
    requestCount: 0,
    subscriberCount: 0,
    updatedAt: '2026-04-22',
    chainStatus: 'NOT_SUBMITTED',
    projectionStatus: 'PENDING',
  },
]

const STATUS_CONFIG: Record<ListingStatus, { label: string; color: string; icon: any }> = {
  DRAFT: { label: '草稿', color: 'bg-gray-100 text-gray-800', icon: Edit },
  PENDING_REVIEW: { label: '待审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: AlertCircle },
  LISTED: { label: '已上架', color: 'bg-blue-100 text-blue-800', icon: CheckCircle },
  SUSPENDED: { label: '已暂停', color: 'bg-orange-100 text-orange-800', icon: AlertCircle },
  DELISTED: { label: '已下架', color: 'bg-gray-100 text-gray-800', icon: AlertCircle },
}

export default function SellerListingsPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredListings = MOCK_LISTINGS.filter((listing) => {
    const matchesKeyword = listing.title.toLowerCase().includes(searchKeyword.toLowerCase())
    const matchesStatus = selectedStatus === 'all' || listing.status === selectedStatus
    return matchesKeyword && matchesStatus
  })

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
        {/* 页面标题和操作 */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">商品管理</h1>
            <p className="text-gray-600">管理您的数据商品，查看申请和订阅情况</p>
          </div>
          <Link
            href="/console/seller/listings/create"
            className="flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
          >
            <Plus className="w-5 h-5" />
            <span>创建商品</span>
          </Link>
        </div>

        {/* 筛选和搜索 */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
          <div className="flex items-center gap-4">
            {/* 搜索框 */}
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
              <input
                type="text"
                value={searchKeyword}
                onChange={(e) => setSearchKeyword(e.target.value)}
                placeholder="搜索商品名称..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>

            {/* 状态筛选 */}
            <select
              value={selectedStatus}
              onChange={(e) => setSelectedStatus(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部状态</option>
              <option value="DRAFT">草稿</option>
              <option value="PENDING_REVIEW">待审核</option>
              <option value="LISTED">已上架</option>
              <option value="SUSPENDED">已暂停</option>
            </select>

            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>更多筛选</span>
            </button>
          </div>
        </div>

        {/* 商品列表 */}
        <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">商品信息</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">状态</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">价格模式</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">交付方式</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">质量评分</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">申请数</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">订阅客户</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">链状态</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {filteredListings.map((listing) => {
                const statusConfig = STATUS_CONFIG[listing.status]
                const StatusIcon = statusConfig.icon
                
                return (
                  <tr key={listing.id} className="hover:bg-gray-50">
                    <td className="py-4 px-6">
                      <div>
                        <div className="font-medium text-gray-900 mb-1">{listing.title}</div>
                        <div className="flex items-center gap-2 text-sm text-gray-600">
                          <span className="tag">{listing.industry}</span>
                          <span className="text-xs text-gray-400">更新于 {listing.updatedAt}</span>
                        </div>
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      <span className={`status-tag ${statusConfig.color}`}>
                        <StatusIcon className="w-3.5 h-3.5" />
                        <span>{statusConfig.label}</span>
                      </span>
                    </td>
                    <td className="py-4 px-6">
                      <span className="text-sm text-gray-900">{listing.pricingModel}</span>
                    </td>
                    <td className="py-4 px-6">
                      <div className="flex flex-wrap gap-1">
                        {listing.deliveryMethods.map((method) => (
                          <span key={method} className="tag-primary text-xs">
                            {method}
                          </span>
                        ))}
                      </div>
                    </td>
                    <td className="py-4 px-6 text-center">
                      {listing.qualityScore > 0 ? (
                        <span className="font-medium text-gray-900">{listing.qualityScore}</span>
                      ) : (
                        <span className="text-gray-400">-</span>
                      )}
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className="font-medium text-gray-900">{listing.requestCount}</span>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className="font-medium text-gray-900">{listing.subscriberCount}</span>
                    </td>
                    <td className="py-4 px-6">
                      <div className="flex items-center gap-1">
                        {listing.chainStatus === 'CONFIRMED' ? (
                          <Shield className="w-4 h-4 text-success-600" />
                        ) : (
                          <Clock className="w-4 h-4 text-gray-400" />
                        )}
                        <span className="text-xs text-gray-600">
                          {listing.chainStatus === 'CONFIRMED' ? '已确认' : '未提交'}
                        </span>
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="flex items-center justify-end gap-2">
                        <button
                          className="p-2 text-gray-600 hover:text-primary-600 hover:bg-primary-50 rounded-lg"
                          title="查看"
                        >
                          <Eye className="w-4 h-4" />
                        </button>
                        <button
                          className="p-2 text-gray-600 hover:text-primary-600 hover:bg-primary-50 rounded-lg"
                          title="编辑"
                        >
                          <Edit className="w-4 h-4" />
                        </button>
                        <button
                          className="p-2 text-gray-600 hover:text-primary-600 hover:bg-primary-50 rounded-lg"
                          title="复制"
                        >
                          <Copy className="w-4 h-4" />
                        </button>
                        <button
                          className="p-2 text-gray-600 hover:text-gray-600 hover:bg-gray-100 rounded-lg"
                          title="更多"
                        >
                          <MoreVertical className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>

          {filteredListings.length === 0 && (
            <div className="text-center py-12">
              <div className="text-gray-400 mb-4">
                <Package className="w-16 h-16 mx-auto opacity-50" />
              </div>
              <h3 className="text-lg font-medium text-gray-900 mb-2">暂无商品</h3>
              <p className="text-gray-600 mb-6">开始创建您的第一个数据商品</p>
              <Link
                href="/console/seller/listings/create"
                className="inline-flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
              >
                <Plus className="w-5 h-5" />
                <span>创建商品</span>
              </Link>
            </div>
          )}
        </div>

        {/* 分页 */}
        {filteredListings.length > 0 && (
          <div className="flex items-center justify-between mt-6">
            <div className="text-sm text-gray-600">
              共 {filteredListings.length} 个商品
            </div>
            <div className="flex items-center gap-2">
              <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed">
                上一页
              </button>
              <button className="px-4 py-2 bg-primary-600 text-white rounded-lg">1</button>
              <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
                2
              </button>
              <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
                下一页
              </button>
            </div>
          </div>
        )}
      </div>
    </>
  )
}
