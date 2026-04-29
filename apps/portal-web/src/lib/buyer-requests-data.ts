export interface AccessRequest {
  id: string
  requestId: string
  listingTitle: string
  supplierName: string
  plan: string
  status: 'DRAFT' | 'SUBMITTED' | 'PENDING_SUPPLIER_REVIEW' | 'PENDING_PLATFORM_REVIEW' | 'NEED_MORE_INFO' | 'APPROVED' | 'REJECTED' | 'CANCELLED'
  usagePurpose: string
  expectedUsage: string
  createdAt: string
  updatedAt: string
  reviewNotes?: string
  chainStatus: 'NOT_SUBMITTED' | 'SUBMITTED' | 'CONFIRMED'
  projectionStatus: 'PENDING' | 'PROJECTED'
}

export const MOCK_REQUESTS: AccessRequest[] = [
  {
    id: 'req_001',
    requestId: 'request_20260428_001',
    listingTitle: '金融市场行情数据',
    supplierName: '金融数据服务',
    plan: '专业版',
    status: 'PENDING_SUPPLIER_REVIEW',
    usagePurpose: '用于量化交易策略回测和实时行情分析',
    expectedUsage: '预计每日调用 5000 次',
    createdAt: '2026-04-28 10:30:00',
    updatedAt: '2026-04-28 10:30:00',
    chainStatus: 'SUBMITTED',
    projectionStatus: 'PENDING',
  },
  {
    id: 'req_002',
    requestId: 'request_20260427_002',
    listingTitle: '医疗健康知识图谱',
    supplierName: '医疗大数据中心',
    plan: '定制版',
    status: 'NEED_MORE_INFO',
    usagePurpose: '用于智能诊断辅助系统开发',
    expectedUsage: '预计每月调用 20000 次',
    createdAt: '2026-04-27 14:20:00',
    updatedAt: '2026-04-28 09:15:00',
    reviewNotes: '请补充：数据使用场景详细说明、数据安全保障措施、相关医疗资质证明',
    chainStatus: 'SUBMITTED',
    projectionStatus: 'PROJECTED',
  },
  {
    id: 'req_003',
    requestId: 'request_20260426_003',
    listingTitle: '供应链物流数据',
    supplierName: '智运物流数据',
    plan: '企业版',
    status: 'APPROVED',
    usagePurpose: '用于供应链优化和物流路径规划',
    expectedUsage: '预计每日调用 3000 次',
    createdAt: '2026-04-26 16:00:00',
    updatedAt: '2026-04-27 10:30:00',
    reviewNotes: '申请已通过，请前往“我的订阅”查看 API Key',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
  },
]

export const REQ_STATUS_CONFIG = {
  DRAFT: { label: '草稿', color: 'bg-gray-100 text-gray-800' },
  SUBMITTED: { label: '已提交', color: 'bg-blue-100 text-blue-800' },
  PENDING_SUPPLIER_REVIEW: { label: '待供应商审核', color: 'bg-yellow-100 text-yellow-800' },
  PENDING_PLATFORM_REVIEW: { label: '待平台审核', color: 'bg-purple-100 text-purple-800' },
  NEED_MORE_INFO: { label: '需补充材料', color: 'bg-orange-100 text-orange-800' },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800' },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800' },
  CANCELLED: { label: '已取消', color: 'bg-gray-100 text-gray-800' },
}
