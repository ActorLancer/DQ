'use client'

import { useState } from 'react'
import Link from 'next/link'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Search, 
  Filter,
  DollarSign,
  FileText,
  Download,
  CheckCircle,
  Clock,
  XCircle,
  Calendar,
  CreditCard,
  Receipt,
  Eye
} from 'lucide-react'

interface Order {
  id: string
  orderId: string
  listingTitle: string
  supplierName: string
  plan: string
  orderType: 'SUBSCRIPTION' | 'ONE_TIME' | 'RENEWAL'
  amount: number
  currency: 'CNY'
  status: 'PENDING_PAYMENT' | 'PAID' | 'CANCELLED' | 'REFUNDED'
  paymentMethod?: string
  createdAt: string
  paidAt?: string
  invoiceStatus: 'NOT_REQUESTED' | 'REQUESTED' | 'ISSUED'
  invoiceNumber?: string
}

interface LinkedApiKey {
  id: string
  orderId: string
  name: string
  status: 'ACTIVE' | 'DISABLED'
  createdAt: string
}

const MOCK_ORDERS: Order[] = [
  {
    id: 'order_001',
    orderId: 'order_20260428_001',
    listingTitle: '企业工商风险数据',
    supplierName: '天眼数据科技',
    plan: '标准版',
    orderType: 'SUBSCRIPTION',
    amount: 9800,
    currency: 'CNY',
    status: 'PAID',
    paymentMethod: '企业对公转账',
    createdAt: '2026-03-28 10:00:00',
    paidAt: '2026-03-28 15:30:00',
    invoiceStatus: 'ISSUED',
    invoiceNumber: 'INV-2026-03-001',
  },
  {
    id: 'order_002',
    orderId: 'order_20260401_002',
    listingTitle: '消费者行为分析数据',
    supplierName: '智慧消费研究院',
    plan: '企业版',
    orderType: 'SUBSCRIPTION',
    amount: 58000,
    currency: 'CNY',
    status: 'PAID',
    paymentMethod: '企业对公转账',
    createdAt: '2026-01-01 09:00:00',
    paidAt: '2026-01-02 10:15:00',
    invoiceStatus: 'ISSUED',
    invoiceNumber: 'INV-2026-01-001',
  },
  {
    id: 'order_003',
    orderId: 'order_20260410_003',
    listingTitle: '物流轨迹实时数据',
    supplierName: '智运物流数据',
    plan: '按量计费',
    orderType: 'ONE_TIME',
    amount: 15600,
    currency: 'CNY',
    status: 'PAID',
    paymentMethod: '企业对公转账',
    createdAt: '2026-04-10 14:00:00',
    paidAt: '2026-04-10 16:20:00',
    invoiceStatus: 'REQUESTED',
  },
  {
    id: 'order_004',
    orderId: 'order_20260426_004',
    listingTitle: '供应链物流数据',
    supplierName: '智运物流数据',
    plan: '企业版',
    orderType: 'SUBSCRIPTION',
    amount: 28000,
    currency: 'CNY',
    status: 'PAID',
    paymentMethod: '企业对公转账',
    createdAt: '2026-04-26 11:00:00',
    paidAt: '2026-04-26 14:30:00',
    invoiceStatus: 'NOT_REQUESTED',
  },
  {
    id: 'order_005',
    orderId: 'order_20260428_005',
    listingTitle: '企业工商风险数据',
    supplierName: '天眼数据科技',
    plan: '标准版',
    orderType: 'RENEWAL',
    amount: 9800,
    currency: 'CNY',
    status: 'PENDING_PAYMENT',
    createdAt: '2026-04-28 09:00:00',
    invoiceStatus: 'NOT_REQUESTED',
  },
]

const MOCK_LINKED_API_KEYS: LinkedApiKey[] = [
  { id: 'key_001', orderId: 'order_20260428_001', name: '生产环境 - 企业风险数据', status: 'ACTIVE', createdAt: '2026-03-28 10:00:00' },
  { id: 'key_002', orderId: 'order_20260428_001', name: '测试环境 - 企业风险数据', status: 'ACTIVE', createdAt: '2026-03-28 10:05:00' },
  { id: 'key_003', orderId: 'order_20260401_002', name: '生产环境 - 消费行为数据', status: 'ACTIVE', createdAt: '2026-01-01 09:00:00' },
]

const ORDER_STATUS_CONFIG = {
  PENDING_PAYMENT: { label: '待支付', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  PAID: { label: '已支付', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  CANCELLED: { label: '已取消', color: 'bg-gray-100 text-gray-800', icon: XCircle },
  REFUNDED: { label: '已退款', color: 'bg-red-100 text-red-800', icon: XCircle },
}

const INVOICE_STATUS_CONFIG = {
  NOT_REQUESTED: { label: '未申请', color: 'bg-gray-100 text-gray-800' },
  REQUESTED: { label: '已申请', color: 'bg-blue-100 text-blue-800' },
  ISSUED: { label: '已开具', color: 'bg-green-100 text-green-800' },
}

const ORDER_TYPE_CONFIG = {
  SUBSCRIPTION: { label: '订阅', color: 'bg-blue-100 text-blue-800' },
  ONE_TIME: { label: '一次性', color: 'bg-purple-100 text-purple-800' },
  RENEWAL: { label: '续订', color: 'bg-green-100 text-green-800' },
}

export default function BuyerOrdersPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedOrder, setSelectedOrder] = useState<Order | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredOrders = MOCK_ORDERS.filter((order) => {
    const matchesKeyword = 
      order.listingTitle.toLowerCase().includes(searchKeyword.toLowerCase()) ||
      order.supplierName.toLowerCase().includes(searchKeyword.toLowerCase()) ||
      order.orderId.toLowerCase().includes(searchKeyword.toLowerCase())
    const matchesStatus = selectedStatus === 'all' || order.status === selectedStatus
    return matchesKeyword && matchesStatus
  })

  const totalSpent = MOCK_ORDERS
    .filter(o => o.status === 'PAID')
    .reduce((sum, o) => sum + o.amount, 0)

  const pendingPayment = MOCK_ORDERS
    .filter(o => o.status === 'PENDING_PAYMENT')
    .reduce((sum, o) => sum + o.amount, 0)

  const selectedOrderKeys = selectedOrder
    ? MOCK_LINKED_API_KEYS.filter((item) => item.orderId === selectedOrder.orderId)
    : []

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:orders:read"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">订单与账单</h1>
          <p className="text-gray-600">管理您的订单记录和发票</p>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center">
                <DollarSign className="w-6 h-6 text-green-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">¥{totalSpent.toLocaleString()}</div>
            <div className="text-sm text-gray-600">累计支出</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-yellow-50 rounded-lg flex items-center justify-center">
                <Clock className="w-6 h-6 text-yellow-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">¥{pendingPayment.toLocaleString()}</div>
            <div className="text-sm text-gray-600">待支付</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center">
                <Receipt className="w-6 h-6 text-blue-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">
              {MOCK_ORDERS.filter(o => o.invoiceStatus === 'ISSUED').length}
            </div>
            <div className="text-sm text-gray-600">已开发票</div>
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
                placeholder="搜索订单号、商品名称或供应商..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>

            <select
              value={selectedStatus}
              onChange={(e) => setSelectedStatus(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部状态</option>
              <option value="PENDING_PAYMENT">待支付</option>
              <option value="PAID">已支付</option>
              <option value="CANCELLED">已取消</option>
              <option value="REFUNDED">已退款</option>
            </select>

            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>更多筛选</span>
            </button>
          </div>
        </div>

        {/* 订单列表 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 左侧列表 */}
          <div className="lg:col-span-2 space-y-4">
            {filteredOrders.map((order) => {
              const statusConfig = ORDER_STATUS_CONFIG[order.status]
              const StatusIcon = statusConfig.icon
              const isSelected = selectedOrder?.id === order.id

              return (
                <div
                  key={order.id}
                  onClick={() => setSelectedOrder(order)}
                  className={`bg-white rounded-xl border-2 p-6 cursor-pointer transition-all ${
                    isSelected
                      ? 'border-primary-500 shadow-lg'
                      : 'border-gray-200 hover:border-primary-300 hover:shadow-md'
                  }`}
                >
                  {/* 头部 */}
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <h3 className="text-lg font-bold text-gray-900">{order.listingTitle}</h3>
                        <span className={`status-tag text-xs ${ORDER_TYPE_CONFIG[order.orderType].color}`}>
                          {ORDER_TYPE_CONFIG[order.orderType].label}
                        </span>
                      </div>
                      <div className="flex items-center gap-3 mb-2">
                        <span className="text-sm text-gray-600">{order.supplierName}</span>
                        <span className="text-sm text-gray-400">·</span>
                        <span className="text-sm font-medium text-gray-900">{order.plan}</span>
                      </div>
                    </div>
                    <span className={`status-tag ${statusConfig.color}`}>
                      <StatusIcon className="w-3.5 h-3.5" />
                      <span>{statusConfig.label}</span>
                    </span>
                  </div>

                  {/* 金额和订单号 */}
                  <div className="grid grid-cols-2 gap-4 mb-4 pb-4 border-b border-gray-100">
                    <div>
                      <div className="text-xs text-gray-500 mb-1">订单金额</div>
                      <div className="text-xl font-bold text-gray-900">¥{order.amount.toLocaleString()}</div>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">订单号</div>
                      <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block truncate">
                        {order.orderId}
                      </code>
                    </div>
                  </div>

                  {/* 底部信息 */}
                  <div className="flex items-center justify-between text-xs">
                    <div className="flex items-center gap-4 text-gray-500">
                      <div className="flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        <span>创建: {order.createdAt.split(' ')[0]}</span>
                      </div>
                      {order.paidAt && (
                        <div className="flex items-center gap-1">
                          <CheckCircle className="w-3 h-3" />
                          <span>支付: {order.paidAt.split(' ')[0]}</span>
                        </div>
                      )}
                    </div>
                    <span className={`status-tag text-xs ${INVOICE_STATUS_CONFIG[order.invoiceStatus].color}`}>
                      发票: {INVOICE_STATUS_CONFIG[order.invoiceStatus].label}
                    </span>
                  </div>

                  {/* 待支付提示 */}
                  {order.status === 'PENDING_PAYMENT' && (
                    <div className="mt-4 flex gap-2">
                      <button className="flex-1 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium">
                        立即支付
                      </button>
                      <button className="px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium">
                        取消订单
                      </button>
                    </div>
                  )}
                </div>
              )
            })}
          </div>

          {/* 右侧详情 */}
          <div className="lg:col-span-1">
            {selectedOrder ? (
              <div className="bg-white rounded-xl border border-gray-200 p-6 sticky top-28">
                <h3 className="text-lg font-bold text-gray-900 mb-6">订单详情</h3>

                <div className="space-y-6">
                  {/* Order ID */}
                  <div>
                    <div className="text-xs text-gray-500 mb-1">Order ID</div>
                    <code className="font-mono text-xs text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                      {selectedOrder.orderId}
                    </code>
                  </div>

                  {/* 商品信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">商品信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">商品:</span>
                        <span className="font-medium text-gray-900">{selectedOrder.listingTitle}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">供应商:</span>
                        <span className="font-medium text-gray-900">{selectedOrder.supplierName}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">套餐:</span>
                        <span className="font-medium text-gray-900">{selectedOrder.plan}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">订单类型:</span>
                        <span className={`status-tag text-xs ${ORDER_TYPE_CONFIG[selectedOrder.orderType].color}`}>
                          {ORDER_TYPE_CONFIG[selectedOrder.orderType].label}
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* 金额信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">金额信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">订单金额:</span>
                        <span className="font-bold text-gray-900">¥{selectedOrder.amount.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">币种:</span>
                        <span className="font-medium text-gray-900">{selectedOrder.currency}</span>
                      </div>
                      {selectedOrder.paymentMethod && (
                        <div className="flex justify-between">
                          <span className="text-gray-600">支付方式:</span>
                          <span className="font-medium text-gray-900">{selectedOrder.paymentMethod}</span>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* 状态信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">状态信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">订单状态:</span>
                        <span className={`status-tag text-xs ${ORDER_STATUS_CONFIG[selectedOrder.status].color}`}>
                          {ORDER_STATUS_CONFIG[selectedOrder.status].label}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">发票状态:</span>
                        <span className={`status-tag text-xs ${INVOICE_STATUS_CONFIG[selectedOrder.invoiceStatus].color}`}>
                          {INVOICE_STATUS_CONFIG[selectedOrder.invoiceStatus].label}
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* 时间信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">时间信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">创建时间:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedOrder.createdAt}</span>
                      </div>
                      {selectedOrder.paidAt && (
                        <div className="flex justify-between">
                          <span className="text-gray-600">支付时间:</span>
                          <span className="font-medium text-gray-900 text-xs">{selectedOrder.paidAt}</span>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* 发票信息 */}
                  {selectedOrder.invoiceNumber && (
                    <div>
                      <div className="text-xs text-gray-500 mb-2">发票信息</div>
                      <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
                        <div className="text-xs text-green-800 mb-1">发票号</div>
                        <code className="text-sm font-mono text-green-900 font-medium">
                          {selectedOrder.invoiceNumber}
                        </code>
                      </div>
                    </div>
                  )}

                  {selectedOrder.status === 'PAID' && (
                    <div>
                      <div className="text-xs text-gray-500 mb-2">关联 API Key</div>
                      <div className="p-3 bg-blue-50 border border-blue-200 rounded-lg">
                        <div className="text-xs text-blue-800 mb-2">已绑定 {selectedOrderKeys.length} 个密钥</div>
                        {selectedOrderKeys.length > 0 ? (
                          <div className="space-y-1.5 mb-3">
                            {selectedOrderKeys.map((key) => (
                              <div key={key.id} className="text-xs text-blue-900 flex items-center justify-between">
                                <span>{key.name}</span>
                                <span>{key.status === 'ACTIVE' ? '活跃' : '禁用'}</span>
                              </div>
                            ))}
                          </div>
                        ) : (
                          <div className="text-xs text-blue-900 mb-3">当前订单尚未创建 API Key</div>
                        )}
                        <Link
                          href={`/console/buyer/api-keys?orderId=${selectedOrder.orderId}`}
                          className="text-xs font-medium text-primary-700 hover:text-primary-800"
                        >
                          前往 API 密钥管理 →
                        </Link>
                      </div>
                    </div>
                  )}

                  {/* 操作按钮 */}
                  <div className="space-y-2 pt-4 border-t border-gray-200">
                    {selectedOrder.status === 'PENDING_PAYMENT' && (
                      <>
                        <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                          <CreditCard className="w-4 h-4" />
                          <span>立即支付</span>
                        </button>
                        <button className="w-full flex items-center justify-center gap-2 px-4 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                          <XCircle className="w-4 h-4" />
                          <span>取消订单</span>
                        </button>
                      </>
                    )}

                    {selectedOrder.status === 'PAID' && selectedOrder.invoiceStatus === 'NOT_REQUESTED' && (
                      <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                        <FileText className="w-4 h-4" />
                        <span>申请发票</span>
                      </button>
                    )}

                    {selectedOrder.invoiceStatus === 'ISSUED' && (
                      <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                        <Download className="w-4 h-4" />
                        <span>下载发票</span>
                      </button>
                    )}
                  </div>
                </div>
              </div>
            ) : (
              <div className="bg-white rounded-xl border border-gray-200 p-12 text-center sticky top-28">
                <Eye className="w-12 h-12 mx-auto text-gray-300 mb-4" />
                <p className="text-gray-600">选择一个订单查看详情</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
