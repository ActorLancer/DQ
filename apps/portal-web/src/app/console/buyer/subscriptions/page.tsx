'use client'

import { useEffect, useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import InlineExpandableList from '@/components/console/InlineExpandableList'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import { Filter, CheckCircle, Clock, AlertCircle, XCircle, KeyRound, Activity, RefreshCw, Shield, ArrowUpDown, Layers } from 'lucide-react'
import { MOCK_SUBSCRIPTIONS, SUB_STATUS_CONFIG, type Subscription } from '@/lib/buyer-subscriptions-data'

const STATUS_ICON = { ACTIVE: CheckCircle, EXPIRED: Clock, SUSPENDED: AlertCircle, REVOKED: XCircle }
type GroupBy = 'none' | 'status' | 'supplier'
type SortBy = 'recent_call' | 'quota_usage' | 'total_calls'

export default function BuyerSubscriptionsPage() {
  const router = useRouter()
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedSupplier, setSelectedSupplier] = useState<string>('all')
  const [groupBy, setGroupBy] = useState<GroupBy>('none')
  const [sortBy, setSortBy] = useState<SortBy>('recent_call')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [highlightedId, setHighlightedId] = useState<string | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const supplierOptions = useMemo(() => Array.from(new Set(MOCK_SUBSCRIPTIONS.map((s) => s.supplierName))), [])

  const filteredSubscriptions = useMemo(() => {
    const list = MOCK_SUBSCRIPTIONS.filter((sub) => {
      const kw = searchKeyword.toLowerCase()
      const matchesKeyword = sub.listingTitle.toLowerCase().includes(kw) || sub.supplierName.toLowerCase().includes(kw) || sub.subscriptionId.toLowerCase().includes(kw)
      const matchesStatus = selectedStatus === 'all' || sub.status === selectedStatus
      const matchesSupplier = selectedSupplier === 'all' || sub.supplierName === selectedSupplier
      return matchesKeyword && matchesStatus && matchesSupplier
    })

    return [...list].sort((a, b) => {
      if (sortBy === 'total_calls') return b.apiCallsTotal - a.apiCallsTotal
      if (sortBy === 'quota_usage') {
        const aq = a.quota ? a.usedQuota / a.quota : 0
        const bq = b.quota ? b.usedQuota / b.quota : 0
        return bq - aq
      }
      return new Date(b.lastCallAt).getTime() - new Date(a.lastCallAt).getTime()
    })
  }, [searchKeyword, selectedStatus, selectedSupplier, sortBy])

  const pagedSubscriptions = useMemo(() => {
    const start = (page - 1) * pageSize
    return filteredSubscriptions.slice(start, start + pageSize)
  }, [filteredSubscriptions, page, pageSize])

  const groupedSubscriptions = useMemo(() => {
    if (groupBy === 'none') return [{ label: '全部结果', items: pagedSubscriptions }]
    const map = new Map<string, Subscription[]>()
    for (const sub of pagedSubscriptions) {
      const key = groupBy === 'status' ? SUB_STATUS_CONFIG[sub.status].label : sub.supplierName
      map.set(key, [...(map.get(key) || []), sub])
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }))
  }, [pagedSubscriptions, groupBy])

  useEffect(() => { setPage(1) }, [searchKeyword, selectedStatus, selectedSupplier, groupBy, sortBy, pageSize])

  useEffect(() => {
    const params = new URLSearchParams(window.location.search)
    const focusSubscriptionId = params.get('focus')
    if (!focusSubscriptionId) return
    const matched = MOCK_SUBSCRIPTIONS.find((s) => s.subscriptionId === focusSubscriptionId)
    if (!matched) return
    setSelectedId(matched.id)
    setHighlightedId(matched.id)
    requestAnimationFrame(() => {
      document.getElementById(`subscription-card-${matched.id}`)?.scrollIntoView({ behavior: 'smooth', block: 'center' })
    })
    const timer = window.setTimeout(() => setHighlightedId(null), 2400)
    return () => window.clearTimeout(timer)
  }, [])

  return (
    <>
      <SessionIdentityBar subjectName="某某科技有限公司" roleName="买家管理员" tenantId="tenant_buyer_001" scope="buyer:subscriptions:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="mb-8"><h1 className="text-3xl font-bold text-gray-900 mb-2">我的订阅</h1><p className="text-gray-600">管理数据订阅、配额、授权与调用使用情况</p></div>

        <QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索商品、供应商、订阅ID..."
          onReset={() => { setSearchKeyword(''); setSelectedStatus('all'); setSelectedSupplier('all'); setGroupBy('none'); setSortBy('recent_call'); setPage(1); setPageSize(10) }}
          controls={
            <div className="grid grid-cols-1 md:grid-cols-5 gap-3">
              <select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="ACTIVE">活跃</option><option value="EXPIRED">已过期</option><option value="SUSPENDED">已暂停</option><option value="REVOKED">已撤销</option></select>
              <select value={selectedSupplier} onChange={(e) => setSelectedSupplier(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部供应商</option>{supplierOptions.map((s) => <option key={s} value={s}>{s}</option>)}</select>
              <select value={groupBy} onChange={(e) => setGroupBy(e.target.value as GroupBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="none">不分组</option><option value="status">按状态分组</option><option value="supplier">按供应商分组</option></select>
              <select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select>
              <select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="recent_call">最近调用优先</option><option value="quota_usage">配额使用率优先</option><option value="total_calls">总调用量优先</option></select>
            </div>
          }
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filteredSubscriptions.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分组 {groupedSubscriptions.length}</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'recent_call' ? '最近调用' : sortBy === 'quota_usage' ? '配额使用率' : '总调用量'}</span></>}
        />

        <div className="space-y-5">
          {groupedSubscriptions.map((group) => (
            <section key={group.label}>
              {groupBy !== 'none' && <div className="mb-2 text-sm font-semibold text-gray-700">{group.label} <span className="text-gray-400 font-normal">({group.items.length})</span></div>}
              <InlineExpandableList
                items={group.items}
                getKey={(sub) => sub.id}
                selectedKey={selectedId}
                onSelect={setSelectedId}
                highlightedKey={highlightedId}
                getItemId={(sub) => `subscription-card-${sub.id}`}
                onOpenDetail={(sub) => router.push(`/console/buyer/subscriptions/${sub.subscriptionId}`)}
                renderSummary={(sub) => {
                  const StatusIcon = STATUS_ICON[sub.status]
                  return (
                    <>
                      <div className="flex items-start justify-between mb-4"><div><h3 className="text-lg font-bold text-gray-900 mb-1">{sub.listingTitle}</h3><div className="text-sm text-gray-600">{sub.supplierName} · <span className="font-medium text-gray-900">{sub.plan}</span></div></div><span className={`status-tag ${SUB_STATUS_CONFIG[sub.status].color}`}><StatusIcon className="w-3.5 h-3.5" /><span>{SUB_STATUS_CONFIG[sub.status].label}</span></span></div>
                      <div className="grid grid-cols-3 gap-4"><div><div className="text-xs text-gray-500 mb-1">今日调用</div><div className="text-lg font-semibold text-gray-900">{sub.apiCallsToday}</div></div><div><div className="text-xs text-gray-500 mb-1">总调用</div><div className="text-lg font-semibold text-gray-900">{sub.apiCallsTotal.toLocaleString()}</div></div><div><div className="text-xs text-gray-500 mb-1">有效期</div><div className="font-medium text-gray-900">{sub.endsAt || '无限期'}</div></div></div>
                    </>
                  )
                }}
                renderExpanded={(sub) => (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
                    <div className="space-y-4">
                      <div><div className="text-xs text-gray-500 mb-1">Subscription ID</div><code className="block bg-gray-50 text-xs px-2 py-1 rounded text-gray-900 break-all">{sub.subscriptionId}</code></div>
                      <div className="text-sm space-y-2"><div className="flex justify-between"><span className="text-gray-600">状态</span><span className={`status-tag text-xs ${SUB_STATUS_CONFIG[sub.status].color}`}>{SUB_STATUS_CONFIG[sub.status].label}</span></div><div className="flex justify-between"><span className="text-gray-600">授权区间</span><span className="font-medium text-gray-900">{sub.startsAt} ~ {sub.endsAt || '无限期'}</span></div><div className="flex justify-between"><span className="text-gray-600">最近调用</span><span className="font-medium text-gray-900 text-xs">{sub.lastCallAt}</span></div></div>
                      <div className="rounded-lg border border-green-200 bg-green-50 px-3 py-2 inline-flex items-center gap-2 text-xs text-green-800"><Shield className="w-4 h-4" /><span className="font-medium">链上凭证：{sub.chainProofId}</span></div>
                    </div>
                    <div className="space-y-2">
                      <button onClick={(e) => { e.stopPropagation(); router.push(`/console/buyer/subscriptions/${sub.subscriptionId}`) }} className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium">进入订阅详情页</button>
                      <button onClick={(e) => { e.stopPropagation(); router.push('/console/buyer/api-keys') }} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><KeyRound className="w-4 h-4" /><span>查看 API 密钥</span></button>
                      <button onClick={(e) => { e.stopPropagation(); router.push('/console/buyer/usage') }} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><Activity className="w-4 h-4" /><span>查看调用分析</span></button>
                      <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><RefreshCw className="w-4 h-4" /><span>续订</span></button>
                    </div>
                  </div>
                )}
              />
            </section>
          ))}
        </div>

        <PaginationBar page={page} pageSize={pageSize} total={filteredSubscriptions.length} onPageChange={setPage} onPageSizeChange={setPageSize} />
      </div>
    </>
  )
}
