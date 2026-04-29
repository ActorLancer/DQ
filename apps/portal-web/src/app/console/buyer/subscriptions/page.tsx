'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Search, 
  Filter, 
  Key,
  Activity,
  Calendar,
  TrendingUp,
  AlertCircle,
  CheckCircle,
  XCircle,
  Clock,
  RefreshCw,
  Eye,
  Shield
} from 'lucide-react'

interface Subscription {
  id: string
  subscriptionId: string
  listingTitle: string
  supplierName: string
  plan: string
  status: 'ACTIVE' | 'EXPIRED' | 'SUSPENDED' | 'REVOKED'
  quota: number | null
  usedQuota: number
  startsAt: string
  endsAt: string | null
  apiCallsToday: number
  apiCallsTotal: number
  lastCallAt: string
  chainProofId: string
}

const MOCK_SUBSCRIPTIONS: Subscription[] = [
  {
    id: 'sub_001',
    subscriptionId: 'subscription_20260328_001',
    listingTitle: '企业工商风险数据',
    supplierName: '天眼数据科技',
    plan: '标准版',
    status: 'ACTIVE',
    quota: 10000,
    usedQuota: 6580,
    startsAt: '2026-03-28',
    endsAt: '2026-05-28',
    apiCallsToday: 245,
    apiCallsTotal: 6580,
    lastCallAt: '2026-04-28 15:30:00',
    chainProofId: 'proof_sub_001',
  },
  {
    id: 'sub_002',
    subscriptionId: 'subscription_20260101_002',
    listingTitle: '消费者行为分析数据',
    supplierName: '智慧消费研究院',
    plan: '企业版',
    status: 'ACTIVE',
    quota: 50000,
    usedQuota: 12350,
    startsAt: '2026-01-01',
    endsAt: '2026-12-31',
    apiCallsToday: 892,
    apiCallsTotal: 12350,
    lastCallAt: '2026-04-28 16:15:00',
    chainProofId: 'proof_sub_002',
  },
  {
    id: 'sub_003',
    subscriptionId: 'subscription_20260410_003',
    listingTitle: '物流轨迹实时数据',
    supplierName: '智运物流数据',
    plan: '按量计费',
    status: 'ACTIVE',
    quota: null,
    usedQuota: 8920,
    startsAt: '2026-04-10',
    endsAt: null,
    apiCallsToday: 156,
    apiCallsTotal: 8920,
    lastCallAt: '2026-04-28 14:45:00',
    chainProofId: 'proof_sub_003',
  },
  {
    id: 'sub_004',
    subscriptionId: 'subscription_20251201_004',
    listingTitle: '金融市场行情数据',
    supplierName: '金融数据服务',
    plan: '专业版',
    status: 'EXPIRED',
    quota: 30000,
    usedQuota: 29850,
    startsAt: '2025-12-01',
    endsAt: '2026-03-01',
    apiCallsToday: 0,
    apiCallsTotal: 29850,
    lastCallAt: '2026-02-28 23:59:00',
    chainProofId: 'proof_sub_004',
  },
]

const STATUS_CONFIG = {
  ACTIVE: { label: '活跃', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  EXPIRED: { label: '已过期', color: 'bg-gray-100 text-gray-800', icon: Clock },
  SUSPENDED: { label: '已暂停', color: 'bg-orange-100 text-orange-800', icon: AlertCircle },
  REVOKED: { label: '已撤销', color: 'bg-red-100 text-red-800', icon: XCircle },
}

export default function BuyerSubscriptionsPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedSub, setSelectedSub] = useState<Subscription | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredSubscriptions = MOCK_SUBSCRIPTIONS.filter((sub) => {
    const matchesKeyword = 
      sub.listingTitle.toLowerCase().includes(searchKeyword.toLowerCase()) ||
      sub.supplierName.toLowerCase().includes(searchKeyword.toLowerCase())
    const matchesStatus = selectedStatus === 'all' || sub.status === selectedStatus
    return matchesKeyword && matchesStatus
  })

  const getQuotaPercentage = (used: number, total: number) => {
    return Math.round((used / total) * 100)
  }

  const getQuotaColor = (percentage: number) => {
    if (percentage >= 80) return 'bg-red-500'
    if (percentage >= 60) return 'bg-yellow-500'
    return 'bg-green-500'
  }

  const getDaysRemaining = (endsAt: string | null) => {
    if (!endsAt) return null
    const now = new Date()
    const end = new Date(endsAt)
    const diff = end.getTime() - now.getTime()
    return Math.ceil(diff / (1000 * 60 * 60 * 24))
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:subscriptions:read"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">我的订阅</h1>
          <p className="text-gray-600">管理您的数据订阅和 API 访问权限</p>
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
              <option value="ACTIVE">活跃</option>
              <option value="EXPIRED">已过期</option>
              <option value="SUSPENDED">已暂停</option>
            </select>

            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>更多筛选</span>
            </button>
          </div>
        </div>

        {/* 订阅列表 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 左侧列表 */}
          <div className="lg:col-span-2 space-y-4">
            {filteredSubscriptions.map((sub) => {
              const statusConfig = STATUS_CONFIG[sub.status]
              const StatusIcon = statusConfig.icon
              const isSelected = selectedSub?.id === sub.id
              const percentage = sub.quota ? getQuotaPercentage(sub.usedQuota, sub.quota) : null
              const quotaColor = percentage ? getQuotaColor(percentage) : 'bg-blue-500'
              const daysRemaining = getDaysRemaining(sub.endsAt)

              return (
                <div
                  key={sub.id}
                  onClick={() => setSelectedSub(sub)}
                  className={`bg-white rounded-xl border-2 p-6 cursor-pointer transition-all ${
                    isSelected
                      ? 'border-primary-500 shadow-lg'
                      : 'border-gray-200 hover:border-primary-300 hover:shadow-md'
                  }`}
                >
                  {/* 头部 */}
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex-1">
                      <h3 className="text-lg font-bold text-gray-900 mb-2">{sub.listingTitle}</h3>
                      <div className="flex items-center gap-3 mb-2">
                        <span className="text-sm text-gray-600">{sub.supplierName}</span>
                        <span className="text-sm text-gray-400">·</span>
                        <span className="text-sm font-medium text-gray-900">{sub.plan}</span>
                      </div>
                    </div>
                    <span className={`status-tag ${statusConfig.color}`}>
                      <StatusIcon className="w-3.5 h-3.5" />
                      <span>{statusConfig.label}</span>
                    </span>
                  </div>

                  {/* 配额进度 */}
                  {sub.quota && (
                    <div className="mb-4">
                      <div className="flex items-center justify-between text-xs text-gray-600 mb-2">
                        <span>配额使用情况</span>
                        <span>{sub.usedQuota.toLocaleString()} / {sub.quota.toLocaleString()} ({percentage}%)</span>
                      </div>
                      <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
                        <div
                          className={`h-full ${quotaColor} transition-all`}
                          style={{ width: `${percentage}%` }}
                        />
                      </div>
                    </div>
                  )}

                  {/* 统计信息 */}
                  <div className="grid grid-cols-3 gap-4 mb-4 pb-4 border-b border-gray-100">
                    <div>
                      <div className="text-xs text-gray-500 mb-1">今日调用</div>
                      <div className="text-lg font-bold text-gray-900">{sub.apiCallsToday}</div>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">总调用</div>
                      <div className="text-lg font-bold text-gray-900">{sub.apiCallsTotal.toLocaleString()}</div>
                    </div>
                    {daysRemaining !== null && (
                      <div>
                        <div className="text-xs text-gray-500 mb-1">剩余天数</div>
                        <div className={`text-lg font-bold ${
                          daysRemaining < 30 ? 'text-red-600' : 'text-gray-900'
                        }`}>
                          {daysRemaining}
                        </div>
                      </div>
                    )}
                  </div>

                  {/* 底部信息 */}
                  <div className="flex items-center justify-between text-xs text-gray-500">
                    <div className="flex items-center gap-4">
                      <div className="flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        <span>{sub.startsAt} ~ {sub.endsAt || '无限期'}</span>
                      </div>
                    </div>
                    {sub.status === 'ACTIVE' && (
                      <div className="flex gap-2">
                        <button className="text-primary-600 hover:text-primary-700 font-medium">
                          查看 API Key
                        </button>
                        <span className="text-gray-300">|</span>
                        <button className="text-primary-600 hover:text-primary-700 font-medium">
                          续订
                        </button>
                      </div>
                    )}
                  </div>

                  {/* 到期警告 */}
                  {daysRemaining !== null && daysRemaining < 30 && sub.status === 'ACTIVE' && (
                    <div className="mt-4 p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
                      <div className="flex items-center gap-2 text-sm text-yellow-800">
                        <AlertCircle className="w-4 h-4" />
                        <span>订阅将在 {daysRemaining} 天后到期，请及时续订</span>
                      </div>
                    </div>
                  )}
                </div>
              )
            })}
          </div>

          {/* 右侧详情 */}
          <div className="lg:col-span-1">
            {selectedSub ? (
              <div className="bg-white rounded-xl border border-gray-200 p-6 sticky top-28">
                <h3 className="text-lg font-bold text-gray-900 mb-6">订阅详情</h3>

                <div className="space-y-6">
                  {/* Subscription ID */}
                  <div>
                    <div className="text-xs text-gray-500 mb-1">Subscription ID</div>
                    <code className="font-mono text-xs text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                      {selectedSub.subscriptionId}
                    </code>
                  </div>

                  {/* 商品信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">商品信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">商品:</span>
                        <span className="font-medium text-gray-900">{selectedSub.listingTitle}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">供应商:</span>
                        <span className="font-medium text-gray-900">{selectedSub.supplierName}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">套餐:</span>
                        <span className="font-medium text-gray-900">{selectedSub.plan}</span>
                      </div>
                    </div>
                  </div>

                  {/* 授权范围 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">授权范围</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">开始时间:</span>
                        <span className="font-medium text-gray-900">{selectedSub.startsAt}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">结束时间:</span>
                        <span className="font-medium text-gray-900">{selectedSub.endsAt || '无限期'}</span>
                      </div>
                      {selectedSub.quota && (
                        <div className="flex justify-between">
                          <span className="text-gray-600">配额:</span>
                          <span className="font-medium text-gray-900">{selectedSub.quota.toLocaleString()}</span>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* 使用情况 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">使用情况</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">今日调用:</span>
                        <span className="font-medium text-gray-900">{selectedSub.apiCallsToday}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">总调用:</span>
                        <span className="font-medium text-gray-900">{selectedSub.apiCallsTotal.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">最近调用:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedSub.lastCallAt}</span>
                      </div>
                    </div>
                  </div>

                  {/* 链上凭证 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">链上凭证</div>
                    <div className="flex items-center gap-2 p-3 bg-green-50 border border-green-200 rounded-lg">
                      <Shield className="w-4 h-4 text-success-600" />
                      <span className="text-xs text-success-800 font-medium">已链上存证</span>
                    </div>
                  </div>

                  {/* 操作按钮 */}
                  {selectedSub.status === 'ACTIVE' && (
                    <div className="space-y-2 pt-4 border-t border-gray-200">
                      <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                        <Key className="w-4 h-4" />
                        <span>查看 API Key</span>
                      </button>
                      <button className="w-full flex items-center justify-center gap-2 px-4 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                        <Activity className="w-4 h-4" />
                        <span>查看调用日志</span>
                      </button>
                      <button className="w-full flex items-center justify-center gap-2 px-4 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                        <RefreshCw className="w-4 h-4" />
                        <span>续订</span>
                      </button>
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <div className="bg-white rounded-xl border border-gray-200 p-12 text-center sticky top-28">
                <Eye className="w-12 h-12 mx-auto text-gray-300 mb-4" />
                <p className="text-gray-600">选择一个订阅查看详情</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
