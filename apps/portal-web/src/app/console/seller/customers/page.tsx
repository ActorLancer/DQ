'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Search,
  Filter,
  Users,
  TrendingUp,
  DollarSign,
  Activity,
  Eye,
  Mail,
  Phone,
  Calendar,
  Package,
  AlertCircle,
  CheckCircle,
  Clock
} from 'lucide-react'

interface Customer {
  id: string
  subjectId: string
  name: string
  type: 'ENTERPRISE' | 'INDIVIDUAL'
  industry: string
  contactPerson: string
  contactEmail: string
  contactPhone: string
  subscriptionCount: number
  activeSubscriptions: number
  totalRevenue: number
  totalCalls: number
  avgResponseTime: number
  successRate: number
  status: 'ACTIVE' | 'INACTIVE' | 'SUSPENDED'
  firstSubscribeDate: string
  lastActiveDate: string
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
}

const MOCK_CUSTOMERS: Customer[] = [
  {
    id: 'customer_001',
    subjectId: 'subject_buyer_001',
    name: '某某金融科技有限公司',
    type: 'ENTERPRISE',
    industry: '金融',
    contactPerson: '张三',
    contactEmail: 'zhangsan@example.com',
    contactPhone: '13800138000',
    subscriptionCount: 3,
    activeSubscriptions: 2,
    totalRevenue: 29997,
    totalCalls: 125000,
    avgResponseTime: 120,
    successRate: 99.8,
    status: 'ACTIVE',
    firstSubscribeDate: '2026-01-15',
    lastActiveDate: '2026-04-29 14:30',
    riskLevel: 'LOW',
  },
  {
    id: 'customer_002',
    subjectId: 'subject_buyer_002',
    name: '智慧物流数据中心',
    type: 'ENTERPRISE',
    industry: '物流',
    contactPerson: '李四',
    contactEmail: 'lisi@example.com',
    contactPhone: '13900139000',
    subscriptionCount: 2,
    activeSubscriptions: 2,
    totalRevenue: 19998,
    totalCalls: 85000,
    avgResponseTime: 95,
    successRate: 99.9,
    status: 'ACTIVE',
    firstSubscribeDate: '2026-02-20',
    lastActiveDate: '2026-04-29 10:15',
    riskLevel: 'LOW',
  },
  {
    id: 'customer_003',
    subjectId: 'subject_buyer_003',
    name: '某某咨询服务公司',
    type: 'ENTERPRISE',
    industry: '企业服务',
    contactPerson: '王五',
    contactEmail: 'wangwu@example.com',
    contactPhone: '13700137000',
    subscriptionCount: 1,
    activeSubscriptions: 0,
    totalRevenue: 999,
    totalCalls: 5000,
    avgResponseTime: 150,
    successRate: 98.5,
    status: 'INACTIVE',
    firstSubscribeDate: '2026-03-10',
    lastActiveDate: '2026-04-10 16:20',
    riskLevel: 'MEDIUM',
  },
  {
    id: 'customer_004',
    subjectId: 'subject_buyer_004',
    name: '某某数据分析公司',
    type: 'ENTERPRISE',
    industry: '数据服务',
    contactPerson: '赵六',
    contactEmail: 'zhaoliu@example.com',
    contactPhone: '13600136000',
    subscriptionCount: 4,
    activeSubscriptions: 3,
    totalRevenue: 49996,
    totalCalls: 250000,
    avgResponseTime: 85,
    successRate: 99.95,
    status: 'ACTIVE',
    firstSubscribeDate: '2025-12-01',
    lastActiveDate: '2026-04-29 15:45',
    riskLevel: 'LOW',
  },
]

const STATUS_CONFIG = {
  ACTIVE: { label: '活跃', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  INACTIVE: { label: '不活跃', color: 'bg-gray-100 text-gray-800', icon: Clock },
  SUSPENDED: { label: '已暂停', color: 'bg-red-100 text-red-800', icon: AlertCircle },
}

const RISK_CONFIG = {
  LOW: { label: '低风险', color: 'text-green-600 bg-green-50' },
  MEDIUM: { label: '中风险', color: 'text-yellow-600 bg-yellow-50' },
  HIGH: { label: '高风险', color: 'text-red-600 bg-red-50' },
}

export default function SellerCustomersPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedCustomer, setSelectedCustomer] = useState<Customer | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredCustomers = MOCK_CUSTOMERS.filter((customer) => {
    const matchesKeyword = 
      customer.name.toLowerCase().includes(searchKeyword.toLowerCase()) ||
      customer.contactPerson.toLowerCase().includes(searchKeyword.toLowerCase())
    const matchesStatus = selectedStatus === 'all' || customer.status === selectedStatus
    return matchesKeyword && matchesStatus
  })

  const stats = {
    total: MOCK_CUSTOMERS.length,
    active: MOCK_CUSTOMERS.filter(c => c.status === 'ACTIVE').length,
    totalRevenue: MOCK_CUSTOMERS.reduce((sum, c) => sum + c.totalRevenue, 0),
    totalCalls: MOCK_CUSTOMERS.reduce((sum, c) => sum + c.totalCalls, 0),
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:customers:read"
        sessionExpiresAt={sessionExpiresAt}
        userName="管理员"
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">订阅客户</h1>
          <p className="text-gray-600">管理和查看订阅您商品的客户</p>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center">
                <Users className="w-6 h-6 text-blue-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{stats.total}</div>
            <div className="text-sm text-gray-600">总客户数</div>
            <div className="mt-2 text-xs text-green-600">活跃: {stats.active}</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center">
                <DollarSign className="w-6 h-6 text-green-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              ¥{stats.totalRevenue.toLocaleString()}
            </div>
            <div className="text-sm text-gray-600">累计收入</div>
            <div className="mt-2 text-xs text-green-600 flex items-center gap-1">
              <TrendingUp className="w-3 h-3" />
              <span>+18% 本月</span>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center">
                <Activity className="w-6 h-6 text-purple-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {(stats.totalCalls / 1000).toFixed(0)}K
            </div>
            <div className="text-sm text-gray-600">总调用次数</div>
            <div className="mt-2 text-xs text-green-600 flex items-center gap-1">
              <TrendingUp className="w-3 h-3" />
              <span>+25% 本月</span>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-yellow-50 rounded-lg flex items-center justify-center">
                <Package className="w-6 h-6 text-yellow-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {MOCK_CUSTOMERS.reduce((sum, c) => sum + c.activeSubscriptions, 0)}
            </div>
            <div className="text-sm text-gray-600">活跃订阅数</div>
            <div className="mt-2 text-xs text-gray-500">
              总订阅: {MOCK_CUSTOMERS.reduce((sum, c) => sum + c.subscriptionCount, 0)}
            </div>
          </div>
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
                placeholder="搜索客户名称或联系人..."
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
              <option value="INACTIVE">不活跃</option>
              <option value="SUSPENDED">已暂停</option>
            </select>

            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>更多筛选</span>
            </button>
          </div>
        </div>

        {/* 客户列表 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 左侧列表 */}
          <div className="lg:col-span-2 space-y-4">
            {filteredCustomers.map((customer) => {
              const statusConfig = STATUS_CONFIG[customer.status]
              const StatusIcon = statusConfig.icon
              const isSelected = selectedCustomer?.id === customer.id

              return (
                <div
                  key={customer.id}
                  onClick={() => setSelectedCustomer(customer)}
                  className={`bg-white rounded-xl border-2 p-6 cursor-pointer transition-all ${
                    isSelected
                      ? 'border-primary-500 shadow-lg'
                      : 'border-gray-200 hover:border-primary-300 hover:shadow-md'
                  }`}
                >
                  {/* 头部 */}
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex-1">
                      <h3 className="text-lg font-bold text-gray-900 mb-2">{customer.name}</h3>
                      <div className="flex items-center gap-2 mb-2">
                        <span className="text-xs px-2 py-1 bg-blue-100 text-blue-800 rounded-full font-medium">
                          {customer.industry}
                        </span>
                        <span className={`text-xs px-2 py-1 rounded-full font-medium ${RISK_CONFIG[customer.riskLevel].color}`}>
                          {RISK_CONFIG[customer.riskLevel].label}
                        </span>
                      </div>
                    </div>
                    <span className={`status-tag ${statusConfig.color}`}>
                      <StatusIcon className="w-3.5 h-3.5" />
                      <span>{statusConfig.label}</span>
                    </span>
                  </div>

                  {/* 统计信息 */}
                  <div className="grid grid-cols-4 gap-4 mb-4 pb-4 border-b border-gray-100">
                    <div>
                      <div className="text-xs text-gray-500 mb-1">订阅数</div>
                      <div className="text-lg font-bold text-gray-900">
                        {customer.activeSubscriptions}/{customer.subscriptionCount}
                      </div>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">累计收入</div>
                      <div className="text-lg font-bold text-gray-900">
                        ¥{(customer.totalRevenue / 1000).toFixed(1)}K
                      </div>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">总调用</div>
                      <div className="text-lg font-bold text-gray-900">
                        {(customer.totalCalls / 1000).toFixed(0)}K
                      </div>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">成功率</div>
                      <div className="text-lg font-bold text-green-600">
                        {customer.successRate}%
                      </div>
                    </div>
                  </div>

                  {/* 底部信息 */}
                  <div className="flex items-center justify-between text-xs text-gray-500">
                    <div className="flex items-center gap-4">
                      <div className="flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        <span>首次订阅: {customer.firstSubscribeDate}</span>
                      </div>
                      <div className="flex items-center gap-1">
                        <Activity className="w-3 h-3" />
                        <span>最近活跃: {customer.lastActiveDate.split(' ')[0]}</span>
                      </div>
                    </div>
                    <button className="text-primary-600 hover:text-primary-700 font-medium">
                      查看详情 →
                    </button>
                  </div>
                </div>
              )
            })}
          </div>

          {/* 右侧详情 */}
          <div className="lg:col-span-1">
            {selectedCustomer ? (
              <div className="bg-white rounded-xl border border-gray-200 p-6 sticky top-28">
                <h3 className="text-lg font-bold text-gray-900 mb-6">客户详情</h3>

                <div className="space-y-6">
                  {/* 基本信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">基本信息</div>
                    <div className="space-y-3">
                      <div>
                        <div className="text-xs text-gray-600 mb-1">客户名称</div>
                        <div className="text-sm font-medium text-gray-900">{selectedCustomer.name}</div>
                      </div>
                      <div>
                        <div className="text-xs text-gray-600 mb-1">Subject ID</div>
                        <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                          {selectedCustomer.subjectId}
                        </code>
                      </div>
                      <div>
                        <div className="text-xs text-gray-600 mb-1">客户类型</div>
                        <div className="text-sm text-gray-900">
                          {selectedCustomer.type === 'ENTERPRISE' ? '企业客户' : '个人客户'}
                        </div>
                      </div>
                      <div>
                        <div className="text-xs text-gray-600 mb-1">所属行业</div>
                        <div className="text-sm text-gray-900">{selectedCustomer.industry}</div>
                      </div>
                    </div>
                  </div>

                  {/* 联系信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">联系信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex items-center gap-2">
                        <Users className="w-4 h-4 text-gray-400" />
                        <span className="text-gray-600">联系人:</span>
                        <span className="font-medium text-gray-900">{selectedCustomer.contactPerson}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Mail className="w-4 h-4 text-gray-400" />
                        <span className="text-gray-600">邮箱:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedCustomer.contactEmail}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Phone className="w-4 h-4 text-gray-400" />
                        <span className="text-gray-600">电话:</span>
                        <span className="font-medium text-gray-900">{selectedCustomer.contactPhone}</span>
                      </div>
                    </div>
                  </div>

                  {/* 订阅统计 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">订阅统计</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">总订阅数:</span>
                        <span className="font-medium text-gray-900">{selectedCustomer.subscriptionCount}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">活跃订阅:</span>
                        <span className="font-medium text-green-600">{selectedCustomer.activeSubscriptions}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">累计收入:</span>
                        <span className="font-medium text-gray-900">¥{selectedCustomer.totalRevenue.toLocaleString()}</span>
                      </div>
                    </div>
                  </div>

                  {/* 使用统计 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">使用统计</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">总调用次数:</span>
                        <span className="font-medium text-gray-900">{selectedCustomer.totalCalls.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">平均响应:</span>
                        <span className="font-medium text-gray-900">{selectedCustomer.avgResponseTime}ms</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">成功率:</span>
                        <span className="font-medium text-green-600">{selectedCustomer.successRate}%</span>
                      </div>
                    </div>
                  </div>

                  {/* 时间信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">时间信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">首次订阅:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedCustomer.firstSubscribeDate}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">最近活跃:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedCustomer.lastActiveDate}</span>
                      </div>
                    </div>
                  </div>

                  {/* 操作按钮 */}
                  <div className="space-y-2 pt-4 border-t border-gray-200">
                    <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                      <Mail className="w-4 h-4" />
                      <span>发送消息</span>
                    </button>
                    <button className="w-full flex items-center justify-center gap-2 px-4 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                      <Eye className="w-4 h-4" />
                      <span>查看订阅详情</span>
                    </button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="bg-white rounded-xl border border-gray-200 p-12 text-center sticky top-28">
                <Users className="w-12 h-12 mx-auto text-gray-300 mb-4" />
                <p className="text-gray-600">选择一个客户查看详情</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
