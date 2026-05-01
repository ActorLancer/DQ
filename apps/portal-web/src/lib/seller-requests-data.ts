export type SellerRequestStatus = 'PENDING_SUPPLIER_REVIEW' | 'NEED_MORE_INFO' | 'APPROVED' | 'REJECTED'

export interface SellerAccessRequest {
  id: string
  requestId: string
  buyerName: string
  buyerSubject: string
  listingTitle: string
  planName: string
  usagePurpose: string
  expectedUsage: string
  status: SellerRequestStatus
  workflowStatus: string
  chainStatus: string
  projectionStatus: string
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
  createdAt: string
  updatedAt: string
}

export const SELLER_REQUESTS: SellerAccessRequest[] = [
  {
    id: 'ar_001',
    requestId: 'req_20260428_000001',
    buyerName: '某某科技有限公司',
    buyerSubject: '买方主体',
    listingTitle: '企业工商风险数据',
    planName: '标准版',
    usagePurpose: '用于供应链合作伙伴风险评估，预计每月查询 5000 次',
    expectedUsage: '5000/月',
    status: 'PENDING_SUPPLIER_REVIEW',
    workflowStatus: '待供应商审核',
    chainStatus: 'NOT_SUBMITTED',
    projectionStatus: 'PROJECTED',
    riskLevel: 'LOW',
    createdAt: '2026-04-28 14:30:00',
    updatedAt: '2026-04-28 14:30:00',
  },
  {
    id: 'ar_002',
    requestId: 'req_20260428_000002',
    buyerName: '智慧消费研究院',
    buyerSubject: '买方主体',
    listingTitle: '企业工商风险数据',
    planName: '企业版',
    usagePurpose: '用于信贷审批与风险控制，预计每月查询 15000 次',
    expectedUsage: '15000/月',
    status: 'PENDING_SUPPLIER_REVIEW',
    workflowStatus: '待供应商审核',
    chainStatus: 'NOT_SUBMITTED',
    projectionStatus: 'PROJECTED',
    riskLevel: 'MEDIUM',
    createdAt: '2026-04-28 10:15:00',
    updatedAt: '2026-04-28 10:15:00',
  },
  {
    id: 'ar_003',
    requestId: 'req_20260427_000003',
    buyerName: '金融数据服务',
    buyerSubject: '买方主体',
    listingTitle: '企业工商风险数据',
    planName: '企业版',
    usagePurpose: '用于投资决策支持，预计每月查询 20000 次',
    expectedUsage: '20000/月',
    status: 'NEED_MORE_INFO',
    workflowStatus: '需补充材料',
    chainStatus: 'NOT_SUBMITTED',
    projectionStatus: 'PROJECTED',
    riskLevel: 'HIGH',
    createdAt: '2026-04-27 16:45:00',
    updatedAt: '2026-04-28 09:20:00',
  },
]
