'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  DollarSign,
  TrendingUp,
  TrendingDown,
  Calendar,
  Download,
  Filter,
  Package,
  Users,
  Activity,
  BarChart3
} from 'lucide-react'

interface RevenueRecord {
  id: string
  date: string
  customer: string
  listing: string
  plan: string
  amount: number
  type: 'SUBSCRIPTION' | 'RENEWAL' | 'UPGRADE' | 'ONE_TIME'
  status: 'PAID' | 'PENDING' | 'REFUNDED'
}

const MOCK_REVENUE: RevenueRecord[] = [
  {
    id: 'rev_001',
    date: '2026-04-29',
    customer: '某某金融科技',
    listing: '企业工商风险数据',
    plan: '企业版',
    amount: 9999,
    type: 'SUBSCRIPTION',
    status: 'PAID',
  },
  {
    id: 'rev_002',
    date: '2026-04-28',
    customer: '智慧物流数据中心',
    listing: '物流轨迹实时数据',
    plan: '标准版',
    amount: 999,
    type: 'RENEWAL',
    status: 'PAID',
  },
  {
    id: 'rev_003',
    date: '2026-04-27',
    customer: '某某数据分析公司',
    listing: '企业工商风险数据',
    plan: '标准版',
    amount: 999,
    type: 'SUBSCRIPTION',
    status: 'PAID',
  },
  {
    id: 'rev_004',
    date: '2026-04-26',
    customer: '某某金融科技',
    listing: '金融市场行情数据',
    plan: '企业版',
    amount: 19999,
    type: 'UPGRADE',
    status: 'PAID',
  },
  {
    id: 'rev_005',
    date: '2026-04-25',
    customer: '智慧物流数据中心',
    listing: '企业工商风险数据',
    plan: '标准版',
    amount: 999,
    type: 'SUBSCRIPTION',
    status: 'PAID',
  },
]

const TYPE_CONFIG = {
  SUBSCRIPTION: { label: '新订阅', color: 'bg-green-100 text-green-800' },
  RENEWAL: { label: '续订', color: 'bg-blue-100 text-blue-800' },
  UPGRADE: { label: '升级', color: 'bg-purple-100 text-purple-800' },
  ONE_TIME: { label: '一次性', color: 'bg-yellow-100 text-yellow-800' },
}

const STATUS_CONFIG = {
  PAID: { label: '已支付', color: 'bg-green-100 text-green-800' },
  PENDING: { label: '待支付', color: 'bg-yellow-100 text-yellow-800' },
  REFUNDED: { label: '已退款', color: 'bg-red-100 text-red-800' },
}

export default function SellerRevenuePage() {
  const [selectedPeriod, setSelectedPeriod] = useState<string>('30days')
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  // 计算统计数据
  const totalRevenue = MOCK_REVENUE.filter(r => r.status === 'PAID').reduce((sum, r) => sum + r.amount, 0)
  const lastMonthRevenue = 28500 // Mock 上月数据
  const revenueGrowth = ((totalRevenue - lastMonthRevenue) / lastMonthRevenue * 100).toFixed(1)

  const subscriptionRevenue = MOCK_REVENUE
    .filter(r => r.type === 'SUBSCRIPTION' && r.status === 'PAID')
    .reduce((sum, r) => sum + r.amount, 0)
  
  const renewalRevenue = MOCK_REVENUE
    .filter(r => r.type === 'RENEWAL' && r.status === 'PAID')
    .reduce((sum, r) => sum + r.amount, 0)

  const avgOrderValue = totalRevenue / MOCK_REVENUE.filter(r => r.status === 'PAID').length

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:revenue:read"
        sessionExpiresAt={sessionExpiresAt}
        userName="管理员"
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">收入看板</h1>
            <p className="text-gray-600">查看和分析收入数据</p>
          </div>
          <div className="flex items-center gap-3">
            <select
              value={selectedPeriod}
              onChange={(e) => setSelectedPeriod(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="7days">近 7 天</option>
              <option value="30days">近 30 天</option>
              <option value="90days">近 90 天</option>
              <option value="1year">近 1 年</option>
            </select>
            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>筛选</span>
            </button>
            <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700">
              <Download className="w-4 h-4" />
              <span>导出报表</span>
            </button>
          </div>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center">
                <DollarSign className="w-6 h-6 text-green-600" />
              </div>
              <div className={`flex items-center gap-1 text-sm font-medium ${
                Number(revenueGrowth) >= 0 ? 'text-green-600' : 'text-red-600'
              }`}>
                {Number(revenueGrowth) >= 0 ? (
                  <TrendingUp className="w-4 h-4" />
                ) : (
                  <TrendingDown className="w-4 h-4" />
                )}
                <span>{Math.abs(Number(revenueGrowth))}%</span>
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              ¥{totalRevenue.toLocaleString()}
            </div>
            <div className="text-sm text-gray-600">总收入</div>
            <div className="mt-2 text-xs text-gray-500">
              上月: ¥{lastMonthRevenue.toLocaleString()}
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center">
                <Package className="w-6 h-6 text-blue-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              ¥{subscriptionRevenue.toLocaleString()}
            </div>
            <div className="text-sm text-gray-600">新订阅收入</div>
            <div className="mt-2 text-xs text-gray-500">
              占比: {((subscriptionRevenue / totalRevenue) * 100).toFixed(1)}%
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center">
                <Activity className="w-6 h-6 text-purple-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              ¥{renewalRevenue.toLocaleString()}
            </div>
            <div className="text-sm text-gray-600">续订收入</div>
            <div className="mt-2 text-xs text-gray-500">
              占比: {((renewalRevenue / totalRevenue) * 100).toFixed(1)}%
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-yellow-50 rounded-lg flex items-center justify-center">
                <Users className="w-6 h-6 text-yellow-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              ¥{avgOrderValue.toLocaleString()}
            </div>
            <div className="text-sm text-gray-600">平均订单金额</div>
            <div className="mt-2 text-xs text-green-600 flex items-center gap-1">
              <TrendingUp className="w-3 h-3" />
              <span>+12% 本月</span>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 收入趋势图 */}
          <div className="lg:col-span-2 bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-6">
              <h2 className="text-xl font-bold text-gray-900">收入趋势</h2>
              <div className="flex items-center gap-2">
                <button className="px-3 py-1 text-sm bg-primary-100 text-primary-700 rounded-lg font-medium">
                  日
                </button>
                <button className="px-3 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded-lg font-medium">
                  周
                </button>
                <button className="px-3 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded-lg font-medium">
                  月
                </button>
              </div>
            </div>

            {/* 图表占位 */}
            <div className="h-80 flex items-center justify-center border-2 border-dashed border-gray-200 rounded-lg">
              <div className="text-center">
                <BarChart3 className="w-16 h-16 text-gray-300 mx-auto mb-4" />
                <p className="text-gray-600 mb-2">收入趋势图表</p>
                <p className="text-sm text-gray-500">集成 ECharts 或 Recharts 后显示</p>
              </div>
            </div>

            {/* 图例 */}
            <div className="flex items-center justify-center gap-6 mt-6">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                <span className="text-sm text-gray-600">新订阅</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-blue-500 rounded-full"></div>
                <span className="text-sm text-gray-600">续订</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 bg-purple-500 rounded-full"></div>
                <span className="text-sm text-gray-600">升级</span>
              </div>
            </div>
          </div>

          {/* 收入构成 */}
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-6">收入构成</h2>

            {/* 饼图占位 */}
            <div className="h-64 flex items-center justify-center border-2 border-dashed border-gray-200 rounded-lg mb-6">
              <div className="text-center">
                <Activity className="w-12 h-12 text-gray-300 mx-auto mb-2" />
                <p className="text-sm text-gray-600">饼图</p>
              </div>
            </div>

            {/* 图例 */}
            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                  <span className="text-sm text-gray-700">新订阅</span>
                </div>
                <div className="text-sm font-medium text-gray-900">
                  {((subscriptionRevenue / totalRevenue) * 100).toFixed(1)}%
                </div>
              </div>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-blue-500 rounded-full"></div>
                  <span className="text-sm text-gray-700">续订</span>
                </div>
                <div className="text-sm font-medium text-gray-900">
                  {((renewalRevenue / totalRevenue) * 100).toFixed(1)}%
                </div>
              </div>
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 bg-purple-500 rounded-full"></div>
                  <span className="text-sm text-gray-700">升级</span>
                </div>
                <div className="text-sm font-medium text-gray-900">
                  {(((totalRevenue - subscriptionRevenue - renewalRevenue) / totalRevenue) * 100).toFixed(1)}%
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* 收入明细 */}
        <div className="mt-6 bg-white rounded-xl border border-gray-200 overflow-hidden">
          <div className="p-6 border-b border-gray-200">
            <h2 className="text-xl font-bold text-gray-900">收入明细</h2>
          </div>

          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">日期</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">客户</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">商品</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">套餐</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">类型</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">金额</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {MOCK_REVENUE.map((record) => {
                const typeConfig = TYPE_CONFIG[record.type]
                const statusConfig = STATUS_CONFIG[record.status]

                return (
                  <tr key={record.id} className="hover:bg-gray-50">
                    <td className="py-4 px-6">
                      <div className="flex items-center gap-2 text-sm text-gray-900">
                        <Calendar className="w-4 h-4 text-gray-400" />
                        <span>{record.date}</span>
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">{record.customer}</div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">{record.listing}</div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">{record.plan}</div>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${typeConfig.color}`}>
                        {typeConfig.label}
                      </span>
                    </td>
                    <td className="py-4 px-6 text-right">
                      <div className="text-sm font-medium text-gray-900">
                        ¥{record.amount.toLocaleString()}
                      </div>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${statusConfig.color}`}>
                        {statusConfig.label}
                      </span>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>

          {/* 分页 */}
          <div className="p-4 border-t border-gray-200 flex items-center justify-between">
            <div className="text-sm text-gray-600">
              显示 1-{MOCK_REVENUE.length} 条，共 {MOCK_REVENUE.length} 条
            </div>
            <div className="flex items-center gap-2">
              <button className="px-3 py-1 border border-gray-300 rounded text-sm hover:bg-gray-50 disabled:opacity-50" disabled>
                上一页
              </button>
              <button className="px-3 py-1 bg-primary-600 text-white rounded text-sm">
                1
              </button>
              <button className="px-3 py-1 border border-gray-300 rounded text-sm hover:bg-gray-50" disabled>
                下一页
              </button>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
