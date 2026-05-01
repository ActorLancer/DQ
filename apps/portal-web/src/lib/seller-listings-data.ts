import type { ListingStatus } from '@/types'

export interface SellerListingItem {
  id: string
  title: string
  industry: string
  status: ListingStatus
  pricingModel: string
  deliveryMethods: string[]
  qualityScore: number
  requestCount: number
  subscriberCount: number
  updatedAt: string
  chainStatus: string
}

export const SELLER_LISTINGS: SellerListingItem[] = [
  { id: 'listing_001', title: '企业工商风险数据', industry: '金融', status: 'LISTED', pricingModel: '月订阅', deliveryMethods: ['API', 'FILE'], qualityScore: 9.2, requestCount: 45, subscriberCount: 28, updatedAt: '2026-04-20', chainStatus: 'CONFIRMED' },
  { id: 'listing_002', title: '消费者行为分析数据', industry: '消费', status: 'LISTED', pricingModel: '年订阅', deliveryMethods: ['API'], qualityScore: 8.8, requestCount: 32, subscriberCount: 18, updatedAt: '2026-04-18', chainStatus: 'CONFIRMED' },
  { id: 'listing_003', title: '物流轨迹实时数据', industry: '交通', status: 'PENDING_REVIEW', pricingModel: '按量计费', deliveryMethods: ['API'], qualityScore: 9.5, requestCount: 0, subscriberCount: 0, updatedAt: '2026-04-25', chainStatus: 'NOT_SUBMITTED' },
  { id: 'listing_004', title: '医疗健康知识图谱', industry: '医疗', status: 'DRAFT', pricingModel: '定制', deliveryMethods: ['API', 'FILE'], qualityScore: 0, requestCount: 0, subscriberCount: 0, updatedAt: '2026-04-22', chainStatus: 'NOT_SUBMITTED' },
]
