export interface SellerContractInvoiceItem {
  id: string
  type: 'CONTRACT' | 'INVOICE'
  docNo: string
  customerName: string
  listingTitle: string
  amount: number
  status: 'PENDING' | 'ISSUED' | 'SIGNED' | 'FAILED'
  createdAt: string
  relatedOrderId: string
}

export const SELLER_CONTRACT_DOCS: SellerContractInvoiceItem[] = [
  { id: 'doc_001', type: 'CONTRACT', docNo: 'CTR-2026-0001', customerName: '某某金融科技', listingTitle: '企业工商风险数据', amount: 9800, status: 'SIGNED', createdAt: '2026-03-28', relatedOrderId: 'order_20260428_001' },
  { id: 'doc_002', type: 'INVOICE', docNo: 'INV-2026-0118', customerName: '某某金融科技', listingTitle: '企业工商风险数据', amount: 9800, status: 'ISSUED', createdAt: '2026-03-30', relatedOrderId: 'order_20260428_001' },
  { id: 'doc_003', type: 'CONTRACT', docNo: 'CTR-2026-0002', customerName: '智慧物流数据中心', listingTitle: '物流轨迹实时数据', amount: 15600, status: 'PENDING', createdAt: '2026-04-10', relatedOrderId: 'order_20260410_003' },
]
