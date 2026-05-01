'use client'

import { useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import Link from 'next/link'
import { Plus, Package, Eye, Edit, Copy, MoreVertical, Shield, Clock, CheckCircle, AlertCircle, Filter, Layers, ArrowUpDown } from 'lucide-react'
import type { ListingStatus } from '@/types'
import { SELLER_LISTINGS } from '@/lib/seller-listings-data'

const STATUS_CONFIG: Record<ListingStatus, { label: string; color: string; icon: any }> = {
  DRAFT: { label: '草稿', color: 'bg-gray-100 text-gray-800', icon: Edit },
  PENDING_REVIEW: { label: '待审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: AlertCircle },
  LISTED: { label: '已上架', color: 'bg-blue-100 text-blue-800', icon: CheckCircle },
  SUSPENDED: { label: '已暂停', color: 'bg-orange-100 text-orange-800', icon: AlertCircle },
  DELISTED: { label: '已下架', color: 'bg-gray-100 text-gray-800', icon: AlertCircle },
}

type SortBy = 'updated_desc' | 'quality_desc' | 'requests_desc'

export default function SellerListingsPage() {
  const router = useRouter()
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedIndustry, setSelectedIndustry] = useState<string>('all')
  const [sortBy, setSortBy] = useState<SortBy>('updated_desc')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const industries = useMemo(() => Array.from(new Set(SELLER_LISTINGS.map((x) => x.industry))), [])

  const filtered = useMemo(() => {
    const list = SELLER_LISTINGS.filter((listing) => {
      const kw = searchKeyword.toLowerCase()
      const matchKeyword = listing.title.toLowerCase().includes(kw)
      const matchStatus = selectedStatus === 'all' || listing.status === selectedStatus
      const matchIndustry = selectedIndustry === 'all' || listing.industry === selectedIndustry
      return matchKeyword && matchStatus && matchIndustry
    })
    return [...list].sort((a, b) => {
      if (sortBy === 'quality_desc') return b.qualityScore - a.qualityScore
      if (sortBy === 'requests_desc') return b.requestCount - a.requestCount
      return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
    })
  }, [searchKeyword, selectedStatus, selectedIndustry, sortBy])

  const paged = useMemo(() => {
    const start = (page - 1) * pageSize
    return filtered.slice(start, start + pageSize)
  }, [filtered, page, pageSize])

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:listings:write" sessionExpiresAt={sessionExpiresAt} />
      <ConsoleListPageShell
        title="商品管理"
        subtitle="管理数据商品、申请和订阅情况"
        headerAction={<Link href="/console/seller/listings/create" className="flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"><Plus className="w-5 h-5" /><span>创建商品</span></Link>}
        toolbar={<QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索商品名称..."
          onReset={() => { setSearchKeyword(''); setSelectedStatus('all'); setSelectedIndustry('all'); setSortBy('updated_desc'); setPage(1); setPageSize(10) }}
          controls={<div className="grid grid-cols-1 md:grid-cols-4 gap-3"><select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="DRAFT">草稿</option><option value="PENDING_REVIEW">待审核</option><option value="LISTED">已上架</option><option value="SUSPENDED">已暂停</option></select><select value={selectedIndustry} onChange={(e) => setSelectedIndustry(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部行业</option>{industries.map((i) => <option key={i} value={i}>{i}</option>)}</select><select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select><select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="updated_desc">最近更新优先</option><option value="quality_desc">质量评分优先</option><option value="requests_desc">申请数优先</option></select></div>}
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filtered.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分页 {pageSize}/页</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'updated_desc' ? '最近更新' : sortBy === 'quality_desc' ? '质量评分' : '申请数'}</span></>}
        />}
        content={<div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200"><tr><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">商品信息</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">状态</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">价格模式</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">质量评分</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">申请数</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">订阅客户</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">链状态</th><th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th></tr></thead>
            <tbody className="divide-y divide-gray-200">
              {paged.map((listing) => {
                const statusConfig = STATUS_CONFIG[listing.status]
                const StatusIcon = statusConfig.icon
                return (
                  <tr key={listing.id} className="hover:bg-gray-50 cursor-pointer" onDoubleClick={() => router.push(`/console/seller/listings/${listing.id}`)}>
                    <td className="py-4 px-6"><div><div className="font-medium text-gray-900 mb-1">{listing.title}</div><div className="text-xs text-gray-500">{listing.industry} · 更新于 {listing.updatedAt}</div></div></td>
                    <td className="py-4 px-6"><span className={`status-tag ${statusConfig.color}`}><StatusIcon className="w-3.5 h-3.5" /><span>{statusConfig.label}</span></span></td>
                    <td className="py-4 px-6 text-sm text-gray-900">{listing.pricingModel}</td>
                    <td className="py-4 px-6 text-center">{listing.qualityScore > 0 ? listing.qualityScore : '-'}</td>
                    <td className="py-4 px-6 text-center">{listing.requestCount}</td>
                    <td className="py-4 px-6 text-center">{listing.subscriberCount}</td>
                    <td className="py-4 px-6"><div className="flex items-center gap-1">{listing.chainStatus === 'CONFIRMED' ? <Shield className="w-4 h-4 text-success-600" /> : <Clock className="w-4 h-4 text-gray-400" />}<span className="text-xs text-gray-600">{listing.chainStatus === 'CONFIRMED' ? '已确认' : '未提交'}</span></div></td>
                    <td className="py-4 px-6"><div className="flex items-center justify-end gap-2"><button onClick={() => router.push(`/console/seller/listings/${listing.id}`)} className="p-2 text-gray-600 hover:bg-gray-100 rounded-lg"><Eye className="w-4 h-4" /></button><button className="p-2 text-gray-600 hover:bg-gray-100 rounded-lg"><Edit className="w-4 h-4" /></button><button className="p-2 text-gray-600 hover:bg-gray-100 rounded-lg"><Copy className="w-4 h-4" /></button><button className="p-2 text-gray-600 hover:bg-gray-100 rounded-lg"><MoreVertical className="w-4 h-4" /></button></div></td>
                  </tr>
                )
              })}
            </tbody>
          </table>
          {paged.length === 0 && <div className="text-center py-12"><Package className="w-16 h-16 mx-auto text-gray-300 mb-4" /><h3 className="text-lg font-medium text-gray-900 mb-2">暂无商品</h3></div>}
        </div>}
        pagination={<PaginationBar page={page} pageSize={pageSize} total={filtered.length} onPageChange={setPage} onPageSizeChange={setPageSize} />}
      />
    </>
  )
}
