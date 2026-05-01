'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import SellerApiTrendChart from '@/components/charts/SellerApiTrendChart'
import SellerStatusCodeChart from '@/components/charts/SellerStatusCodeChart'
import SellerLatencyDistributionChart from '@/components/charts/SellerLatencyDistributionChart'
import { 
  Activity,
  TrendingUp,
  TrendingDown,
  Zap,
  AlertTriangle,
  CheckCircle,
  Download,
  ExternalLink,
  Filter,
} from 'lucide-react'
import { SELLER_API_CALLS } from '@/lib/seller-analytics-data'

const METHOD_COLORS: Record<string, string> = {
  GET: 'bg-blue-100 text-blue-800',
  POST: 'bg-green-100 text-green-800',
  PUT: 'bg-yellow-100 text-yellow-800',
  DELETE: 'bg-red-100 text-red-800',
}

export default function SellerAnalyticsPage() {
  const router = useRouter()
  const [selectedPeriod, setSelectedPeriod] = useState<string>('24hours')
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  // 计算统计数据
  const totalCalls = 125000
  const lastPeriodCalls = 105000
  const callsGrowth = ((totalCalls - lastPeriodCalls) / lastPeriodCalls * 100).toFixed(1)

  const successCalls = 124750
  const successRate = ((successCalls / totalCalls) * 100).toFixed(2)

  const failedCalls = totalCalls - successCalls
  const failureRate = ((failedCalls / totalCalls) * 100).toFixed(2)

  const avgResponseTime = 105 // ms

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:analytics:read"
        sessionExpiresAt={sessionExpiresAt}
        userName="管理员"
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">调用看板</h1>
            <p className="text-gray-600">监控 API 调用情况和性能指标</p>
          </div>
          <div className="flex items-center gap-3">
            <select
              value={selectedPeriod}
              onChange={(e) => setSelectedPeriod(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="1hour">近 1 小时</option>
              <option value="24hours">近 24 小时</option>
              <option value="7days">近 7 天</option>
              <option value="30days">近 30 天</option>
            </select>
            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>筛选</span>
            </button>
            <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700">
              <Download className="w-4 h-4" />
              <span>导出数据</span>
            </button>
          </div>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center">
                <Activity className="w-6 h-6 text-blue-600" />
              </div>
              <div className={`flex items-center gap-1 text-sm font-medium ${
                Number(callsGrowth) >= 0 ? 'text-green-600' : 'text-red-600'
              }`}>
                {Number(callsGrowth) >= 0 ? (
                  <TrendingUp className="w-4 h-4" />
                ) : (
                  <TrendingDown className="w-4 h-4" />
                )}
                <span>{Math.abs(Number(callsGrowth))}%</span>
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {(totalCalls / 1000).toFixed(0)}K
            </div>
            <div className="text-sm text-gray-600">总调用次数</div>
            <div className="mt-2 text-xs text-gray-500">
              上期: {(lastPeriodCalls / 1000).toFixed(0)}K
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center">
                <CheckCircle className="w-6 h-6 text-green-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {successRate}%
            </div>
            <div className="text-sm text-gray-600">成功率</div>
            <div className="mt-2 text-xs text-green-600 flex items-center gap-1">
              <TrendingUp className="w-3 h-3" />
              <span>+0.2% 本期</span>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-red-50 rounded-lg flex items-center justify-center">
                <AlertTriangle className="w-6 h-6 text-red-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {failedCalls.toLocaleString()}
            </div>
            <div className="text-sm text-gray-600">失败次数</div>
            <div className="mt-2 text-xs text-gray-500">
              失败率: {failureRate}%
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center">
                <Zap className="w-6 h-6 text-purple-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {avgResponseTime}ms
            </div>
            <div className="text-sm text-gray-600">平均响应时间</div>
            <div className="mt-2 text-xs text-green-600 flex items-center gap-1">
              <TrendingDown className="w-3 h-3" />
              <span>-12ms 本期</span>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
          {/* 调用趋势图 */}
          <div className="lg:col-span-2 bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">调用趋势</h2>
              <div className="flex items-center gap-2">
                <button className="px-3 py-1 text-sm bg-primary-100 text-primary-700 rounded-lg font-medium">
                  小时
                </button>
                <button className="px-3 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded-lg font-medium">
                  日
                </button>
                <button className="px-3 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded-lg font-medium">
                  周
                </button>
              </div>
            </div>

            <div className="h-80">
              <SellerApiTrendChart />
            </div>

            {/* 图例 */}
            <div className="flex items-center justify-center gap-6 mt-6">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                <span className="text-sm text-gray-600">成功</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-red-500 rounded-full"></div>
                <span className="text-sm text-gray-600">失败</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-blue-500 rounded-full"></div>
                <span className="text-sm text-gray-600">总计</span>
              </div>
            </div>
          </div>

          {/* 状态码分布 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-6">状态码分布</h2>

            <div className="h-64 mb-6">
              <SellerStatusCodeChart />
            </div>

            {/* 图例 */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                  <span className="text-sm text-gray-700">200 OK</span>
                </div>
                <div className="text-sm font-medium text-gray-900">99.5%</div>
              </div>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-yellow-500 rounded-full"></div>
                  <span className="text-sm text-gray-700">4xx 错误</span>
                </div>
                <div className="text-sm font-medium text-gray-900">0.3%</div>
              </div>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-red-500 rounded-full"></div>
                  <span className="text-sm text-gray-700">5xx 错误</span>
                </div>
                <div className="text-sm font-medium text-gray-900">0.2%</div>
              </div>
            </div>
          </div>
        </div>

        {/* 响应时间分布 */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
          <h2 className="text-xl font-bold text-gray-900 mb-6">响应时间分布</h2>

          <div className="h-64">
            <SellerLatencyDistributionChart />
          </div>

          {/* 统计数据 */}
          <div className="grid grid-cols-5 gap-4 mt-6">
            <div className="text-center">
              <div className="text-xs text-gray-600 mb-1">P50</div>
              <div className="text-lg font-bold text-gray-900">85ms</div>
            </div>
            <div className="text-center">
              <div className="text-xs text-gray-600 mb-1">P90</div>
              <div className="text-lg font-bold text-gray-900">150ms</div>
            </div>
            <div className="text-center">
              <div className="text-xs text-gray-600 mb-1">P95</div>
              <div className="text-lg font-bold text-gray-900">220ms</div>
            </div>
            <div className="text-center">
              <div className="text-xs text-gray-600 mb-1">P99</div>
              <div className="text-lg font-bold text-gray-900">450ms</div>
            </div>
            <div className="text-center">
              <div className="text-xs text-gray-600 mb-1">最大</div>
              <div className="text-lg font-bold text-gray-900">3500ms</div>
            </div>
          </div>
        </div>

        {/* 最近调用记录 */}
        <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <div className="p-6 border-b border-gray-200">
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-bold text-gray-900">最近调用记录</h2>
              <span className="text-xs text-gray-500">双击行可查看请求详情</span>
            </div>
          </div>

          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">时间</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">客户</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">商品</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">接口</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">方法</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态码</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">响应时间</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">结果</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {SELLER_API_CALLS.map((call) => (
                <tr key={call.id} className="hover:bg-gray-50 cursor-pointer" onDoubleClick={() => router.push(`/console/seller/analytics/calls/${call.id}`)}>
                  <td className="py-4 px-6">
                    <div className="text-xs text-gray-900 font-mono">{call.timestamp}</div>
                  </td>
                  <td className="py-4 px-6">
                    <div className="text-sm text-gray-900">{call.customer}</div>
                  </td>
                  <td className="py-4 px-6">
                    <div className="text-sm text-gray-900">{call.listing}</div>
                  </td>
                  <td className="py-4 px-6">
                    <code className="text-xs font-mono text-gray-700 bg-gray-50 px-2 py-1 rounded">
                      {call.endpoint}
                    </code>
                  </td>
                  <td className="py-4 px-6 text-center">
                    <span className={`inline-flex items-center px-2 py-1 rounded text-xs font-medium ${METHOD_COLORS[call.method]}`}>
                      {call.method}
                    </span>
                  </td>
                  <td className="py-4 px-6 text-center">
                    <span className={`inline-flex items-center px-2 py-1 rounded text-xs font-medium ${
                      call.statusCode === 200 ? 'bg-green-100 text-green-800' :
                      call.statusCode >= 400 && call.statusCode < 500 ? 'bg-yellow-100 text-yellow-800' :
                      'bg-red-100 text-red-800'
                    }`}>
                      {call.statusCode}
                    </span>
                  </td>
                  <td className="py-4 px-6 text-right">
                    <div className={`text-sm font-medium ${
                      call.responseTime < 200 ? 'text-green-600' :
                      call.responseTime < 500 ? 'text-yellow-600' :
                      'text-red-600'
                    }`}>
                      {call.responseTime}ms
                    </div>
                  </td>
                  <td className="py-4 px-6 text-center">
                    {call.success ? (
                      <CheckCircle className="w-5 h-5 text-green-600 mx-auto" />
                    ) : (
                      <AlertTriangle className="w-5 h-5 text-red-600 mx-auto" />
                    )}
                  </td>
                  <td className="py-4 px-6 text-right">
                    <button onClick={() => router.push(`/console/seller/analytics/calls/${call.id}`)} className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-md border border-primary-300 text-primary-700 hover:bg-primary-50">
                      <ExternalLink className="w-3.5 h-3.5" />
                      详情
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>

          {/* 分页 */}
          <div className="p-4 border-t border-gray-200 flex items-center justify-between">
            <div className="text-sm text-gray-600">
              显示最近 {SELLER_API_CALLS.length} 条调用记录
            </div>
            <button className="text-sm text-primary-600 hover:text-primary-700 font-medium">
              查看更多 →
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
