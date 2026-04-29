'use client'

import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  TrendingUp, 
  TrendingDown, 
  Package, 
  FileText, 
  Users, 
  DollarSign,
  AlertCircle,
  CheckCircle,
  Clock,
  Activity
} from 'lucide-react'

// Mock 数据
const STATS = [
  {
    id: 'listings',
    label: '已发布商品',
    value: '12',
    change: '+2',
    trend: 'up',
    icon: Package,
    color: 'text-blue-600',
    bgColor: 'bg-blue-50',
  },
  {
    id: 'pending_requests',
    label: '待处理申请',
    value: '3',
    change: '+1',
    trend: 'up',
    icon: FileText,
    color: 'text-yellow-600',
    bgColor: 'bg-yellow-50',
  },
  {
    id: 'subscribers',
    label: '活跃订阅客户',
    value: '28',
    change: '+5',
    trend: 'up',
    icon: Users,
    color: 'text-green-600',
    bgColor: 'bg-green-50',
  },
  {
    id: 'revenue',
    label: '本月收入',
    value: '¥128,500',
    change: '+12%',
    trend: 'up',
    icon: DollarSign,
    color: 'text-purple-600',
    bgColor: 'bg-purple-50',
  },
]

const RECENT_REQUESTS = [
  {
    id: 'req_001',
    buyerName: '某某科技有限公司',
    listingTitle: '企业工商风险数据',
    plan: '标准版',
    status: 'PENDING_REVIEW',
    createdAt: '2026-04-28 14:30',
  },
  {
    id: 'req_002',
    buyerName: '智慧消费研究院',
    listingTitle: '消费者行为分析数据',
    plan: '企业版',
    status: 'PENDING_REVIEW',
    createdAt: '2026-04-28 10:15',
  },
  {
    id: 'req_003',
    buyerName: '金融数据服务',
    listingTitle: '企业工商风险数据',
    plan: '企业版',
    status: 'NEED_MORE_INFO',
    createdAt: '2026-04-27 16:45',
  },
]

const ALERTS = [
  {
    id: 'alert_001',
    type: 'warning',
    message: '商品"企业工商风险数据"的 API 调用量接近配额上限',
    time: '10 分钟前',
  },
  {
    id: 'alert_002',
    type: 'error',
    message: '链上提交失败：订单 order_12345 需要重试',
    time: '1 小时前',
  },
  {
    id: 'alert_003',
    type: 'success',
    message: '新订阅客户"某某科技"已成功激活',
    time: '2 小时前',
  },
]

const STATUS_CONFIG: Record<string, { label: string; color: string }> = {
  PENDING_REVIEW: { label: '待审核', color: 'bg-yellow-100 text-yellow-800' },
  NEED_MORE_INFO: { label: '需补充', color: 'bg-orange-100 text-orange-800' },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800' },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800' },
}

export default function SellerDashboard() {
  // 计算会话过期时间（30 分钟后）
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:listings:write"
        sessionExpiresAt={sessionExpiresAt}
        userName="张三"
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">仪表盘</h1>
          <p className="text-gray-600">欢迎回来，这是您的数据商品运营概览</p>
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
                  <div className={`flex items-center gap-1 text-sm font-medium ${
                    stat.trend === 'up' ? 'text-green-600' : 'text-red-600'
                  }`}>
                    {stat.trend === 'up' ? (
                      <TrendingUp className="w-4 h-4" />
                    ) : (
                      <TrendingDown className="w-4 h-4" />
                    )}
                    <span>{stat.change}</span>
                  </div>
                </div>
                <div className="text-2xl font-bold text-gray-900 mb-1">{stat.value}</div>
                <div className="text-sm text-gray-600">{stat.label}</div>
              </div>
            )
          })}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 待处理申请 */}
          <div className="lg:col-span-2 bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">待处理申请</h2>
              <a href="/console/seller/requests" className="text-sm text-primary-600 hover:text-primary-700 font-medium">
                查看全部 →
              </a>
            </div>

            <div className="space-y-4">
              {RECENT_REQUESTS.map((request) => (
                <div
                  key={request.id}
                  className="flex items-center justify-between p-4 border border-gray-200 rounded-lg hover:border-primary-300 hover:shadow-sm transition-all"
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-3 mb-2">
                      <h3 className="font-medium text-gray-900">{request.buyerName}</h3>
                      <span className={`status-tag ${STATUS_CONFIG[request.status].color}`}>
                        {STATUS_CONFIG[request.status].label}
                      </span>
                    </div>
                    <div className="text-sm text-gray-600 mb-1">
                      商品: {request.listingTitle} · {request.plan}
                    </div>
                    <div className="flex items-center gap-1 text-xs text-gray-500">
                      <Clock className="w-3 h-3" />
                      <span>{request.createdAt}</span>
                    </div>
                  </div>
                  <div className="flex gap-2">
                    <button className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium">
                      审批
                    </button>
                    <button className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium">
                      详情
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 系统告警 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">系统告警</h2>
              <Activity className="w-5 h-5 text-gray-400" />
            </div>

            <div className="space-y-4">
              {ALERTS.map((alert) => (
                <div
                  key={alert.id}
                  className={`p-4 rounded-lg border ${
                    alert.type === 'error'
                      ? 'bg-red-50 border-red-200'
                      : alert.type === 'warning'
                      ? 'bg-yellow-50 border-yellow-200'
                      : 'bg-green-50 border-green-200'
                  }`}
                >
                  <div className="flex items-start gap-3">
                    {alert.type === 'error' ? (
                      <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
                    ) : alert.type === 'warning' ? (
                      <AlertCircle className="w-5 h-5 text-yellow-600 flex-shrink-0 mt-0.5" />
                    ) : (
                      <CheckCircle className="w-5 h-5 text-green-600 flex-shrink-0 mt-0.5" />
                    )}
                    <div className="flex-1">
                      <p className={`text-sm font-medium mb-1 ${
                        alert.type === 'error'
                          ? 'text-red-900'
                          : alert.type === 'warning'
                          ? 'text-yellow-900'
                          : 'text-green-900'
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

        {/* 图表区域 */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6">
          {/* 收入趋势 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-6">近 30 日收入趋势</h2>
            <div className="h-64 flex items-center justify-center text-gray-400">
              <div className="text-center">
                <Activity className="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p className="text-sm">图表组件待集成</p>
              </div>
            </div>
          </div>

          {/* 调用量趋势 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-6">近 30 日调用量趋势</h2>
            <div className="h-64 flex items-center justify-center text-gray-400">
              <div className="text-center">
                <Activity className="w-12 h-12 mx-auto mb-2 opacity-50" />
                <p className="text-sm">图表组件待集成</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
