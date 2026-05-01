'use client'

import { useMemo, useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import InlineExpandableList from '@/components/console/InlineExpandableList'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import {
  Database,
  Link2,
  CheckCircle,
  AlertTriangle,
  RefreshCw,
  Shield,
  Activity,
  Filter,
  Layers,
  ArrowUpDown,
} from 'lucide-react'

interface ConsistencyCheck {
  id: string
  requestId?: string
  txHash?: string
  businessId?: string
  businessType: 'LISTING' | 'ACCESS_REQUEST' | 'ORDER' | 'SUBSCRIPTION' | 'DELIVERY'
  chainStatus: 'NOT_SUBMITTED' | 'SUBMITTED' | 'CONFIRMED' | 'FAILED'
  projectionStatus: 'PENDING' | 'PROJECTED' | 'OUT_OF_SYNC' | 'FAILED'
  dbRecord: boolean
  chainRecord: boolean
  projectionRecord: boolean
  consistent: boolean
  inconsistencyType?: string
  createdAt: string
  lastCheckedAt: string
}

type GroupBy = 'none' | 'biz_type' | 'consistency'
type SortBy = 'latest' | 'inconsistent_first'

const MOCK_CHECKS: ConsistencyCheck[] = [
  {
    id: 'check_001',
    requestId: 'request_20260428_001',
    txHash: '0x1234567890abcdef1234567890abcdef12345678',
    businessId: 'listing_001',
    businessType: 'LISTING',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
    dbRecord: true,
    chainRecord: true,
    projectionRecord: true,
    consistent: true,
    createdAt: '2026-04-28 10:00:00',
    lastCheckedAt: '2026-04-28 15:30:00',
  },
  {
    id: 'check_002',
    requestId: 'request_20260428_002',
    txHash: '0xabcdef1234567890abcdef1234567890abcdef12',
    businessId: 'request_002',
    businessType: 'ACCESS_REQUEST',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'OUT_OF_SYNC',
    dbRecord: true,
    chainRecord: true,
    projectionRecord: false,
    consistent: false,
    inconsistencyType: 'PROJECTION_MISSING',
    createdAt: '2026-04-28 09:00:00',
    lastCheckedAt: '2026-04-28 15:25:00',
  },
  {
    id: 'check_003',
    requestId: 'request_20260427_003',
    businessId: 'order_003',
    businessType: 'ORDER',
    chainStatus: 'SUBMITTED',
    projectionStatus: 'PENDING',
    dbRecord: true,
    chainRecord: false,
    projectionRecord: false,
    consistent: false,
    inconsistencyType: 'CHAIN_NOT_CONFIRMED',
    createdAt: '2026-04-27 16:00:00',
    lastCheckedAt: '2026-04-28 15:20:00',
  },
  {
    id: 'check_004',
    requestId: 'request_20260427_004',
    txHash: '0x567890abcdef1234567890abcdef1234567890ab',
    businessId: 'subscription_004',
    businessType: 'SUBSCRIPTION',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
    dbRecord: true,
    chainRecord: true,
    projectionRecord: true,
    consistent: true,
    createdAt: '2026-04-27 14:00:00',
    lastCheckedAt: '2026-04-28 15:15:00',
  },
  {
    id: 'check_005',
    requestId: 'request_20260427_005',
    txHash: '0x234567890abcdef1234567890abcdef1234567890',
    businessId: 'delivery_005',
    businessType: 'DELIVERY',
    chainStatus: 'FAILED',
    projectionStatus: 'FAILED',
    dbRecord: true,
    chainRecord: false,
    projectionRecord: false,
    consistent: false,
    inconsistencyType: 'CHAIN_SUBMISSION_FAILED',
    createdAt: '2026-04-27 11:00:00',
    lastCheckedAt: '2026-04-28 15:10:00',
  },
]

const BUSINESS_TYPE_CONFIG = {
  LISTING: { label: '商品', color: 'bg-blue-100 text-blue-800' },
  ACCESS_REQUEST: { label: '访问申请', color: 'bg-purple-100 text-purple-800' },
  ORDER: { label: '订单', color: 'bg-green-100 text-green-800' },
  SUBSCRIPTION: { label: '订阅', color: 'bg-yellow-100 text-yellow-800' },
  DELIVERY: { label: '交付', color: 'bg-pink-100 text-pink-800' },
}

const CHAIN_STATUS_CONFIG = {
  NOT_SUBMITTED: { label: '未提交', color: 'bg-gray-100 text-gray-800' },
  SUBMITTED: { label: '已提交', color: 'bg-yellow-100 text-yellow-800' },
  CONFIRMED: { label: '已确认', color: 'bg-green-100 text-green-800' },
  FAILED: { label: '失败', color: 'bg-red-100 text-red-800' },
}

const PROJECTION_STATUS_CONFIG = {
  PENDING: { label: '待投影', color: 'bg-gray-100 text-gray-800' },
  PROJECTED: { label: '已投影', color: 'bg-green-100 text-green-800' },
  OUT_OF_SYNC: { label: '不同步', color: 'bg-orange-100 text-orange-800' },
  FAILED: { label: '失败', color: 'bg-red-100 text-red-800' },
}

const INCONSISTENCY_TYPE_CONFIG: Record<string, { label: string; description: string }> = {
  PROJECTION_MISSING: {
    label: '投影缺失',
    description: '链上记录已确认，但投影数据库中缺少对应记录',
  },
  CHAIN_NOT_CONFIRMED: {
    label: '链未确认',
    description: '数据库有记录，但链上交易尚未确认',
  },
  CHAIN_SUBMISSION_FAILED: {
    label: '链提交失败',
    description: '向区块链提交交易失败',
  },
  DATA_MISMATCH: {
    label: '数据不一致',
    description: '数据库、链上、投影三者数据内容不一致',
  },
}

export default function AdminConsistencyPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedFilter, setSelectedFilter] = useState<string>('all')
  const [selectedBizType, setSelectedBizType] = useState<string>('all')
  const [groupBy, setGroupBy] = useState<GroupBy>('none')
  const [sortBy, setSortBy] = useState<SortBy>('inconsistent_first')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [isChecking, setIsChecking] = useState(false)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredChecks = useMemo(() => {
    const key = searchKeyword.trim().toLowerCase()
    const list = MOCK_CHECKS.filter((check) => {
      const matchesKeyword =
        !key ||
        check.requestId?.toLowerCase().includes(key) ||
        check.txHash?.toLowerCase().includes(key) ||
        check.businessId?.toLowerCase().includes(key)

      const matchesFilter =
        selectedFilter === 'all' ||
        (selectedFilter === 'inconsistent' && !check.consistent) ||
        (selectedFilter === 'consistent' && check.consistent)

      const matchesBizType = selectedBizType === 'all' || check.businessType === selectedBizType

      return matchesKeyword && matchesFilter && matchesBizType
    })

    list.sort((a, b) => {
      if (sortBy === 'inconsistent_first') {
        if (a.consistent === b.consistent) return new Date(b.lastCheckedAt).getTime() - new Date(a.lastCheckedAt).getTime()
        return a.consistent ? 1 : -1
      }
      return new Date(b.lastCheckedAt).getTime() - new Date(a.lastCheckedAt).getTime()
    })

    return list
  }, [searchKeyword, selectedFilter, selectedBizType, sortBy])

  const pagedChecks = useMemo(() => {
    const start = (page - 1) * pageSize
    return filteredChecks.slice(start, start + pageSize)
  }, [filteredChecks, page, pageSize])

  const groupedChecks = useMemo(() => {
    if (groupBy === 'none') return [{ label: '全部结果', items: pagedChecks }]
    const map = new Map<string, ConsistencyCheck[]>()
    for (const check of pagedChecks) {
      const key = groupBy === 'biz_type' ? BUSINESS_TYPE_CONFIG[check.businessType].label : check.consistent ? '一致' : '不一致'
      map.set(key, [...(map.get(key) || []), check])
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }))
  }, [pagedChecks, groupBy])

  const stats = {
    total: MOCK_CHECKS.length,
    consistent: MOCK_CHECKS.filter((c) => c.consistent).length,
    inconsistent: MOCK_CHECKS.filter((c) => !c.consistent).length,
    consistencyRate: ((MOCK_CHECKS.filter((c) => c.consistent).length / MOCK_CHECKS.length) * 100).toFixed(1),
  }

  const handleRunCheck = () => {
    setIsChecking(true)
    setTimeout(() => setIsChecking(false), 1600)
  }

  return (
    <>
      <SessionIdentityBar subjectName="数据交易平台" roleName="平台管理员" tenantId="tenant_platform_001" scope="admin:consistency:write" sessionExpiresAt={sessionExpiresAt} userName="管理员" />

      <ConsoleListPageShell
        title="一致性检查"
        subtitle="联查数据库、链上状态、投影状态并提供修复入口"
        headerAction={<button
            onClick={handleRunCheck}
            disabled={isChecking}
            className="h-10 px-4 inline-flex items-center gap-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium disabled:opacity-50"
          >
            <RefreshCw className={`w-4 h-4 ${isChecking ? 'animate-spin' : ''}`} />
            <span>{isChecking ? '检查中...' : '运行检查'}</span>
          </button>}
        summaryCards={<div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="flex items-center justify-between mb-4"><div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center"><Database className="w-6 h-6 text-blue-600" /></div></div><div className="text-2xl font-bold text-gray-900 mb-1">{stats.total}</div><div className="text-sm text-gray-600">总检查记录</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="flex items-center justify-between mb-4"><div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center"><CheckCircle className="w-6 h-6 text-green-600" /></div></div><div className="text-2xl font-bold text-gray-900 mb-1">{stats.consistent}</div><div className="text-sm text-gray-600">一致记录</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="flex items-center justify-between mb-4"><div className="w-12 h-12 bg-red-50 rounded-lg flex items-center justify-center"><AlertTriangle className="w-6 h-6 text-red-600" /></div></div><div className="text-2xl font-bold text-gray-900 mb-1">{stats.inconsistent}</div><div className="text-sm text-gray-600">不一致记录</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="flex items-center justify-between mb-4"><div className="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center"><Activity className="w-6 h-6 text-purple-600" /></div></div><div className="text-2xl font-bold text-gray-900 mb-1">{stats.consistencyRate}%</div><div className="text-sm text-gray-600">一致性率</div></div>
        </div>}
        toolbar={<QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索 Request ID、TX Hash、Business ID..."
          onReset={() => {
            setSearchKeyword('')
            setSelectedFilter('all')
            setSelectedBizType('all')
            setGroupBy('none')
            setSortBy('inconsistent_first')
            setPage(1)
            setPageSize(10)
          }}
          controls={
            <div className="grid grid-cols-1 md:grid-cols-5 gap-3">
              <select value={selectedFilter} onChange={(e) => setSelectedFilter(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部记录</option><option value="consistent">仅一致</option><option value="inconsistent">仅不一致</option></select>
              <select value={selectedBizType} onChange={(e) => setSelectedBizType(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部业务类型</option><option value="LISTING">商品</option><option value="ACCESS_REQUEST">访问申请</option><option value="ORDER">订单</option><option value="SUBSCRIPTION">订阅</option><option value="DELIVERY">交付</option></select>
              <select value={groupBy} onChange={(e) => setGroupBy(e.target.value as GroupBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="none">不分组</option><option value="biz_type">按业务类型分组</option><option value="consistency">按一致性分组</option></select>
              <select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select>
              <select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="inconsistent_first">异常优先</option><option value="latest">最新检查</option></select>
            </div>
          }
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filteredChecks.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分组 {groupedChecks.length}</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'inconsistent_first' ? '异常优先' : '最新检查'}</span></>}
        />}
        content={<>
          {groupedChecks.map((group) => (
            <section key={group.label}>
              {groupBy !== 'none' && <div className="mb-2 text-sm font-semibold text-gray-700">{group.label} <span className="text-gray-400 font-normal">({group.items.length})</span></div>}
              <InlineExpandableList
                items={group.items}
                getKey={(check) => check.id}
                selectedKey={selectedId}
                onSelect={setSelectedId}
                getItemId={(check) => `admin-check-card-${check.id}`}
                renderSummary={(check) => (
                  <>
                    <div className="flex items-start justify-between mb-4">
                      <div className="space-y-2">
                        <div className="flex items-center gap-2">
                          <span className={`status-tag text-xs ${BUSINESS_TYPE_CONFIG[check.businessType].color}`}>{BUSINESS_TYPE_CONFIG[check.businessType].label}</span>
                          {check.consistent ? (
                            <span className="status-tag text-xs bg-green-100 text-green-800"><CheckCircle className="w-3 h-3" /><span>一致</span></span>
                          ) : (
                            <span className="status-tag text-xs bg-red-100 text-red-800"><AlertTriangle className="w-3 h-3" /><span>不一致</span></span>
                          )}
                        </div>
                        {check.requestId && <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded inline-block">{check.requestId}</code>}
                      </div>
                      <div className="text-xs text-gray-500">{check.lastCheckedAt}</div>
                    </div>

                    <div className="grid grid-cols-3 gap-3 mb-1">
                      <div className={`p-3 rounded-lg border ${check.dbRecord ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'}`}><div className="text-xs font-medium text-gray-900 mb-1">数据库</div><div className={`text-xs ${check.dbRecord ? 'text-green-700' : 'text-red-700'}`}>{check.dbRecord ? '有记录' : '无记录'}</div></div>
                      <div className={`p-3 rounded-lg border ${check.chainRecord ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'}`}><div className="text-xs font-medium text-gray-900 mb-1">区块链</div><div className={`text-xs ${check.chainRecord ? 'text-green-700' : 'text-red-700'}`}>{check.chainRecord ? '已确认' : '未确认'}</div></div>
                      <div className={`p-3 rounded-lg border ${check.projectionRecord ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'}`}><div className="text-xs font-medium text-gray-900 mb-1">投影</div><div className={`text-xs ${check.projectionRecord ? 'text-green-700' : 'text-red-700'}`}>{check.projectionRecord ? '已投影' : '未投影'}</div></div>
                    </div>
                  </>
                )}
                renderExpanded={(check) => (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
                    <div className="space-y-4">
                      {check.businessId && <div><div className="text-xs text-gray-500 mb-1">Business ID</div><code className="block bg-gray-50 text-xs px-2 py-1 rounded text-gray-900 break-all">{check.businessId}</code></div>}
                      {check.txHash && <div><div className="text-xs text-gray-500 mb-1">TX Hash</div><code className="block bg-gray-50 text-xs px-2 py-1 rounded text-gray-900 break-all">{check.txHash}</code></div>}
                      <div className="text-sm space-y-2">
                        <div className="flex justify-between"><span className="text-gray-600">链状态</span><span className={`status-tag text-xs ${CHAIN_STATUS_CONFIG[check.chainStatus].color}`}>{CHAIN_STATUS_CONFIG[check.chainStatus].label}</span></div>
                        <div className="flex justify-between"><span className="text-gray-600">投影状态</span><span className={`status-tag text-xs ${PROJECTION_STATUS_CONFIG[check.projectionStatus].color}`}>{PROJECTION_STATUS_CONFIG[check.projectionStatus].label}</span></div>
                      </div>
                      {!check.consistent && check.inconsistencyType && <div className="rounded-lg border border-red-200 bg-red-50 p-3"><div className="text-xs font-medium text-red-900 mb-1">{INCONSISTENCY_TYPE_CONFIG[check.inconsistencyType]?.label || check.inconsistencyType}</div><div className="text-xs text-red-700">{INCONSISTENCY_TYPE_CONFIG[check.inconsistencyType]?.description || '数据不一致'}</div></div>}
                    </div>
                    <div className="space-y-2">
                      <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium">重新检查</button>
                      {!check.consistent && <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><Shield className="w-4 h-4" /><span>尝试修复</span></button>}
                      <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><Link2 className="w-4 h-4" /><span>查看链路详情</span></button>
                    </div>
                  </div>
                )}
              />
            </section>
          ))}
        </>}
        pagination={<PaginationBar page={page} pageSize={pageSize} total={filteredChecks.length} onPageChange={setPage} onPageSizeChange={setPageSize} />}
      />
    </>
  )
}
