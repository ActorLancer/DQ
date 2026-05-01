export interface SellerRevenueRecord {
  id: string
  date: string
  customer: string
  listing: string
  plan: string
  amount: number
  type: 'SUBSCRIPTION' | 'RENEWAL' | 'UPGRADE' | 'ONE_TIME'
  status: 'PAID' | 'PENDING' | 'REFUNDED'
}

export const SELLER_REVENUE_RECORDS: SellerRevenueRecord[] = [
  { id: 'rev_001', date: '2026-04-29', customer: '某某金融科技', listing: '企业工商风险数据', plan: '企业版', amount: 9999, type: 'SUBSCRIPTION', status: 'PAID' },
  { id: 'rev_002', date: '2026-04-28', customer: '智慧物流数据中心', listing: '物流轨迹实时数据', plan: '标准版', amount: 999, type: 'RENEWAL', status: 'PAID' },
  { id: 'rev_003', date: '2026-04-27', customer: '某某数据分析公司', listing: '企业工商风险数据', plan: '标准版', amount: 999, type: 'SUBSCRIPTION', status: 'PAID' },
  { id: 'rev_004', date: '2026-04-26', customer: '某某金融科技', listing: '金融市场行情数据', plan: '企业版', amount: 19999, type: 'UPGRADE', status: 'PAID' },
  { id: 'rev_005', date: '2026-04-25', customer: '智慧物流数据中心', listing: '企业工商风险数据', plan: '标准版', amount: 999, type: 'SUBSCRIPTION', status: 'PAID' },
]
