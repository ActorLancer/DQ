export interface Subscription {
  id: string
  subscriptionId: string
  listingTitle: string
  supplierName: string
  plan: string
  status: 'ACTIVE' | 'EXPIRED' | 'SUSPENDED' | 'REVOKED'
  quota: number | null
  usedQuota: number
  startsAt: string
  endsAt: string | null
  apiCallsToday: number
  apiCallsTotal: number
  lastCallAt: string
  chainProofId: string
}

export const MOCK_SUBSCRIPTIONS: Subscription[] = [
  {
    id: 'sub_001',
    subscriptionId: 'subscription_20260328_001',
    listingTitle: '企业工商风险数据',
    supplierName: '天眼数据科技',
    plan: '标准版',
    status: 'ACTIVE',
    quota: 10000,
    usedQuota: 6580,
    startsAt: '2026-03-28',
    endsAt: '2026-05-28',
    apiCallsToday: 245,
    apiCallsTotal: 6580,
    lastCallAt: '2026-04-28 15:30:00',
    chainProofId: 'proof_sub_001',
  },
  {
    id: 'sub_002',
    subscriptionId: 'subscription_20260101_002',
    listingTitle: '消费者行为分析数据',
    supplierName: '智慧消费研究院',
    plan: '企业版',
    status: 'ACTIVE',
    quota: 50000,
    usedQuota: 12350,
    startsAt: '2026-01-01',
    endsAt: '2026-12-31',
    apiCallsToday: 892,
    apiCallsTotal: 12350,
    lastCallAt: '2026-04-28 16:15:00',
    chainProofId: 'proof_sub_002',
  },
  {
    id: 'sub_003',
    subscriptionId: 'subscription_20260410_003',
    listingTitle: '物流轨迹实时数据',
    supplierName: '智运物流数据',
    plan: '按量计费',
    status: 'ACTIVE',
    quota: null,
    usedQuota: 8920,
    startsAt: '2026-04-10',
    endsAt: null,
    apiCallsToday: 156,
    apiCallsTotal: 8920,
    lastCallAt: '2026-04-28 14:45:00',
    chainProofId: 'proof_sub_003',
  },
]

export const SUB_STATUS_CONFIG = {
  ACTIVE: { label: '活跃', color: 'bg-green-100 text-green-800' },
  EXPIRED: { label: '已过期', color: 'bg-gray-100 text-gray-800' },
  SUSPENDED: { label: '已暂停', color: 'bg-orange-100 text-orange-800' },
  REVOKED: { label: '已撤销', color: 'bg-red-100 text-red-800' },
}
