'use client'

import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Package, 
  FileText, 
  Key, 
  TrendingUp,
  AlertCircle,
  CheckCircle,
  Clock,
  Activity,
  Calendar,
  DollarSign
} from 'lucide-react'

// Mock 数据
const STATS = [
  {
    id: 'subscriptions',
    label: '活跃订阅',
    value: '8',
    change: '+2',
    trend: 'up',
    icon: Package,
    color: 'text-blue-600',
    bgColor: 'bg-blue-50',
  },
  {
    id: 'pending_requests',
    label: '待审批申请',
    value: '2',
    change: '0',
    trend: 'neutral',
    icon: FileText,
    color: 'text-yellow-600',
    bgColor: 'bg-yellow-50',
  },
  {
    id: 'api_calls',
    label: '本月 API 调用',
    value: '125,680',
    change: '+18%',
    trend: 'up',
    icon: Activity,
    color: 'text-green-600',
    bgColor: 'bg-green-50',
  },
  {
    id: 'spending',
    label: '本月支出',
    value: '¥28,500',
    change: '+8%',
    trend: 'up',
    icon: DollarSign,
    color: 'text-purple-600',
    bgColor: 'bg-purple-50',
  },
]

const ACTIVE_SUBSCRIPTIONS = [
  {
    id: 'sub_001',
    listingTitle: '企业工商风险数据',
    supplierName: '天眼数据科技',
    plan: '标准版',
    status: 'ACTIVE',
    quota: 10000,
    usedQuota: 6580,
    endsAt: '2026-05-28',
    apiCallsToday: 245,
  },
  {
    id: 'sub_002',
    listingTitle: '消费者行为分析数据',
    supplierName: '智慧消费研究院',
    plan: '企业版',
    status: 'ACTIVE',
    quota: 50000,
    usedQuota: 12350,
    endsAt: '2026-12-31',
    apiCallsToday: 892,
  },
  {
    id: 'sub_003',
    listingTitle: '物流轨迹实时数据',
    supplierName: '智运物流数据',
    plan: '按量计费',
    status: 'ACTIVE',
    quota: null,
    usedQuota: 8920,
    endsAt: null,
    apiCallsToday: 156,
  },
]

const RECENT_REQUESTS = [
  {
    id: 'req_001',
    listingTitle: '金融市场行情数据',
    supplierName: '金融数据服务',
    plan: '专业版',
    status: 'PENDING_SUPPLIER_REVIEW',
    createdAt: '2026-04-28 10:30',
  },
  {
    id: 'req_002',
    listingTitle: '医疗健康知识图谱',
    supplierName: '医疗大数据中心',
    plan: '定制版',
    status: 'NEED_MORE_INFO',
    createdAt: '2026-04-27 14:20',
  },
]

const ALERTS = [
  {
    id: 'alert_001',
    type: 'warning',
    message: '订阅"企业工商风险数据"将在 30 天后到期',
    time: '今天',
  },
  {
    id: 'alert_002',
    type: 'warning',
    message: '订阅"企业工商风险数据"已使用 65% 配额',
    time: '今天',
  },
  {
    id: 'alert_003',
    type: 'info',
    message: '申请"金融市场行情数据"正在审核中',
    time: '1 天前',
  },
]

const STATUS_CONFIG: Record<string, { label: string; color: string }> = {
  PENDING_SUPPLIER_REVIEW: { label: '待供应商审核', color: 'bg-yellow-100 text-yellow-800' },
  NEED_MORE_INFO: { label: '需补充材料', color: 'bg-orange-100 text-orange-800' },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800' },
}

export default function BuyerDashboard() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const getQuotaPercentage = (used: number, total: number) => {
    return Math.round((used / total) * 100)
  }

  const getQuotaColor = (percentage: number) => {
    if (percentage >= 80) return 'bg-red-500'
    if (percentage >= 60) return 'bg-yellow-500'
    return 'bg-green-500'
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:subscriptions:read"
        sessionExpiresAt={sessionExpiresAt}
        userName="李四"
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">仪表盘</h1>
          <p className="text-gray-600">欢迎回来，这是您的数据订阅概览</p>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {STATS.map((stat) => {
            const Icon = stat.icon
            return (
              <div key={stat.id} className="bg-white rounded-xl border border-gray-200 p-6">
                <div className="flex items-center justify-between mb-4">
                  <div className={`w-12 h-12 ${stat.bgColor} rounded-lg flex items-center justify-center`}>
                    <Icon className={`w-6 h-6 ${stat.color}`} />
                  </div>
                  {stat.trend !== 'neutral' && (
                    <div className={`flex items-center gap-1 text-sm font-medium ${
                      stat.trend === 'up' ? 'text-green-600' : 'text-red-600'
                    }`}>
                      <TrendingUp className="w-4 h-4" />
                      <span>{stat.change}</span>
                    </div>
                  )}
                </div>
                <div className="text-2xl font-bold text-gray-900 mb-1">{stat.value}</div>
                <div className="text-sm text-gray-600">{stat.label}</div>
              </div>
            )
          })}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 活跃订阅 */}
          <div className="lg:col-span-2 bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">活跃订阅</h2>
              <a href="/console/buyer/subscriptions" className="text-sm text-primary-600 hover:text-primary-700 font-medium">
                查看全部 →
              </a>
            </div>

            <div className="space-y-4">
              {ACTIVE_SUBSCRIPTIONS.map((sub) => {
                const percentage = sub.quota ? getQuotaPercentage(sub.usedQuota, sub.quota) : null
                const quotaColor = percentage ? getQuotaColor(percentage) : 'bg-blue-500'

                return (
                  <div
                    key={sub.id}
                    className="p-4 border border-gray-200 rounded-lg hover:border-primary-300 hover:shadow-sm transition-all"
                  >
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex-1">
                        <h3 className="font-medium text-gray-900 mb-1">{sub.listingTitle}</h3>
                        <div className="text-sm text-gray-600">
                          {sub.supplierName} · {sub.plan}
                        </div>
                      </div>
                      <span className="status-tag bg-green-100 text-green-800">
                        <CheckCircle className="w-3.5 h-3.5" />
                        <span>活跃</span>
                      </span>
                    </div>

                    {/* 配额进度条 */}
                    {sub.quota && (
                      <div className="mb-3">
                        <div className="flex items-center justify-between text-xs text-gray-600 mb-1">
                          <span>已使用 {sub.usedQuota.toLocaleString()} / {sub.quota.toLocaleString()}</span>
                          <span>{percentage}%</span>
                        </div>
                        <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
                          <div
                            className={`h-full ${quotaColor} transition-all`}
                            style={{ width: `${percentage}%` }}
                          />
                        </div>
                      </div>
                    )}

                    <div className="flex items-center justify-between text-xs text-gray-500">
                      <div className="flex items-center gap-4">
                        <div className="flex items-center gap-1">
                          <Activity className="w-3 h-3" />
                          <span>今日调用: {sub.apiCallsToday}</span>
                        </div>
                        {sub.endsAt && (
                          <div className="flex items-center gap-1">
                            <Calendar className="w-3 h-3" />
                            <span>到期: {sub.endsAt}</span>
                          </div>
                        )}
                      </div>
                      <button className="text-primary-600 hover:text-primary-700 font-medium">
                        管理
                      </button>
                    </div>
                  </div>
                )
              })}
            </div>
          </div>

          {/* 右侧栏 */}
          <div className="space-y-6">
            {/* 待审批申请 */}
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-bold text-gray-900">待审批申请</h2>
                <a href="/console/buyer/requests" className="text-sm text-primary-600 hover:text-primary-700 font-medium">
                  查看全部 →
                </a>
              </div>

              <div className="space-y-3">
                {RECENT_REQUESTS.map((request) => (
                  <div
                    key={request.id}
                    className="p-3 border border-gray-200 rounded-lg hover:border-primary-300 transition-colors"
                  >
                    <div className="font-medium text-sm text-gray-900 mb-1">
                      {request.listingTitle}
                    </div>
                    <div className="text-xs text-gray-600 mb-2">
                      {request.supplierName}
                    </div>
                    <div className="flex items-center justify-between">
                      <span className={`status-tag text-xs ${STATUS_CONFIG[request.status].color}`}>
                        {STATUS_CONFIG[request.status].label}
                      </span>
                      <span className="text-xs text-gray-500">{request.createdAt.split(' ')[0]}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* 系统提醒 */}
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center justify-between mb-6">
                <h2 className="text-xl font-bold text-gray-900">系统提醒</h2>
                <Activity className="w-5 h-5 text-gray-400" />
              </div>

              <div className="space-y-3">
                {ALERTS.map((alert) => (
                  <div
                    key={alert.id}
                    className={`p-3 rounded-lg border ${
                      alert.type === 'warning'
                        ? 'bg-yellow-50 border-yellow-200'
                        : 'bg-blue-50 border-blue-200'
                    }`}
                  >
                    <div className="flex items-start gap-2">
                      <AlertCircle className={`w-4 h-4 flex-shrink-0 mt-0.5 ${
                        alert.type === 'warning' ? 'text-yellow-600' : 'text-blue-600'
                      }`} />
                      <div className="flex-1">
                        <p className={`text-xs font-medium mb-1 ${
                          alert.type === 'warning' ? 'text-yellow-900' : 'text-blue-900'
                        }`}>
                          {alert.message}
                        </p>
                        <p className="text-xs text-gray-600">{alert.time}</p>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* 使用趋势图表 */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-6">近 30 日 API 调用趋势</h2>
            <div className="h-64 flex items-center justify-center text-gray-400">
              <div className="text-center">
                <Activity className="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p className="text-sm">图表组件待集成</p>
              </div>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-6">近 30 日支出趋势</h2>
            <div className="h-64 flex items-center justify-center text-gray-400">
              <div className="text-center">
                <DollarSign className="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p className="text-sm">图表组件待集成</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
