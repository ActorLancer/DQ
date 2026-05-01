'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import SellerRevenueTrendChart from '@/components/charts/SellerRevenueTrendChart'
import SellerRevenueCompositionChart from '@/components/charts/SellerRevenueCompositionChart'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  DollarSign,
  TrendingUp,
  TrendingDown,
  Calendar,
  Download,
  ExternalLink,
  Filter,
  Package,
  Users,
  Activity,
} from 'lucide-react'
import { SELLER_REVENUE_RECORDS } from '@/lib/seller-revenue-data'

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
  const router = useRouter()
  const [selectedPeriod, setSelectedPeriod] = useState<string>('30days')
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  // 计算统计数据
  const totalRevenue = SELLER_REVENUE_RECORDS.filter(r => r.status === 'PAID').reduce((sum, r) => sum + r.amount, 0)
  const lastMonthRevenue = 28500 // Mock 上月数据
  const revenueGrowth = ((totalRevenue - lastMonthRevenue) / lastMonthRevenue * 100).toFixed(1)

  const subscriptionRevenue = SELLER_REVENUE_RECORDS
    .filter(r => r.type === 'SUBSCRIPTION' && r.status === 'PAID')
    .reduce((sum, r) => sum + r.amount, 0)
  
  const renewalRevenue = SELLER_REVENUE_RECORDS
    .filter(r => r.type === 'RENEWAL' && r.status === 'PAID')
    .reduce((sum, r) => sum + r.amount, 0)

  const avgOrderValue = totalRevenue / SELLER_REVENUE_RECORDS.filter(r => r.status === 'PAID').length

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

            <div className="h-80">
              <SellerRevenueTrendChart />
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

            <div className="h-64 mb-6">
              <SellerRevenueCompositionChart />
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
            <div className="flex items-center justify-between">
              <h2 className="text-xl font-bold text-gray-900">收入明细</h2>
              <span className="text-xs text-gray-500">双击行可查看明细</span>
            </div>
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
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {SELLER_REVENUE_RECORDS.map((record) => {
                const typeConfig = TYPE_CONFIG[record.type]
                const statusConfig = STATUS_CONFIG[record.status]

                return (
                  <tr key={record.id} className="hover:bg-gray-50 cursor-pointer" onDoubleClick={() => router.push(`/console/seller/revenue/records/${record.id}`)}>
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
                    <td className="py-4 px-6 text-right">
                      <button onClick={() => router.push(`/console/seller/revenue/records/${record.id}`)} className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-md border border-primary-300 text-primary-700 hover:bg-primary-50">
                        <ExternalLink className="w-3.5 h-3.5" />
                        详情
                      </button>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>

          {/* 分页 */}
          <div className="p-4 border-t border-gray-200 flex items-center justify-between">
            <div className="text-sm text-gray-600">
              显示 1-{SELLER_REVENUE_RECORDS.length} 条，共 {SELLER_REVENUE_RECORDS.length} 条
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
