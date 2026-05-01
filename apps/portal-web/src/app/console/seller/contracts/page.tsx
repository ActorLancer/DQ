'use client'

import { useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import { FileText, Receipt, Download, CheckCircle, Clock, AlertCircle, Filter, Layers, ArrowUpDown } from 'lucide-react'
import { SELLER_CONTRACT_DOCS, type SellerContractInvoiceItem } from '@/lib/seller-contracts-data'

const STATUS_CONFIG = {
  PENDING: { label: '处理中', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  ISSUED: { label: '已开具', color: 'bg-blue-100 text-blue-800', icon: CheckCircle },
  SIGNED: { label: '已签署', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  FAILED: { label: '失败', color: 'bg-red-100 text-red-800', icon: AlertCircle },
}

type SortBy = 'created_desc' | 'amount_desc'

export default function SellerContractsPage() {
  const router = useRouter()
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()
  const [keyword, setKeyword] = useState('')
  const [docType, setDocType] = useState<'all' | 'CONTRACT' | 'INVOICE'>('all')
  const [status, setStatus] = useState<'all' | SellerContractInvoiceItem['status']>('all')
  const [sortBy, setSortBy] = useState<SortBy>('created_desc')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)

  const filtered = useMemo(() => {
    const list = SELLER_CONTRACT_DOCS.filter((item) => {
      const kw = keyword.toLowerCase()
      const matchKeyword = item.docNo.toLowerCase().includes(kw) || item.customerName.toLowerCase().includes(kw) || item.relatedOrderId.toLowerCase().includes(kw)
      const matchType = docType === 'all' || item.type === docType
      const matchStatus = status === 'all' || item.status === status
      return matchKeyword && matchType && matchStatus
    })
    return [...list].sort((a, b) => sortBy === 'amount_desc' ? b.amount - a.amount : new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
  }, [keyword, docType, status, sortBy])

  const paged = useMemo(() => filtered.slice((page - 1) * pageSize, (page - 1) * pageSize + pageSize), [filtered, page, pageSize])

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:contracts:read" sessionExpiresAt={sessionExpiresAt} />
      <ConsoleListPageShell
        title="合同发票"
        subtitle="管理合同与发票文档，关联订单与结算凭证"
        headerAction={<button className="flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"><FileText className="w-4 h-4" /><span>新建文档任务</span></button>}
        toolbar={<QueryToolbar
          searchValue={keyword}
          onSearchChange={setKeyword}
          searchPlaceholder="搜索文档号 / 客户 / 订单号"
          onReset={() => { setKeyword(''); setDocType('all'); setStatus('all'); setSortBy('created_desc'); setPage(1); setPageSize(10) }}
          controls={<div className="grid grid-cols-1 md:grid-cols-5 gap-3"><select value={docType} onChange={(e) => setDocType(e.target.value as any)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部类型</option><option value="CONTRACT">合同</option><option value="INVOICE">发票</option></select><select value={status} onChange={(e) => setStatus(e.target.value as any)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="PENDING">处理中</option><option value="ISSUED">已开具</option><option value="SIGNED">已签署</option><option value="FAILED">失败</option></select><select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select><select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="created_desc">创建时间优先</option><option value="amount_desc">金额优先</option></select><div className="h-10 px-3 rounded-lg border border-gray-200 bg-gray-50 text-xs text-gray-600 flex items-center">文档总额 ¥{filtered.reduce((s, d) => s + d.amount, 0).toLocaleString()}</div></div>}
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filtered.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分页 {pageSize}/页</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'created_desc' ? '创建时间' : '金额'}</span></>}
        />}
        content={<div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200"><tr><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">文档</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">客户/商品</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">关联订单</th><th className="text-right py-4 px-6 text-sm font-medium text-gray-700">金额</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th><th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th></tr></thead>
            <tbody className="divide-y divide-gray-200">
              {paged.map((item) => {
                const cfg = STATUS_CONFIG[item.status]
                const Icon = cfg.icon
                return (
                  <tr key={item.id} className="hover:bg-gray-50 cursor-pointer" onDoubleClick={() => router.push(`/console/seller/contracts/${item.id}`)}><td className="py-4 px-6"><div className="font-medium text-gray-900">{item.docNo}</div><div className="text-xs text-gray-500 mt-1">{item.type === 'CONTRACT' ? '合同' : '发票'} · {item.createdAt}</div></td><td className="py-4 px-6"><div className="text-sm text-gray-900">{item.customerName}</div><div className="text-xs text-gray-500 mt-1">{item.listingTitle}</div></td><td className="py-4 px-6"><code className="text-xs bg-gray-100 px-2 py-1 rounded">{item.relatedOrderId}</code></td><td className="py-4 px-6 text-right font-medium">¥{item.amount.toLocaleString()}</td><td className="py-4 px-6 text-center"><span className={`status-tag ${cfg.color}`}><Icon className="w-3.5 h-3.5" /><span>{cfg.label}</span></span></td><td className="py-4 px-6"><div className="flex items-center justify-end gap-2"><button onClick={() => router.push(`/console/seller/contracts/${item.id}`)} className="p-2 hover:bg-gray-100 rounded-lg" title="详情"><FileText className="w-4 h-4 text-gray-600" /></button><button className="p-2 hover:bg-gray-100 rounded-lg" title="下载"><Download className="w-4 h-4 text-gray-600" /></button><button className="p-2 hover:bg-gray-100 rounded-lg" title="开票"><Receipt className="w-4 h-4 text-gray-600" /></button></div></td></tr>
                )
              })}
              {paged.length === 0 && (
                <tr>
                  <td colSpan={6} className="py-14 text-center text-sm text-gray-500">
                    暂无符合筛选条件的合同或发票
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>}
        pagination={<PaginationBar page={page} pageSize={pageSize} total={filtered.length} onPageChange={setPage} onPageSizeChange={setPageSize} />}
      />
    </>
  )
}
