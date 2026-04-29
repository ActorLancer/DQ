export interface Order {
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

export const MOCK_ORDERS: Order[] = [
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

export const ORDER_STATUS_CONFIG = {
  PENDING_PAYMENT: { label: '待支付', color: 'bg-yellow-100 text-yellow-800' },
  PAID: { label: '已支付', color: 'bg-green-100 text-green-800' },
  CANCELLED: { label: '已取消', color: 'bg-gray-100 text-gray-800' },
  REFUNDED: { label: '已退款', color: 'bg-red-100 text-red-800' },
}

export const INVOICE_STATUS_CONFIG = {
  NOT_REQUESTED: { label: '未申请', color: 'bg-gray-100 text-gray-800' },
  REQUESTED: { label: '已申请', color: 'bg-blue-100 text-blue-800' },
  ISSUED: { label: '已开具', color: 'bg-green-100 text-green-800' },
}

export const ORDER_TYPE_CONFIG = {
  SUBSCRIPTION: { label: '订阅', color: 'bg-blue-100 text-blue-800' },
  ONE_TIME: { label: '一次性', color: 'bg-purple-100 text-purple-800' },
  RENEWAL: { label: '续订', color: 'bg-green-100 text-green-800' },
}
