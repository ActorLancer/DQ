export interface PaidOrderRef {
  orderId: string
  listingTitle: string
  plan: string
  amount: number
  paidAt: string
}

export interface ApiKey {
  id: string
  name: string
  keyPrefix: string
  fullKey?: string
  orderId: string
  listingTitle: string
  permissions: string[]
  status: 'ACTIVE' | 'DISABLED' | 'EXPIRED'
  createdAt: string
  expiresAt: string | null
  lastUsedAt: string | null
  totalCalls: number
  ipWhitelist: string[]
}

export const PAID_ORDERS: PaidOrderRef[] = [
  {
    orderId: 'order_20260428_001',
    listingTitle: '企业工商风险数据',
    plan: '标准版',
    amount: 9800,
    paidAt: '2026-03-28 15:30:00',
  },
  {
    orderId: 'order_20260401_002',
    listingTitle: '消费者行为分析数据',
    plan: '企业版',
    amount: 58000,
    paidAt: '2026-01-02 10:15:00',
  },
  {
    orderId: 'order_20260410_003',
    listingTitle: '物流轨迹实时数据',
    plan: '按量计费',
    amount: 15600,
    paidAt: '2026-04-10 16:20:00',
  },
]

export const MOCK_API_KEYS: ApiKey[] = [
  {
    id: 'key_001',
    name: '生产环境 - 企业风险数据',
    keyPrefix: 'sk_live_',
    orderId: 'order_20260428_001',
    listingTitle: '企业工商风险数据',
    permissions: ['read'],
    status: 'ACTIVE',
    createdAt: '2026-03-28 10:00:00',
    expiresAt: '2026-05-28 23:59:59',
    lastUsedAt: '2026-04-28 15:30:00',
    totalCalls: 6580,
    ipWhitelist: ['192.168.1.100', '10.0.0.50'],
  },
  {
    id: 'key_002',
    name: '测试环境 - 企业风险数据',
    keyPrefix: 'sk_test_',
    orderId: 'order_20260428_001',
    listingTitle: '企业工商风险数据',
    permissions: ['read'],
    status: 'ACTIVE',
    createdAt: '2026-03-28 10:05:00',
    expiresAt: '2026-05-28 23:59:59',
    lastUsedAt: '2026-04-27 18:20:00',
    totalCalls: 1250,
    ipWhitelist: [],
  },
  {
    id: 'key_003',
    name: '生产环境 - 消费行为数据',
    keyPrefix: 'sk_live_',
    orderId: 'order_20260401_002',
    listingTitle: '消费者行为分析数据',
    permissions: ['read'],
    status: 'ACTIVE',
    createdAt: '2026-01-01 09:00:00',
    expiresAt: null,
    lastUsedAt: '2026-04-28 16:15:00',
    totalCalls: 12350,
    ipWhitelist: ['192.168.1.100'],
  },
]
