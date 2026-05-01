'use client'

import { useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import InlineExpandableList from '@/components/console/InlineExpandableList'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import {
  CheckCircle,
  XCircle,
  AlertCircle,
  Clock,
  Calendar,
  TrendingUp,
  Filter,
  ArrowUpDown,
  Layers,
  Shield,
  FileText,
  ExternalLink,
  type LucideIcon,
} from 'lucide-react'
import { SELLER_REQUESTS, type SellerAccessRequest } from '@/lib/seller-requests-data'

type GroupBy = 'none' | 'status' | 'risk'
type SortBy = 'latest' | 'risk_desc'
const STATUS_CONFIG: Record<SellerAccessRequest['status'], { label: string; color: string; icon: LucideIcon }> = {
  PENDING_SUPPLIER_REVIEW: { label: '待审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  NEED_MORE_INFO: { label: '需补充', color: 'bg-orange-100 text-orange-800', icon: AlertCircle },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: XCircle },
}

const RISK_CONFIG = {
  LOW: { label: '低风险', color: 'text-green-700 bg-green-50 border border-green-200' },
  MEDIUM: { label: '中风险', color: 'text-yellow-700 bg-yellow-50 border border-yellow-200' },
  HIGH: { label: '高风险', color: 'text-red-700 bg-red-50 border border-red-200' },
}

export default function SellerRequestsPage() {
  const router = useRouter()
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedRisk, setSelectedRisk] = useState<string>('all')
  const [groupBy, setGroupBy] = useState<GroupBy>('none')
  const [sortBy, setSortBy] = useState<SortBy>('latest')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredRequests = useMemo(() => {
    const list = SELLER_REQUESTS.filter((request) => {
      const key = searchKeyword.trim().toLowerCase()
      const matchesKeyword =
        !key ||
        request.buyerName.toLowerCase().includes(key) ||
        request.listingTitle.toLowerCase().includes(key) ||
        request.requestId.toLowerCase().includes(key)
      const matchesStatus = selectedStatus === 'all' || request.status === selectedStatus
      const matchesRisk = selectedRisk === 'all' || request.riskLevel === selectedRisk
      return matchesKeyword && matchesStatus && matchesRisk
    })

    const riskRank = { LOW: 1, MEDIUM: 2, HIGH: 3 }
    list.sort((a, b) => {
      if (sortBy === 'risk_desc') return riskRank[b.riskLevel] - riskRank[a.riskLevel]
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    })

    return list
  }, [searchKeyword, selectedStatus, selectedRisk, sortBy])

  const pagedRequests = useMemo(() => {
    const start = (page - 1) * pageSize
    return filteredRequests.slice(start, start + pageSize)
  }, [filteredRequests, page, pageSize])

  const groupedRequests = useMemo(() => {
    if (groupBy === 'none') return [{ label: '全部结果', items: pagedRequests }]
    const map = new Map<string, SellerAccessRequest[]>()
    for (const req of pagedRequests) {
      const key = groupBy === 'status' ? STATUS_CONFIG[req.status].label : RISK_CONFIG[req.riskLevel].label
      map.set(key, [...(map.get(key) || []), req])
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }))
  }, [pagedRequests, groupBy])

  const handleApprove = (request: SellerAccessRequest) => alert(`通过申请: ${request.requestId}`)
  const handleReject = (request: SellerAccessRequest) => alert(`拒绝申请: ${request.requestId}`)
  const handleRequestMoreInfo = (request: SellerAccessRequest) => alert(`要求补充材料: ${request.requestId}`)

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:requests:write" sessionExpiresAt={sessionExpiresAt} />

      <ConsoleListPageShell
        title="申请审批"
        subtitle="处理买方的数据访问申请与审批动作"
        toolbar={
          <QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索买方、商品、申请ID..."
          onReset={() => {
            setSearchKeyword('')
            setSelectedStatus('all')
            setSelectedRisk('all')
            setGroupBy('none')
            setSortBy('latest')
            setPage(1)
            setPageSize(10)
          }}
          controls={
            <div className="grid grid-cols-1 md:grid-cols-5 gap-3">
              <select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="PENDING_SUPPLIER_REVIEW">待审核</option><option value="NEED_MORE_INFO">需补充</option><option value="APPROVED">已通过</option><option value="REJECTED">已拒绝</option></select>
              <select value={selectedRisk} onChange={(e) => setSelectedRisk(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部风险</option><option value="LOW">低风险</option><option value="MEDIUM">中风险</option><option value="HIGH">高风险</option></select>
              <select value={groupBy} onChange={(e) => setGroupBy(e.target.value as GroupBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="none">不分组</option><option value="status">按状态分组</option><option value="risk">按风险分组</option></select>
              <select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select>
              <select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="latest">最新提交优先</option><option value="risk_desc">高风险优先</option></select>
            </div>
          }
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filteredRequests.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分组 {groupedRequests.length}</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'latest' ? '最新提交' : '高风险优先'}</span></>}
        />
        }
        content={
          <>
          {groupedRequests.map((group) => (
            <section key={group.label}>
              {groupBy !== 'none' && <div className="mb-2 text-sm font-semibold text-gray-700">{group.label} <span className="text-gray-400 font-normal">({group.items.length})</span></div>}
              <InlineExpandableList
                items={group.items}
                getKey={(req) => req.id}
                selectedKey={selectedId}
                onSelect={setSelectedId}
                getItemId={(req) => `seller-request-card-${req.id}`}
                onOpenDetail={(req) => router.push(`/console/seller/requests/${req.requestId}`)}
                renderSummary={(request) => {
                  const StatusIcon = STATUS_CONFIG[request.status].icon
                  return (
                    <>
                      <div className="flex items-start justify-between mb-3">
                        <div>
                          <h3 className="text-lg font-bold text-gray-900 mb-1">{request.buyerName}</h3>
                          <div className="text-sm text-gray-600">
                            申请商品: <span className="font-medium text-gray-900">{request.listingTitle}</span> · {request.planName}
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className={`status-tag ${STATUS_CONFIG[request.status].color}`}><StatusIcon className="w-3.5 h-3.5" /><span>{STATUS_CONFIG[request.status].label}</span></span>
                          <span className={`text-xs px-2 py-1 rounded-full font-medium ${RISK_CONFIG[request.riskLevel].color}`}>{RISK_CONFIG[request.riskLevel].label}</span>
                        </div>
                      </div>
                      <div className="mb-4 pb-4 border-b border-gray-100"><div className="text-xs text-gray-500 mb-1">使用用途</div><p className="text-sm text-gray-900 line-clamp-2">{request.usagePurpose}</p></div>
                      <div className="flex items-center justify-between text-xs text-gray-500"><div className="inline-flex items-center gap-1"><Calendar className="w-3 h-3" /><span>提交时间：{request.createdAt.split(' ')[0]}</span></div><div className="inline-flex items-center gap-1"><TrendingUp className="w-3 h-3" /><span>预计用量：{request.expectedUsage}</span></div></div>
                    </>
                  )
                }}
                renderExpanded={(request) => (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
                    <div className="space-y-4">
                      <div><div className="text-xs text-gray-500 mb-1">Request ID</div><code className="block bg-gray-50 text-xs px-2 py-1 rounded text-gray-900 break-all">{request.requestId}</code></div>
                      <div><div className="text-xs text-gray-500 mb-1">买方主体</div><div className="text-sm font-medium text-gray-900">{request.buyerSubject}</div></div>
                      <div className="text-sm space-y-2">
                        <div className="flex justify-between"><span className="text-gray-600">工作流状态</span><span className="font-medium text-gray-900">{request.workflowStatus}</span></div>
                        <div className="flex justify-between"><span className="text-gray-600">链状态</span><span className="font-medium text-gray-900">{request.chainStatus}</span></div>
                        <div className="flex justify-between"><span className="text-gray-600">投影状态</span><span className="font-medium text-gray-900">{request.projectionStatus}</span></div>
                      </div>
                      <div className="rounded-lg border border-green-200 bg-green-50 px-3 py-2 inline-flex items-center gap-2 text-xs text-green-800"><Shield className="w-4 h-4" /><span className="font-medium">审批链路可追踪</span></div>
                    </div>
                    <div className="space-y-2">
                      <button onClick={(e) => { e.stopPropagation(); router.push(`/console/seller/requests/${request.requestId}`) }} className="w-full h-10 px-4 border border-primary-300 text-primary-700 rounded-lg hover:bg-primary-50 text-sm font-medium inline-flex items-center justify-center gap-2"><ExternalLink className="w-4 h-4" /><span>进入申请详情页</span></button>
                      <button onClick={(e) => { e.stopPropagation(); handleApprove(request) }} className="w-full h-10 px-4 bg-success-600 text-white rounded-lg hover:bg-success-700 text-sm font-medium">通过申请</button>
                      <button onClick={(e) => { e.stopPropagation(); handleRequestMoreInfo(request) }} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><FileText className="w-4 h-4" /><span>要求补充材料</span></button>
                      <button onClick={(e) => { e.stopPropagation(); handleReject(request) }} className="w-full h-10 px-4 border border-red-300 text-red-600 rounded-lg hover:bg-red-50 text-sm font-medium">拒绝申请</button>
                    </div>
                  </div>
                )}
              />
            </section>
          ))}
          </>
        }
        pagination={<PaginationBar page={page} pageSize={pageSize} total={filteredRequests.length} onPageChange={setPage} onPageSizeChange={setPageSize} />}
      />
    </>
  )
}
