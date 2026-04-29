'use client'

import ConsoleLayout from '@/components/console/ConsoleLayout'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Users, 
  Package, 
  AlertTriangle,
  Activity,
  TrendingUp,
  TrendingDown,
  Shield,
  AlertCircle
} from 'lucide-react'

// Mock 数据
const STATS = [
  {
    id: 'pending_subjects',
    label: '待审核主体',
    value: '5',
    change: '+2',
    trend: 'up',
    icon: Users,
    color: 'text-yellow-600',
    bgColor: 'bg-yellow-50',
    urgent: true,
  },
  {
    id: 'pending_listings',
    label: '待审核商品',
    value: '8',
    change: '+3',
    trend: 'up',
    icon: Package,
    color: 'text-orange-600',
    bgColor: 'bg-orange-50',
    urgent: true,
  },
  {
    id: 'risk_alerts',
    label: '风险告警',
    value: '12',
    change: '-2',
    trend: 'down',
    icon: AlertTriangle,
    color: 'text-red-600',
    bgColor: 'bg-red-50',
    urgent: true,
  },
  {
    id: 'chain_failures',
    label: '链上失败',
    value: '3',
    change: '0',
    trend: 'neutral',
    icon: Shield,
    color: 'text-purple-600',
    bgColor: 'bg-purple-50',
    urgent: false,
  },
]

const PLATFORM_STATS = [
  { label: '总主体数', value: '1,245', change: '+28 本月' },
  { label: '总商品数', value: '856', change: '+45 本月' },
  { label: '总订阅数', value: '3,420', change: '+156 本月' },
  { label: '总交易额', value: '¥8.5M', change: '+18% 本月' },
]

const PENDING_SUBJECTS = [
  {
    id: 'subject_001',
    name: '某某数据科技有限公司',
    type: 'SUPPLIER',
    creditCode: '91110000****0000XX',
    submittedAt: '2026-04-28 10:30',
    riskLevel: 'LOW',
  },
  {
    id: 'subject_002',
    name: '智慧金融服务有限公司',
    type: 'BUYER',
    creditCode: '91110000****0001XX',
    submittedAt: '2026-04-28 09:15',
    riskLevel: 'MEDIUM',
  },
  {
    id: 'subject_003',
    name: '大数据研究院',
    type: 'SUPPLIER',
    creditCode: '91110000****0002XX',
    submittedAt: '2026-04-27 16:45',
    riskLevel: 'HIGH',
  },
]

const PENDING_LISTINGS = [
  {
    id: 'listing_001',
    title: '新能源汽车数据',
    supplier: '某某数据科技',
    industry: '能源',
    submittedAt: '2026-04-28 14:20',
    riskLevel: 'LOW',
  },
  {
    id: 'listing_002',
    title: '医疗影像数据集',
    supplier: '医疗大数据中心',
    industry: '医疗',
    submittedAt: '2026-04-28 11:30',
    riskLevel: 'HIGH',
  },
]

const RISK_ALERTS = [
  {
    id: 'alert_001',
    type: 'CHAIN_FAILURE',
    severity: 'HIGH',
    message: '订单 order_12345 链上提交失败，已重试 3 次',
    time: '10 分钟前',
  },
  {
    id: 'alert_002',
    type: 'PROJECTION_OUT_OF_SYNC',
    severity: 'MEDIUM',
    message: '申请 req_67890 投影状态不一致',
    time: '30 分钟前',
  },
  {
    id: 'alert_003',
    type: 'NOTIFICATION_FAILURE',
    severity: 'LOW',
    message: '通知 notif_11111 发送失败',
    time: '1 小时前',
  },
]

const RECENT_ACTIVITIES = [
  {
    id: 'act_001',
    type: 'SUBJECT_APPROVED',
    message: '主体"某某科技"审核通过',
    operator: '张三',
    time: '2026-04-28 15:30',
  },
  {
    id: 'act_002',
    type: 'LISTING_REJECTED',
    message: '商品"敏感数据集"审核拒绝',
    operator: '李四',
    time: '2026-04-28 14:20',
  },
  {
    id: 'act_003',
    type: 'CHAIN_RETRY',
    message: '订单 order_12345 手动重试链上提交',
    operator: '王五',
    time: '2026-04-28 13:10',
  },
]

const RISK_CONFIG: Record<string, { label: string; color: string }> = {
  LOW: { label: '低风险', color: 'text-green-600 bg-green-50' },
  MEDIUM: { label: '中风险', color: 'text-yellow-600 bg-yellow-50' },
  HIGH: { label: '高风险', color: 'text-red-600 bg-red-50' },
}

export default function AdminDashboard() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  return (
    <ConsoleLayout role="admin">
      <SessionIdentityBar
        subjectName="数据交易平台运营中心"
        roleName="平台管理员"
        tenantId="tenant_platform_001"
        scope="admin:all:write"
        sessionExpiresAt={sessionExpiresAt}
        userName="管理员"
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">平台运营中心</h1>
          <p className="text-gray-600">监控平台运行状态，处理审核和风险事件</p>
        </div>

        {/* 紧急待办卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {STATS.map((stat) => {
            const Icon = stat.icon
            return (
              <div
                key={stat.id}
                className={`bg-white rounded-xl border-2 p-6 ${
                  stat.urgent ? 'border-red-200 shadow-lg' : 'border-gray-200'
                }`}
              >
                <div className="flex items-center justify-between mb-4">
                  <div className={`w-12 h-12 ${stat.bgColor} rounded-lg flex items-center justify-center`}>
                    <Icon className={`w-6 h-6 ${stat.color}`} />
                  </div>
                  {stat.trend !== 'neutral' && (
                    <div className={`flex items-center gap-1 text-sm font-medium ${
                      stat.trend === 'up' ? 'text-red-600' : 'text-green-600'
                    }`}>
                      {stat.trend === 'up' ? (
                        <TrendingUp className="w-4 h-4" />
                      ) : (
                        <TrendingDown className="w-4 h-4" />
                      )}
                      <span>{stat.change}</span>
                    </div>
                  )}
                </div>
                <div className="text-2xl font-bold text-gray-900 mb-1">{stat.value}</div>
                <div className="text-sm text-gray-600">{stat.label}</div>
                {stat.urgent && (
                  <div className="mt-3 pt-3 border-t border-gray-200">
                    <a href="#" className="text-sm text-primary-600 hover:text-primary-700 font-medium">
                      立即处理 →
                    </a>
                  </div>
                )}
              </div>
            )
          })}
        </div>

        {/* 平台统计 */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
          {PLATFORM_STATS.map((stat, index) => (
            <div key={index} className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="text-sm text-gray-600 mb-2">{stat.label}</div>
              <div className="text-2xl font-bold text-gray-900 mb-1">{stat.value}</div>
              <div className="text-xs text-green-600">{stat.change}</div>
            </div>
          ))}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 待审核主体 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">待审核主体</h2>
              <a href="/admin/console/subjects" className="text-sm text-primary-600 hover:text-primary-700 font-medium">
                查看全部 →
              </a>
            </div>

            <div className="space-y-3">
              {PENDING_SUBJECTS.map((subject) => (
                <div
                  key={subject.id}
                  className="p-4 border border-gray-200 rounded-lg hover:border-primary-300 transition-colors"
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <h3 className="font-medium text-gray-900 mb-1">{subject.name}</h3>
                      <div className="text-xs text-gray-600">
                        {subject.type === 'SUPPLIER' ? '供应商' : '买方'} · {subject.creditCode}
                      </div>
                    </div>
                    <span className={`text-xs px-2 py-1 rounded-full font-medium ${RISK_CONFIG[subject.riskLevel].color}`}>
                      {RISK_CONFIG[subject.riskLevel].label}
                    </span>
                  </div>
                  <div className="flex items-center justify-between text-xs text-gray-500">
                    <span>{subject.submittedAt}</span>
                    <button className="text-primary-600 hover:text-primary-700 font-medium">
                      审核
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 待审核商品 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">待审核商品</h2>
              <a href="/admin/console/listings" className="text-sm text-primary-600 hover:text-primary-700 font-medium">
                查看全部 →
              </a>
            </div>

            <div className="space-y-3">
              {PENDING_LISTINGS.map((listing) => (
                <div
                  key={listing.id}
                  className="p-4 border border-gray-200 rounded-lg hover:border-primary-300 transition-colors"
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <h3 className="font-medium text-gray-900 mb-1">{listing.title}</h3>
                      <div className="text-xs text-gray-600">
                        {listing.supplier} · {listing.industry}
                      </div>
                    </div>
                    <span className={`text-xs px-2 py-1 rounded-full font-medium ${RISK_CONFIG[listing.riskLevel].color}`}>
                      {RISK_CONFIG[listing.riskLevel].label}
                    </span>
                  </div>
                  <div className="flex items-center justify-between text-xs text-gray-500">
                    <span>{listing.submittedAt}</span>
                    <button className="text-primary-600 hover:text-primary-700 font-medium">
                      审核
                    </button>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 风险告警 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">风险告警</h2>
              <Activity className="w-5 h-5 text-gray-400" />
            </div>

            <div className="space-y-3">
              {RISK_ALERTS.map((alert) => (
                <div
                  key={alert.id}
                  className={`p-3 rounded-lg border ${
                    alert.severity === 'HIGH'
                      ? 'bg-red-50 border-red-200'
                      : alert.severity === 'MEDIUM'
                      ? 'bg-yellow-50 border-yellow-200'
                      : 'bg-blue-50 border-blue-200'
                  }`}
                >
                  <div className="flex items-start gap-2">
                    <AlertCircle className={`w-4 h-4 flex-shrink-0 mt-0.5 ${
                      alert.severity === 'HIGH'
                        ? 'text-red-600'
                        : alert.severity === 'MEDIUM'
                        ? 'text-yellow-600'
                        : 'text-blue-600'
                    }`} />
                    <div className="flex-1">
                      <p className={`text-xs font-medium mb-1 ${
                        alert.severity === 'HIGH'
                          ? 'text-red-900'
                          : alert.severity === 'MEDIUM'
                          ? 'text-yellow-900'
                          : 'text-blue-900'
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

        {/* 最近活动 */}
        <div className="mt-6 bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-xl font-bold text-gray-900 mb-6">最近活动</h2>
          <div className="space-y-4">
            {RECENT_ACTIVITIES.map((activity) => (
              <div key={activity.id} className="flex items-start gap-4 pb-4 border-b border-gray-100 last:border-b-0">
                <div className="w-2 h-2 bg-primary-600 rounded-full mt-2"></div>
                <div className="flex-1">
                  <p className="text-sm text-gray-900 mb-1">{activity.message}</p>
                  <div className="flex items-center gap-3 text-xs text-gray-500">
                    <span>操作人: {activity.operator}</span>
                    <span>·</span>
                    <span>{activity.time}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </ConsoleLayout>
  )
}
