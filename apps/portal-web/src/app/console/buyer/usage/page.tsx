'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import ApiCallsTrendChart from '@/components/charts/ApiCallsTrendChart'
import ResponseTimeChart from '@/components/charts/ResponseTimeChart'
import UsageDistributionChart from '@/components/charts/UsageDistributionChart'
import { 
  Activity,
  TrendingUp,
  TrendingDown,
  BarChart3,
  PieChart,
  Calendar,
  Download,
  Filter
} from 'lucide-react'

interface UsageStats {
  subscriptionId: string
  listingTitle: string
  supplierName: string
  totalCalls: number
  successCalls: number
  failedCalls: number
  avgResponseTime: number
  quota: number | null
  usedQuota: number
}

const MOCK_USAGE_STATS: UsageStats[] = [
  {
    subscriptionId: 'sub_001',
    listingTitle: '企业工商风险数据',
    supplierName: '天眼数据科技',
    totalCalls: 6580,
    successCalls: 6520,
    failedCalls: 60,
    avgResponseTime: 245,
    quota: 10000,
    usedQuota: 6580,
  },
  {
    subscriptionId: 'sub_002',
    listingTitle: '消费者行为分析数据',
    supplierName: '智慧消费研究院',
    totalCalls: 12350,
    successCalls: 12280,
    failedCalls: 70,
    avgResponseTime: 180,
    quota: 50000,
    usedQuota: 12350,
  },
  {
    subscriptionId: 'sub_003',
    listingTitle: '物流轨迹实时数据',
    supplierName: '智运物流数据',
    totalCalls: 8920,
    successCalls: 8850,
    failedCalls: 70,
    avgResponseTime: 320,
    quota: null,
    usedQuota: 8920,
  },
]

export default function BuyerUsagePage() {
  const [selectedPeriod, setSelectedPeriod] = useState('30d')
  const [selectedSubscription, setSelectedSubscription] = useState<string>('all')
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const totalCalls = MOCK_USAGE_STATS.reduce((sum, s) => sum + s.totalCalls, 0)
  const totalSuccess = MOCK_USAGE_STATS.reduce((sum, s) => sum + s.successCalls, 0)
  const totalFailed = MOCK_USAGE_STATS.reduce((sum, s) => sum + s.failedCalls, 0)
  const successRate = ((totalSuccess / totalCalls) * 100).toFixed(2)
  const avgResponseTime = Math.round(
    MOCK_USAGE_STATS.reduce((sum, s) => sum + s.avgResponseTime, 0) / MOCK_USAGE_STATS.length
  )

  const filteredStats = selectedSubscription === 'all'
    ? MOCK_USAGE_STATS
    : MOCK_USAGE_STATS.filter(s => s.subscriptionId === selectedSubscription)

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:usage:read"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">使用分析</h1>
            <p className="text-gray-600">查看您的 API 调用统计和使用趋势</p>
          </div>
          <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
            <Download className="w-4 h-4" />
            <span>导出报告</span>
          </button>
        </div>

        {/* 筛选器 */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
          <div className="flex items-center gap-4">
            <select
              value={selectedPeriod}
              onChange={(e) => setSelectedPeriod(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="7d">近 7 天</option>
              <option value="30d">近 30 天</option>
              <option value="90d">近 90 天</option>
              <option value="1y">近 1 年</option>
            </select>

            <select
              value={selectedSubscription}
              onChange={(e) => setSelectedSubscription(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部订阅</option>
              {MOCK_USAGE_STATS.map(stat => (
                <option key={stat.subscriptionId} value={stat.subscriptionId}>
                  {stat.listingTitle}
                </option>
              ))}
            </select>

            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>更多筛选</span>
            </button>
          </div>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center">
                <Activity className="w-6 h-6 text-blue-600" />
              </div>
              <div className="flex items-center gap-1 text-sm font-medium text-green-600">
                <TrendingUp className="w-4 h-4" />
                <span>+18%</span>
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{totalCalls.toLocaleString()}</div>
            <div className="text-sm text-gray-600">总调用次数</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center">
                <TrendingUp className="w-6 h-6 text-green-600" />
              </div>
              <div className="flex items-center gap-1 text-sm font-medium text-green-600">
                <TrendingUp className="w-4 h-4" />
                <span>+2%</span>
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{successRate}%</div>
            <div className="text-sm text-gray-600">成功率</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-red-50 rounded-lg flex items-center justify-center">
                <TrendingDown className="w-6 h-6 text-red-600" />
              </div>
              <div className="flex items-center gap-1 text-sm font-medium text-red-600">
                <TrendingDown className="w-4 h-4" />
                <span>-5%</span>
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{totalFailed}</div>
            <div className="text-sm text-gray-600">失败次数</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center">
                <BarChart3 className="w-6 h-6 text-purple-600" />
              </div>
              <div className="flex items-center gap-1 text-sm font-medium text-green-600">
                <TrendingUp className="w-4 h-4" />
                <span>-12ms</span>
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{avgResponseTime}ms</div>
            <div className="text-sm text-gray-600">平均响应时间</div>
          </div>
        </div>

        {/* 图表区域 */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
          {/* 调用趋势图 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">调用趋势</h2>
              <Calendar className="w-5 h-5 text-gray-400" />
            </div>
            <div className="h-80">
              <ApiCallsTrendChart />
            </div>
          </div>

          {/* 响应时间分布 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">响应时间分布</h2>
              <Activity className="w-5 h-5 text-gray-400" />
            </div>
            <div className="h-80">
              <ResponseTimeChart />
            </div>
          </div>
        </div>

        {/* 按订阅统计 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 订阅使用排行 */}
          <div className="lg:col-span-2 bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-6">订阅使用排行</h2>
            <div className="space-y-4">
              {filteredStats.map((stat, index) => {
                const successRate = ((stat.successCalls / stat.totalCalls) * 100).toFixed(1)
                const quotaPercentage = stat.quota ? Math.round((stat.usedQuota / stat.quota) * 100) : null

                return (
                  <div key={stat.subscriptionId} className="p-4 border border-gray-200 rounded-lg">
                    <div className="flex items-start justify-between mb-3">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-1">
                          <span className="text-2xl font-bold text-gray-400">#{index + 1}</span>
                          <h3 className="font-bold text-gray-900">{stat.listingTitle}</h3>
                        </div>
                        <p className="text-sm text-gray-600">{stat.supplierName}</p>
                      </div>
                      <div className="text-right">
                        <div className="text-2xl font-bold text-gray-900">{stat.totalCalls.toLocaleString()}</div>
                        <div className="text-xs text-gray-500">总调用</div>
                      </div>
                    </div>

                    <div className="grid grid-cols-4 gap-4 mb-3">
                      <div>
                        <div className="text-xs text-gray-500 mb-1">成功</div>
                        <div className="text-sm font-bold text-green-600">{stat.successCalls.toLocaleString()}</div>
                      </div>
                      <div>
                        <div className="text-xs text-gray-500 mb-1">失败</div>
                        <div className="text-sm font-bold text-red-600">{stat.failedCalls}</div>
                      </div>
                      <div>
                        <div className="text-xs text-gray-500 mb-1">成功率</div>
                        <div className="text-sm font-bold text-gray-900">{successRate}%</div>
                      </div>
                      <div>
                        <div className="text-xs text-gray-500 mb-1">响应时间</div>
                        <div className="text-sm font-bold text-gray-900">{stat.avgResponseTime}ms</div>
                      </div>
                    </div>

                    {stat.quota && (
                      <div>
                        <div className="flex items-center justify-between text-xs text-gray-600 mb-1">
                          <span>配额使用</span>
                          <span>{stat.usedQuota.toLocaleString()} / {stat.quota.toLocaleString()} ({quotaPercentage}%)</span>
                        </div>
                        <div className="w-full h-2 bg-gray-200 rounded-full overflow-hidden">
                          <div
                            className={`h-full transition-all ${
                              quotaPercentage! >= 80 ? 'bg-red-500' : quotaPercentage! >= 60 ? 'bg-yellow-500' : 'bg-green-500'
                            }`}
                            style={{ width: `${quotaPercentage}%` }}
                          />
                        </div>
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
          </div>

          {/* 调用分布 */}
          <div className="lg:col-span-1 bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">调用分布</h2>
              <PieChart className="w-5 h-5 text-gray-400" />
            </div>
            <div className="h-64">
              <UsageDistributionChart />
            </div>

            {/* 图例 */}
            <div className="space-y-3 mt-6">
              {filteredStats.map((stat, index) => {
                const percentage = ((stat.totalCalls / totalCalls) * 100).toFixed(1)
                const colors = ['bg-blue-500', 'bg-green-500', 'bg-purple-500', 'bg-yellow-500', 'bg-red-500']
                
                return (
                  <div key={stat.subscriptionId} className="flex items-center justify-between">
                    <div className="flex items-center gap-2 flex-1">
                      <div className={`w-3 h-3 rounded-full ${colors[index % colors.length]}`} />
                      <span className="text-sm text-gray-900 truncate">{stat.listingTitle}</span>
                    </div>
                    <span className="text-sm font-bold text-gray-900 ml-2">{percentage}%</span>
                  </div>
                )
              })}
            </div>
          </div>
        </div>

        {/* 错误分析 */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 mt-6">
          <h2 className="text-xl font-bold text-gray-900 mb-6">错误分析</h2>
          <div className="h-64 flex items-center justify-center text-gray-400">
            <div className="text-center">
              <TrendingDown className="w-16 h-16 mx-auto mb-4 opacity-50" />
              <p className="text-sm mb-2">错误分析图表待集成</p>
              <p className="text-xs text-gray-500">建议使用 ECharts 或 Recharts</p>
              <div className="mt-4 text-xs text-left bg-gray-50 p-4 rounded-lg max-w-md mx-auto">
                <p className="font-medium mb-2">图表应展示:</p>
                <ul className="space-y-1 text-gray-600">
                  <li>• 错误类型分布（4xx, 5xx, 超时等）</li>
                  <li>• 错误趋势（按时间）</li>
                  <li>• Top 错误码排行</li>
                  <li>• 错误率变化趋势</li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
