'use client'

import { useMemo, useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import { CheckCircle, XCircle, Clock, AlertTriangle, Tag, Filter, Layers, ArrowUpDown } from 'lucide-react'

interface ListingReview {
  id: string
  listingId: string
  title: string
  supplierName: string
  industry: string
  dataType: string
  pricingModel: string
  basePrice: number
  status: 'PENDING' | 'APPROVED' | 'REJECTED' | 'REVISION_REQUIRED'
  submittedAt: string
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
  qualityScore: number
}

const MOCK_LISTINGS: ListingReview[] = [
  { id: 'review_001', listingId: 'listing_new_001', title: '全国企业工商信息实时查询API', supplierName: '天眼数据科技', industry: '企业征信', dataType: 'API', pricingModel: '按量计费', basePrice: 0.5, status: 'PENDING', submittedAt: '2026-04-29 09:30:00', riskLevel: 'LOW', qualityScore: 92 },
  { id: 'review_002', listingId: 'listing_new_002', title: '个人消费行为画像数据', supplierName: '智慧消费研究院', industry: '消费分析', dataType: 'Dataset', pricingModel: '订阅制', basePrice: 9999, status: 'PENDING', submittedAt: '2026-04-29 10:15:00', riskLevel: 'HIGH', qualityScore: 78 },
  { id: 'review_003', listingId: 'listing_new_003', title: '物流轨迹实时追踪数据', supplierName: '快递物流联盟', industry: '物流', dataType: 'Stream', pricingModel: '按量计费', basePrice: 0.1, status: 'REVISION_REQUIRED', submittedAt: '2026-04-28 14:20:00', riskLevel: 'MEDIUM', qualityScore: 85 },
  { id: 'review_004', listingId: 'listing_new_004', title: '金融市场实时行情数据', supplierName: '金融数据服务商', industry: '金融', dataType: 'API', pricingModel: '订阅制', basePrice: 19999, status: 'APPROVED', submittedAt: '2026-04-27 11:00:00', riskLevel: 'LOW', qualityScore: 95 },
]

const STATUS_CONFIG = {
  PENDING: { label: '待审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  APPROVED: { label: '已批准', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: XCircle },
  REVISION_REQUIRED: { label: '需修改', color: 'bg-orange-100 text-orange-800', icon: AlertTriangle },
}

type SortBy = 'submitted_desc' | 'quality_desc'

export default function AdminListingsPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [sortBy, setSortBy] = useState<SortBy>('submitted_desc')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filtered = useMemo(() => {
    const list = MOCK_LISTINGS.filter((x) => {
      const kw = searchKeyword.toLowerCase()
      const matchKeyword = x.title.toLowerCase().includes(kw) || x.supplierName.toLowerCase().includes(kw)
      const matchStatus = selectedStatus === 'all' || x.status === selectedStatus
      return matchKeyword && matchStatus
    })
    return [...list].sort((a, b) => sortBy === 'quality_desc' ? b.qualityScore - a.qualityScore : new Date(b.submittedAt).getTime() - new Date(a.submittedAt).getTime())
  }, [searchKeyword, selectedStatus, sortBy])

  const paged = useMemo(() => filtered.slice((page - 1) * pageSize, (page - 1) * pageSize + pageSize), [filtered, page, pageSize])

  return (
    <>
      <SessionIdentityBar subjectName="数据交易平台" roleName="平台管理员" tenantId="tenant_platform_001" scope="admin:listings:approve" sessionExpiresAt={sessionExpiresAt} />
      <ConsoleListPageShell
        title="商品审核"
        subtitle="审核供应商提交的数据商品，确保质量与合规"
        toolbar={<QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索商品名称或供应商..."
          onReset={() => { setSearchKeyword(''); setSelectedStatus('all'); setSortBy('submitted_desc'); setPage(1); setPageSize(10) }}
          controls={<div className="grid grid-cols-1 md:grid-cols-4 gap-3"><select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="PENDING">待审核</option><option value="APPROVED">已批准</option><option value="REJECTED">已拒绝</option><option value="REVISION_REQUIRED">需修改</option></select><select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select><select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="submitted_desc">提交时间优先</option><option value="quality_desc">质量评分优先</option></select><div className="h-10 px-3 rounded-lg border border-gray-200 bg-gray-50 text-xs text-gray-600 flex items-center">平均评分 {filtered.length ? (filtered.reduce((s,d)=>s+d.qualityScore,0)/filtered.length).toFixed(1) : '-'}</div></div>}
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filtered.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分页 {pageSize}/页</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'submitted_desc' ? '提交时间' : '质量评分'}</span></>}
        />}
        content={<div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200"><tr><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">商品信息</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">供应商</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">定价模型</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">质量评分</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">提交时间</th></tr></thead>
            <tbody className="divide-y divide-gray-200">
              {paged.map((listing) => {
                const status = STATUS_CONFIG[listing.status]
                const Icon = status.icon
                return (
                  <tr key={listing.id} className="hover:bg-gray-50"><td className="py-4 px-6"><div className="font-medium text-gray-900 mb-1">{listing.title}</div><div className="flex items-center gap-2 text-xs text-gray-500"><Tag className="w-3 h-3" /><span>{listing.industry}</span><span>•</span><span>{listing.dataType}</span></div></td><td className="py-4 px-6"><div className="text-sm text-gray-900">{listing.supplierName}</div></td><td className="py-4 px-6 text-center"><div className="text-sm text-gray-900">{listing.pricingModel}</div><div className="text-xs text-gray-500">¥{listing.basePrice.toLocaleString()}</div></td><td className="py-4 px-6 text-center">{listing.qualityScore}</td><td className="py-4 px-6 text-center"><span className={`status-tag ${status.color}`}><Icon className="w-3.5 h-3.5" /><span>{status.label}</span></span></td><td className="py-4 px-6 text-sm text-gray-700">{listing.submittedAt.split(' ')[0]}</td></tr>
                )
              })}
              {paged.length === 0 && (
                <tr>
                  <td colSpan={6} className="py-14 text-center text-sm text-gray-500">
                    暂无符合筛选条件的商品
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
