'use client'

import Link from 'next/link'
import { useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { ADMIN_AUDIT_ITEMS } from '@/lib/admin-audit-data'
import { updateAuditState, useAuditUiState } from '@/lib/admin-audit-state'
import { ArrowLeft, CheckCircle2, History, RefreshCcw, RotateCcw, ShieldAlert } from 'lucide-react'

type AuditTimelineType = 'REPLAY' | 'MARK_PROCESSED' | 'UNMARK_PROCESSED'

export default function AdminAuditDetailPage({ params }: { params: { auditId: string } }) {
  const router = useRouter()
  const item = useMemo(() => ADMIN_AUDIT_ITEMS.find((x) => x.id === params.auditId), [params.auditId])
  const uiState = useAuditUiState(params.auditId)
  const [toast, setToast] = useState<string | null>(null)
  const [replaying, setReplaying] = useState(false)
  const [processing, setProcessing] = useState(false)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!item) {
    return (
      <div className="p-10">
        <p className="text-gray-700 mb-4">审计事件不存在或已被移除。</p>
        <button onClick={() => router.push('/admin/console/audit')} className="h-10 px-4 border border-gray-300 rounded-lg hover:bg-gray-50">
          返回风险审计列表
        </button>
      </div>
    )
  }

  const pushToast = (message: string) => {
    setToast(message)
    window.setTimeout(() => setToast(null), 1800)
  }

  const handleReplay = async () => {
    const ok = window.confirm('确认重放检查该审计事件？将触发一次前端模拟重放。')
    if (!ok) return
    setReplaying(true)
    await new Promise((resolve) => setTimeout(resolve, 700))
    const nextRow: { type: AuditTimelineType; at: string; note: string } = {
      type: 'REPLAY',
      at: new Date().toISOString(),
      note: '管理员手动触发了重放检查',
    }
    updateAuditState(item.id, (prev) => ({
      ...prev,
      replayCount: (prev.replayCount ?? 0) + 1,
      lastActionAt: new Date().toISOString(),
      timeline: [nextRow, ...(prev.timeline ?? [])].slice(0, 20),
    }))
    setReplaying(false)
    pushToast('重放检查已完成')
  }

  const handleMarkProcessed = async () => {
    if (uiState.processed) {
      pushToast('该事件已处置')
      return
    }
    const ok = window.confirm('确认标记该事件为“已处理”？')
    if (!ok) return
    setProcessing(true)
    await new Promise((resolve) => setTimeout(resolve, 650))
    const nextRow: { type: AuditTimelineType; at: string; note: string } = {
      type: 'MARK_PROCESSED',
      at: new Date().toISOString(),
      note: '管理员将该事件标记为已处理',
    }
    updateAuditState(item.id, (prev) => ({
      ...prev,
      processed: true,
      lastActionAt: new Date().toISOString(),
      timeline: [nextRow, ...(prev.timeline ?? [])].slice(0, 20),
    }))
    setProcessing(false)
    pushToast('已标记为已处理，列表状态已同步')
  }

  const handleUnmarkProcessed = async () => {
    if (!uiState.processed) {
      pushToast('当前为待处理状态')
      return
    }
    const ok = window.confirm('确认撤销“已处理”并恢复为“待处理”？')
    if (!ok) return
    setProcessing(true)
    await new Promise((resolve) => setTimeout(resolve, 500))
    const nextRow: { type: AuditTimelineType; at: string; note: string } = {
      type: 'UNMARK_PROCESSED',
      at: new Date().toISOString(),
      note: '管理员撤销了已处理状态',
    }
    updateAuditState(item.id, (prev) => ({
      ...prev,
      processed: false,
      lastActionAt: new Date().toISOString(),
      timeline: [nextRow, ...(prev.timeline ?? [])].slice(0, 20),
    }))
    setProcessing(false)
    pushToast('已恢复为待处理，列表状态已同步')
  }

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

      <div className="p-8 space-y-6">
        <div className="flex items-center justify-between gap-4">
          <div>
            <Link href="/admin/console/audit" className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900 mb-3">
              <ArrowLeft className="w-4 h-4" />
              返回风险审计列表
            </Link>
            <h1 className="text-3xl font-bold text-gray-900">审计事件详情</h1>
            <p className="text-gray-600 mt-2">查看事件链路、操作上下文和处置记录。</p>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={handleReplay}
              disabled={replaying}
              className="h-10 px-4 border border-gray-300 rounded-lg text-sm font-medium hover:bg-gray-50 inline-flex items-center gap-2 disabled:opacity-50"
            >
              <RefreshCcw className={`w-4 h-4 ${replaying ? 'animate-spin' : ''}`} />
              {replaying ? '重放中...' : '重放检查'}
            </button>
            <button
              onClick={handleMarkProcessed}
              disabled={processing || uiState.processed}
              className="h-10 px-4 bg-primary-600 text-white rounded-lg text-sm font-medium hover:bg-primary-700 inline-flex items-center gap-2 disabled:opacity-50"
            >
              <CheckCircle2 className="w-4 h-4" />
              {uiState.processed ? '已处理' : processing ? '处理中...' : '标记已处理'}
            </button>
            <button
              onClick={handleUnmarkProcessed}
              disabled={processing || !uiState.processed}
              className="h-10 px-4 border border-gray-300 rounded-lg text-sm font-medium hover:bg-gray-50 inline-flex items-center gap-2 disabled:opacity-50"
            >
              <RotateCcw className="w-4 h-4" />
              解除已处理
            </button>
          </div>
        </div>

        <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
          <section className="xl:col-span-2 bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-5">事件主体</h2>
            <dl className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">消息 ID</dt><dd className="font-mono text-gray-900">{item.id}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">Request ID</dt><dd className="font-mono text-gray-900 break-all">{item.requestId}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">操作动作</dt><dd className="text-gray-900 font-semibold">{item.action}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">执行人</dt><dd className="text-gray-900">{item.actor}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">关联资源</dt><dd className="text-gray-900">{item.resourceType}/{item.resourceId}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">来源 IP</dt><dd className="text-gray-900">{item.ip}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">风险级别</dt><dd className="text-gray-900">{item.severity}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">执行结果</dt><dd className="text-gray-900">{item.result}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">处置状态</dt><dd className="text-gray-900">{uiState.processed ? '已处理' : '待处理'}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4"><dt className="text-gray-500 mb-1">重放次数</dt><dd className="text-gray-900">{uiState.replayCount}</dd></div>
              <div className="rounded-lg border border-gray-200 p-4 md:col-span-2"><dt className="text-gray-500 mb-1">发生时间</dt><dd className="text-gray-900">{item.createdAt}</dd></div>
            </dl>
          </section>

          <section className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-5">处置建议</h2>
            <div className="rounded-lg border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900 inline-flex gap-2">
              <ShieldAlert className="w-4 h-4 mt-0.5 shrink-0" />
              <span>建议按 request_id 联查通知、投影状态与链上凭证，确认是否存在跨系统回执延迟。</span>
            </div>
            <div className="mt-4 space-y-3 text-sm text-gray-700">
              <p>1. 检查该事件的前后 10 分钟内是否存在同类型失败。</p>
              <p>2. 若为审批类失败，建议先冻结相关商品状态，避免扩大影响。</p>
              <p>3. 处置完成后，在事件通知中心补充处理备注。</p>
            </div>
          </section>
        </div>

        <section className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-xl font-bold text-gray-900 mb-4">事件说明</h2>
          <p className="text-gray-700 leading-7">{item.detail}</p>
        </section>

        <section className="bg-white rounded-xl border border-gray-200 p-6">
          <h2 className="text-xl font-bold text-gray-900 mb-4 inline-flex items-center gap-2">
            <History className="w-5 h-5" />
            操作时间线
          </h2>
          {(uiState.timeline?.length ?? 0) === 0 ? (
            <p className="text-sm text-gray-500">暂无操作记录</p>
          ) : (
            <div className="space-y-3">
              {(uiState.timeline ?? []).map((row, idx) => (
                <div key={`${row.at}-${idx}`} className="rounded-lg border border-gray-200 p-4">
                  <div className="text-sm font-semibold text-gray-900">{row.type}</div>
                  <div className="text-xs text-gray-500 mt-1">{new Date(row.at).toLocaleString('zh-CN')}</div>
                  <div className="text-sm text-gray-700 mt-2">{row.note}</div>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>

      {toast ? (
        <div className="fixed right-6 top-24 z-50 rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-800 shadow-lg">
          {toast}
        </div>
      ) : null}
    </>
  )
}
