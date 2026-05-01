export interface SellerApiCall {
  id: string
  timestamp: string
  customer: string
  listing: string
  endpoint: string
  method: string
  statusCode: number
  responseTime: number
  success: boolean
}

export const SELLER_API_CALLS: SellerApiCall[] = [
  { id: 'call_001', timestamp: '2026-04-29 15:45:23', customer: '某某金融科技', listing: '企业工商风险数据', endpoint: '/api/v1/company/risk', method: 'GET', statusCode: 200, responseTime: 95, success: true },
  { id: 'call_002', timestamp: '2026-04-29 15:44:18', customer: '智慧物流数据中心', listing: '物流轨迹实时数据', endpoint: '/api/v1/logistics/track', method: 'POST', statusCode: 200, responseTime: 120, success: true },
  { id: 'call_003', timestamp: '2026-04-29 15:43:05', customer: '某某数据分析公司', listing: '企业工商风险数据', endpoint: '/api/v1/company/info', method: 'GET', statusCode: 500, responseTime: 3500, success: false },
  { id: 'call_004', timestamp: '2026-04-29 15:42:30', customer: '某某金融科技', listing: '金融市场行情数据', endpoint: '/api/v1/market/quote', method: 'GET', statusCode: 200, responseTime: 85, success: true },
  { id: 'call_005', timestamp: '2026-04-29 15:41:15', customer: '智慧物流数据中心', listing: '企业工商风险数据', endpoint: '/api/v1/company/risk', method: 'GET', statusCode: 429, responseTime: 50, success: false },
]
