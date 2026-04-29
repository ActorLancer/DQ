'use client'

import { useMemo, useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { FileText, Receipt, Download, Search, Filter, CheckCircle, Clock, AlertCircle } from 'lucide-react'

interface ContractInvoiceItem {
  id: string
  type: 'CONTRACT' | 'INVOICE'
  docNo: string
  customerName: string
  listingTitle: string
  amount: number
  currency: 'CNY'
  status: 'PENDING' | 'ISSUED' | 'SIGNED' | 'FAILED'
  createdAt: string
  relatedOrderId: string
}

const MOCK_DOCS: ContractInvoiceItem[] = [
  {
    id: 'doc_001',
    type: 'CONTRACT',
    docNo: 'CTR-2026-0001',
    customerName: '某某金融科技',
    listingTitle: '企业工商风险数据',
    amount: 9800,
    currency: 'CNY',
    status: 'SIGNED',
    createdAt: '2026-03-28',
    relatedOrderId: 'order_20260428_001',
  },
  {
    id: 'doc_002',
    type: 'INVOICE',
    docNo: 'INV-2026-0118',
    customerName: '某某金融科技',
    listingTitle: '企业工商风险数据',
    amount: 9800,
    currency: 'CNY',
    status: 'ISSUED',
    createdAt: '2026-03-30',
    relatedOrderId: 'order_20260428_001',
  },
  {
    id: 'doc_003',
    type: 'CONTRACT',
    docNo: 'CTR-2026-0002',
    customerName: '智慧物流数据中心',
    listingTitle: '物流轨迹实时数据',
    amount: 15600,
    currency: 'CNY',
    status: 'PENDING',
    createdAt: '2026-04-10',
    relatedOrderId: 'order_20260410_003',
  },
]

const STATUS_CONFIG = {
  PENDING: { label: '处理中', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  ISSUED: { label: '已开具', color: 'bg-blue-100 text-blue-800', icon: CheckCircle },
  SIGNED: { label: '已签署', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  FAILED: { label: '失败', color: 'bg-red-100 text-red-800', icon: AlertCircle },
}

export default function SellerContractsPage() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()
  const [keyword, setKeyword] = useState('')
  const [docType, setDocType] = useState<'all' | 'CONTRACT' | 'INVOICE'>('all')

  const filtered = useMemo(() => {
    return MOCK_DOCS.filter((item) => {
      const matchKeyword =
        item.docNo.toLowerCase().includes(keyword.toLowerCase()) ||
        item.customerName.toLowerCase().includes(keyword.toLowerCase()) ||
        item.relatedOrderId.toLowerCase().includes(keyword.toLowerCase())
      const matchType = docType === 'all' || item.type === docType
      return matchKeyword && matchType
    })
  }, [keyword, docType])

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:contracts:read"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">合同发票</h1>
            <p className="text-gray-600">管理合同与发票文档，关联订单与结算凭证</p>
          </div>
          <button className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700">
            <FileText className="w-4 h-4" />
            <span>新建文档任务</span>
          </button>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="text-2xl font-bold">{MOCK_DOCS.filter(d => d.type === 'CONTRACT').length}</div><div className="text-sm text-gray-600">合同总数</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="text-2xl font-bold">{MOCK_DOCS.filter(d => d.type === 'INVOICE').length}</div><div className="text-sm text-gray-600">发票总数</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="text-2xl font-bold">{MOCK_DOCS.filter(d => d.status === 'PENDING').length}</div><div className="text-sm text-gray-600">处理中</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="text-2xl font-bold">¥{MOCK_DOCS.reduce((s, d) => s + d.amount, 0).toLocaleString()}</div><div className="text-sm text-gray-600">关联金额</div></div>
        </div>

        <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
          <div className="flex items-center gap-4">
            <div className="relative flex-1">
              <Search className="w-4 h-4 text-gray-400 absolute left-3 top-1/2 -translate-y-1/2" />
              <input value={keyword} onChange={(e) => setKeyword(e.target.value)} className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg" placeholder="搜索文档号 / 客户 / 订单号" />
            </div>
            <select value={docType} onChange={(e) => setDocType(e.target.value as any)} className="px-4 py-2 border border-gray-300 rounded-lg">
              <option value="all">全部类型</option>
              <option value="CONTRACT">合同</option>
              <option value="INVOICE">发票</option>
            </select>
            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"><Filter className="w-4 h-4" /><span>更多筛选</span></button>
          </div>
        </div>

        <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">文档</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">客户/商品</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">关联订单</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">金额</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {filtered.map((item) => {
                const status = STATUS_CONFIG[item.status]
                const Icon = status.icon
                return (
                  <tr key={item.id} className="hover:bg-gray-50">
                    <td className="py-4 px-6">
                      <div className="font-medium text-gray-900">{item.docNo}</div>
                      <div className="text-xs text-gray-500 mt-1">{item.type === 'CONTRACT' ? '合同' : '发票'} · {item.createdAt}</div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">{item.customerName}</div>
                      <div className="text-xs text-gray-500 mt-1">{item.listingTitle}</div>
                    </td>
                    <td className="py-4 px-6"><code className="text-xs bg-gray-100 px-2 py-1 rounded">{item.relatedOrderId}</code></td>
                    <td className="py-4 px-6 text-right font-medium">¥{item.amount.toLocaleString()}</td>
                    <td className="py-4 px-6 text-center">
                      <span className={`status-tag ${status.color}`}><Icon className="w-3.5 h-3.5" /><span>{status.label}</span></span>
                    </td>
                    <td className="py-4 px-6">
                      <div className="flex items-center justify-end gap-2">
                        <button className="p-2 hover:bg-gray-100 rounded-lg" title="下载"><Download className="w-4 h-4 text-gray-600" /></button>
                        <button className="p-2 hover:bg-gray-100 rounded-lg" title="开票"><Receipt className="w-4 h-4 text-gray-600" /></button>
                      </div>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>
    </>
  )
}
