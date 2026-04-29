'use client'

import { useEffect, useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import InlineExpandableList from '@/components/console/InlineExpandableList'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import { Filter, Clock, CheckCircle, XCircle, AlertCircle, FileText, TrendingUp, Shield, Layers, ArrowUpDown } from 'lucide-react'
import { AccessRequest, MOCK_REQUESTS, REQ_STATUS_CONFIG } from '@/lib/buyer-requests-data'

const STATUS_ICON = { DRAFT: Clock, SUBMITTED: Clock, PENDING_SUPPLIER_REVIEW: Clock, PENDING_PLATFORM_REVIEW: Clock, NEED_MORE_INFO: AlertCircle, APPROVED: CheckCircle, REJECTED: XCircle, CANCELLED: XCircle }
type GroupBy = 'none' | 'status' | 'supplier'
type SortBy = 'updated_desc' | 'created_desc'

export default function BuyerRequestsPage() {
  const router = useRouter()
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedSupplier, setSelectedSupplier] = useState<string>('all')
  const [groupBy, setGroupBy] = useState<GroupBy>('none')
  const [sortBy, setSortBy] = useState<SortBy>('updated_desc')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [highlightedId, setHighlightedId] = useState<string | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const supplierOptions = useMemo(() => Array.from(new Set(MOCK_REQUESTS.map((r) => r.supplierName))), [])

  const filteredRequests = useMemo(() => {
    const list = MOCK_REQUESTS.filter((req) => {
      const kw = searchKeyword.toLowerCase()
      const matchesKeyword = req.listingTitle.toLowerCase().includes(kw) || req.supplierName.toLowerCase().includes(kw) || req.requestId.toLowerCase().includes(kw)
      const matchesStatus = selectedStatus === 'all' || req.status === selectedStatus
      const matchesSupplier = selectedSupplier === 'all' || req.supplierName === selectedSupplier
      return matchesKeyword && matchesStatus && matchesSupplier
    })
    return [...list].sort((a, b) => sortBy === 'created_desc' ? new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime() : new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
  }, [searchKeyword, selectedStatus, selectedSupplier, sortBy])

  const pagedRequests = useMemo(() => {
    const start = (page - 1) * pageSize
    return filteredRequests.slice(start, start + pageSize)
  }, [filteredRequests, page, pageSize])

  const groupedRequests = useMemo(() => {
    if (groupBy === 'none') return [{ label: '全部结果', items: pagedRequests }]
    const map = new Map<string, AccessRequest[]>()
    for (const req of pagedRequests) {
      const key = groupBy === 'status' ? REQ_STATUS_CONFIG[req.status].label : req.supplierName
      map.set(key, [...(map.get(key) || []), req])
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }))
  }, [pagedRequests, groupBy])

  useEffect(() => { setPage(1) }, [searchKeyword, selectedStatus, selectedSupplier, groupBy, sortBy, pageSize])

  useEffect(() => {
    const params = new URLSearchParams(window.location.search)
    const focusRequestId = params.get('focus')
    if (!focusRequestId) return
    const matched = MOCK_REQUESTS.find((r) => r.requestId === focusRequestId)
    if (!matched) return
    setSelectedId(matched.id)
    setHighlightedId(matched.id)
    requestAnimationFrame(() => document.getElementById(`request-card-${matched.id}`)?.scrollIntoView({ behavior: 'smooth', block: 'center' }))
    const timer = window.setTimeout(() => setHighlightedId(null), 2400)
    return () => window.clearTimeout(timer)
  }, [])

  return (
    <>
      <SessionIdentityBar subjectName="某某科技有限公司" roleName="买家管理员" tenantId="tenant_buyer_001" scope="buyer:requests:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="mb-8"><h1 className="text-3xl font-bold text-gray-900 mb-2">我的申请</h1><p className="text-gray-600">查看申请流程、审核反馈、链状态与后续动作</p></div>

        <QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索商品、供应商、申请ID..."
          onReset={() => { setSearchKeyword(''); setSelectedStatus('all'); setSelectedSupplier('all'); setGroupBy('none'); setSortBy('updated_desc'); setPage(1); setPageSize(10) }}
          controls={<div className="grid grid-cols-1 md:grid-cols-5 gap-3"><select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="PENDING_SUPPLIER_REVIEW">待供应商审核</option><option value="PENDING_PLATFORM_REVIEW">待平台审核</option><option value="NEED_MORE_INFO">需补充材料</option><option value="APPROVED">已通过</option><option value="REJECTED">已拒绝</option></select><select value={selectedSupplier} onChange={(e) => setSelectedSupplier(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部供应商</option>{supplierOptions.map((s) => <option key={s} value={s}>{s}</option>)}</select><select value={groupBy} onChange={(e) => setGroupBy(e.target.value as GroupBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="none">不分组</option><option value="status">按状态分组</option><option value="supplier">按供应商分组</option></select><select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select><select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="updated_desc">最近更新优先</option><option value="created_desc">最近创建优先</option></select></div>}
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filteredRequests.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分组 {groupedRequests.length}</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'updated_desc' ? '最近更新' : '最近创建'}</span></>}
        />

        <div className="space-y-5">
          {groupedRequests.map((group) => (
            <section key={group.label}>
              {groupBy !== 'none' && <div className="mb-2 text-sm font-semibold text-gray-700">{group.label} <span className="text-gray-400 font-normal">({group.items.length})</span></div>}
              <InlineExpandableList items={group.items} getKey={(req) => req.id} selectedKey={selectedId} onSelect={setSelectedId} highlightedKey={highlightedId} getItemId={(req) => `request-card-${req.id}`} onOpenDetail={(req) => router.push(`/console/buyer/requests/${req.requestId}`)}
                renderSummary={(request) => { const StatusIcon = STATUS_ICON[request.status]; return (<><div className="flex items-start justify-between mb-3"><div><h3 className="text-lg font-bold text-gray-900 mb-1">{request.listingTitle}</h3><div className="text-sm text-gray-600">{request.supplierName} · <span className="font-medium text-gray-900">{request.plan}</span></div></div><span className={`status-tag ${REQ_STATUS_CONFIG[request.status].color}`}><StatusIcon className="w-3.5 h-3.5" /><span>{REQ_STATUS_CONFIG[request.status].label}</span></span></div><div className="mb-4 pb-4 border-b border-gray-100"><div className="text-xs text-gray-500 mb-1">使用用途</div><p className="text-sm text-gray-900 line-clamp-2">{request.usagePurpose}</p></div><div className="flex items-center justify-between text-xs text-gray-500"><span>申请时间：{request.createdAt.split(' ')[0]}</span><span>更新时间：{request.updatedAt.split(' ')[0]}</span></div></>) }}
                renderExpanded={(request) => (<div className="grid grid-cols-1 lg:grid-cols-2 gap-5"><div className="space-y-4"><div><div className="text-xs text-gray-500 mb-1">Request ID</div><code className="block bg-gray-50 text-xs px-2 py-1 rounded text-gray-900 break-all">{request.requestId}</code></div><div className="text-sm space-y-2"><div className="flex justify-between"><span className="text-gray-600">申请状态</span><span className={`status-tag text-xs ${REQ_STATUS_CONFIG[request.status].color}`}>{REQ_STATUS_CONFIG[request.status].label}</span></div><div className="flex justify-between"><span className="text-gray-600">链状态</span><span className="font-medium text-gray-900">{request.chainStatus}</span></div><div className="flex justify-between"><span className="text-gray-600">投影状态</span><span className="font-medium text-gray-900">{request.projectionStatus}</span></div><div className="text-gray-900">预计用量：<span className="font-medium">{request.expectedUsage}</span></div></div>{request.reviewNotes && <div className="rounded-lg border border-orange-200 bg-orange-50 p-3 text-xs text-orange-900">{request.reviewNotes}</div>}<div className="rounded-lg border border-green-200 bg-green-50 px-3 py-2 inline-flex items-center gap-2 text-xs text-green-800"><Shield className="w-4 h-4" /><span className="font-medium">申请链路可追踪</span></div></div><div className="space-y-2"><button onClick={(e) => { e.stopPropagation(); router.push(`/console/buyer/requests/${request.requestId}`) }} className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium">进入申请详情页</button>{request.status === 'NEED_MORE_INFO' && <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><FileText className="w-4 h-4" /><span>补充材料</span></button>}{request.status === 'APPROVED' && <button onClick={(e) => { e.stopPropagation(); router.push('/console/buyer/subscriptions') }} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><TrendingUp className="w-4 h-4" /><span>前往订阅管理</span></button>}</div></div>)} />
            </section>
          ))}
        </div>

        <PaginationBar page={page} pageSize={pageSize} total={filteredRequests.length} onPageChange={setPage} onPageSizeChange={setPageSize} />
      </div>
    </>
  )
}
