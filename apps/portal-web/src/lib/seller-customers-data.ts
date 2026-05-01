export interface SellerCustomer {
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

export const SELLER_CUSTOMERS: SellerCustomer[] = [
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
