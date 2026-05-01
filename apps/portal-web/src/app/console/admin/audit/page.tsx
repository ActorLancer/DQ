'use client'

import { useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import InlineExpandableList from '@/components/console/InlineExpandableList'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import { ADMIN_AUDIT_ITEMS, type AdminAuditItem, type AuditResult, type AuditSeverity } from '@/lib/admin-audit-data'
import { useAdminAuditStateMap } from '@/lib/admin-audit-state'
import {
  AlertTriangle,
  ArrowUpDown,
  Eye,
  Filter,
  Layers,
  ShieldAlert,
  ShieldCheck,
  ShieldX,
} from 'lucide-react'

type GroupBy = 'none' | 'severity' | 'result'
type SortBy = 'latest' | 'severity_desc'

const severityRank: Record<AuditSeverity, number> = { LOW: 1, MEDIUM: 2, HIGH: 3 }

const severityStyle: Record<AuditSeverity, string> = {
  LOW: 'bg-green-100 text-green-700',
  MEDIUM: 'bg-amber-100 text-amber-700',
  HIGH: 'bg-red-100 text-red-700',
}

const resultStyle: Record<AuditResult, string> = {
  SUCCESS: 'bg-green-100 text-green-700',
  WARN: 'bg-amber-100 text-amber-700',
  FAILED: 'bg-red-100 text-red-700',
}

const resultLabel: Record<AuditResult, string> = {
  SUCCESS: '成功',
  WARN: '告警',
  FAILED: '失败',
}

export default function AdminAuditPage() {
  const router = useRouter()
  const auditStateMap = useAdminAuditStateMap()
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedSeverity, setSelectedSeverity] = useState<'all' | AuditSeverity>('all')
  const [selectedResult, setSelectedResult] = useState<'all' | AuditResult>('all')
  const [selectedHandleStatus, setSelectedHandleStatus] = useState<'all' | 'processed' | 'pending'>('all')
  const [groupBy, setGroupBy] = useState<GroupBy>('none')
  const [sortBy, setSortBy] = useState<SortBy>('latest')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filtered = useMemo(() => {
    const key = searchKeyword.trim().toLowerCase()
    const list = ADMIN_AUDIT_ITEMS.filter((item) => {
      const matchesKeyword =
        !key ||
        item.requestId.toLowerCase().includes(key) ||
        item.action.toLowerCase().includes(key) ||
        item.actor.toLowerCase().includes(key) ||
        item.resourceId.toLowerCase().includes(key)

      const matchesSeverity = selectedSeverity === 'all' || item.severity === selectedSeverity
      const matchesResult = selectedResult === 'all' || item.result === selectedResult
      const processed = auditStateMap[item.id]?.processed ?? false
      const matchesHandleStatus =
        selectedHandleStatus === 'all' ||
        (selectedHandleStatus === 'processed' && processed) ||
        (selectedHandleStatus === 'pending' && !processed)
      return matchesKeyword && matchesSeverity && matchesResult && matchesHandleStatus
    })

    return [...list].sort((a, b) => {
      if (sortBy === 'severity_desc') {
        const rankDiff = severityRank[b.severity] - severityRank[a.severity]
        if (rankDiff !== 0) return rankDiff
      }
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    })
  }, [searchKeyword, selectedSeverity, selectedResult, selectedHandleStatus, sortBy, auditStateMap])

  const paged = useMemo(() => filtered.slice((page - 1) * pageSize, page * pageSize), [filtered, page, pageSize])

  const grouped = useMemo(() => {
    if (groupBy === 'none') return [{ label: '全部审计事件', items: paged }]
    const map = new Map<string, AdminAuditItem[]>()
    for (const item of paged) {
      const label = groupBy === 'severity' ? `风险级别：${item.severity}` : `结果：${resultLabel[item.result]}`
      map.set(label, [...(map.get(label) || []), item])
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }))
  }, [groupBy, paged])

  return (
    <>
      <SessionIdentityBar
        subjectName="数据交易平台运营中心"
        roleName="平台管理员"
        tenantId="tenant_platform_001"
        scope="admin:audit:read"
        sessionExpiresAt={sessionExpiresAt}
        userName="管理员"
      />

      <ConsoleListPageShell
        title="风险审计"
        subtitle="联查关键操作、异常轨迹与风险信号，支持事件明细钻取。"
        summaryCards={
          <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="text-sm text-gray-500 mb-1">审计事件总数</div>
              <div className="text-3xl font-bold text-gray-900">{ADMIN_AUDIT_ITEMS.length}</div>
            </div>
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="text-sm text-gray-500 mb-1">高风险事件</div>
              <div className="text-3xl font-bold text-red-600">{ADMIN_AUDIT_ITEMS.filter((x) => x.severity === 'HIGH').length}</div>
            </div>
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="text-sm text-gray-500 mb-1">告警事件</div>
              <div className="text-3xl font-bold text-amber-600">{ADMIN_AUDIT_ITEMS.filter((x) => x.result === 'WARN').length}</div>
            </div>
            <div className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="text-sm text-gray-500 mb-1">失败事件</div>
              <div className="text-3xl font-bold text-gray-900">{ADMIN_AUDIT_ITEMS.filter((x) => x.result === 'FAILED').length}</div>
            </div>
          </div>
        }
        toolbar={
          <QueryToolbar
            searchValue={searchKeyword}
            onSearchChange={setSearchKeyword}
            searchPlaceholder="搜索 Request ID、操作、执行人、资源 ID..."
            onReset={() => {
              setSearchKeyword('')
              setSelectedSeverity('all')
              setSelectedResult('all')
              setSelectedHandleStatus('all')
              setGroupBy('none')
              setSortBy('latest')
              setPage(1)
              setPageSize(10)
            }}
            controls={
              <div className="grid grid-cols-1 md:grid-cols-6 gap-3">
                <select value={selectedSeverity} onChange={(e) => setSelectedSeverity(e.target.value as 'all' | AuditSeverity)} className="h-10 px-4 border border-gray-300 rounded-lg">
                  <option value="all">全部风险级别</option>
                  <option value="LOW">LOW</option>
                  <option value="MEDIUM">MEDIUM</option>
                  <option value="HIGH">HIGH</option>
                </select>
                <select value={selectedResult} onChange={(e) => setSelectedResult(e.target.value as 'all' | AuditResult)} className="h-10 px-4 border border-gray-300 rounded-lg">
                  <option value="all">全部执行结果</option>
                  <option value="SUCCESS">成功</option>
                  <option value="WARN">告警</option>
                  <option value="FAILED">失败</option>
                </select>
                <select value={selectedHandleStatus} onChange={(e) => setSelectedHandleStatus(e.target.value as 'all' | 'processed' | 'pending')} className="h-10 px-4 border border-gray-300 rounded-lg">
                  <option value="all">全部处置状态</option>
                  <option value="pending">待处理</option>
                  <option value="processed">已处理</option>
                </select>
                <select value={groupBy} onChange={(e) => setGroupBy(e.target.value as GroupBy)} className="h-10 px-4 border border-gray-300 rounded-lg">
                  <option value="none">不分组</option>
                  <option value="severity">按风险级别分组</option>
                  <option value="result">按执行结果分组</option>
                </select>
                <select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg">
                  <option value="latest">最新优先</option>
                  <option value="severity_desc">风险优先</option>
                </select>
                <select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg">
                  <option value={5}>分页 5 条</option>
                  <option value={10}>分页 10 条</option>
                  <option value={20}>分页 20 条</option>
                </select>
              </div>
            }
            stats={
              <>
                <span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filtered.length}</span>
                <span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分页 {pageSize}/页</span>
                <span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'latest' ? '最新优先' : '风险优先'}</span>
              </>
            }
          />
        }
        content={
          <div className="space-y-6">
            {grouped.map((group) => (
              <section key={group.label}>
                <div className="text-sm font-semibold text-gray-700 mb-3">{group.label}</div>
                <InlineExpandableList
                  items={group.items}
                  getKey={(item) => item.id}
                  selectedKey={selectedId}
                  onSelect={setSelectedId}
                  onOpenDetail={(item) => router.push(`/admin/console/audit/${item.id}`)}
                  renderSummary={(item, isSelected) => (
                    <div className="flex items-start justify-between gap-4">
                      <div className="min-w-0">
                        <div className="flex items-center gap-2 mb-2">
                          <span className={`status-tag ${severityStyle[item.severity]}`}>
                            {item.severity === 'HIGH' ? <ShieldX className="w-3.5 h-3.5" /> : item.severity === 'MEDIUM' ? <ShieldAlert className="w-3.5 h-3.5" /> : <ShieldCheck className="w-3.5 h-3.5" />}
                            <span>{item.severity}</span>
                          </span>
                          <span className={`status-tag ${resultStyle[item.result]}`}>{resultLabel[item.result]}</span>
                          <span className={`status-tag ${(auditStateMap[item.id]?.processed ?? false) ? 'bg-blue-100 text-blue-700' : 'bg-gray-100 text-gray-700'}`}>
                            {(auditStateMap[item.id]?.processed ?? false) ? '已处理' : '待处理'}
                          </span>
                        </div>
                        <p className="text-lg font-bold text-gray-900">{item.action}</p>
                        <p className="text-sm text-gray-600 mt-1">
                          执行人 {item.actor} · 资源 {item.resourceType}/{item.resourceId}
                        </p>
                      </div>
                      <div className="text-right shrink-0">
                        <div className="text-sm text-gray-500">{item.createdAt}</div>
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation()
                            router.push(`/admin/console/audit/${item.id}`)
                          }}
                          className={`mt-2 h-9 px-3 border rounded-lg text-sm inline-flex items-center gap-1 ${isSelected ? 'border-primary-300 text-primary-700' : 'border-gray-300 text-gray-700 hover:bg-gray-50'}`}
                        >
                          <Eye className="w-4 h-4" />
                          查看详情
                        </button>
                      </div>
                    </div>
                  )}
                  renderExpanded={(item) => (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                      <div className="rounded-lg border border-gray-200 bg-gray-50 p-3">
                        <div className="text-gray-500 mb-1">Request ID</div>
                        <div className="font-mono text-gray-900 break-all">{item.requestId}</div>
                      </div>
                      <div className="rounded-lg border border-gray-200 bg-gray-50 p-3">
                        <div className="text-gray-500 mb-1">来源 IP</div>
                        <div className="text-gray-900">{item.ip}</div>
                      </div>
                      <div className="rounded-lg border border-gray-200 bg-gray-50 p-3 md:col-span-2">
                        <div className="text-gray-500 mb-1 inline-flex items-center gap-1">
                          <AlertTriangle className="w-4 h-4 text-amber-500" />
                          事件摘要
                        </div>
                        <div className="text-gray-800 leading-6">{item.detail}</div>
                      </div>
                    </div>
                  )}
                />
              </section>
            ))}
          </div>
        }
        pagination={<PaginationBar page={page} pageSize={pageSize} total={filtered.length} onPageChange={setPage} onPageSizeChange={setPageSize} />}
      />
    </>
  )
}
