'use client'

import Link from 'next/link'
import { useMemo, useState, useEffect } from 'react'
import { useSearchParams } from 'next/navigation'
import { AnimatePresence, motion } from 'framer-motion'
import {
  Bell,
  CheckCheck,
  ExternalLink,
  Filter,
  MessageCircle,
  Pin,
  PinOff,
  Search,
  Trash2,
  X,
} from 'lucide-react'
import type { NotificationItem, NotificationLevel, NotificationSource, NotificationStatus } from '@/types/notifications'

type SortMode = 'newest' | 'priority_desc' | 'unread_first' | 'pinned_first'
type SourceFilter = 'all' | NotificationSource
type LevelFilter = 'all' | NotificationLevel
type StatusFilter = 'all' | NotificationStatus

type GroupedEntry =
  | { kind: 'header'; label: '今天' | '昨天' | '更早' }
  | { kind: 'item'; item: NotificationItem }

const sourceLabel: Record<NotificationSource, string> = {
  platform_internal: '平台内部',
  system_internal: '系统内部',
  service_external: '外部服务',
  buyer_message: '买家消息',
  seller_message: '卖家消息',
}

const levelLabel: Record<NotificationLevel, string> = {
  info: '信息',
  warning: '告警',
  error: '错误',
  success: '成功',
}

const levelTone: Record<NotificationLevel, string> = {
  info: 'border-l-sky-400 bg-sky-50/20',
  warning: 'border-l-amber-400 bg-amber-50/20',
  error: 'border-l-rose-400 bg-rose-50/20',
  success: 'border-l-emerald-400 bg-emerald-50/20',
}

const UI = {
  radiusPanel: 'rounded-2xl',
  radiusCard: 'rounded-xl',
  title: 'text-[30px] font-bold tracking-tight text-slate-900',
  sectionTitle: 'text-base font-semibold text-slate-900',
  body: 'text-sm leading-6 text-slate-600',
  meta: 'text-xs text-slate-400',
  buttonH: 'h-9',
  inputH: 'h-10',
}

interface NotificationCenterProps {
  role: 'buyer' | 'seller' | 'admin'
  title: string
  subtitle: string
  initialItems: NotificationItem[]
  rulesPath: string
}

const detailBasePath: Record<'buyer' | 'seller' | 'admin', string> = {
  buyer: '/console/buyer/notifications',
  seller: '/console/seller/notifications',
  admin: '/admin/console/notifications',
}

function getDateGroupLabel(iso: string): '今天' | '昨天' | '更早' {
  const d = new Date(iso)
  const now = new Date()
  const startToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime()
  const startYesterday = startToday - 24 * 60 * 60 * 1000
  const t = d.getTime()
  if (t >= startToday) return '今天'
  if (t >= startYesterday) return '昨天'
  return '更早'
}

export default function NotificationCenter({ role, title, subtitle, initialItems, rulesPath }: NotificationCenterProps) {
  const searchParams = useSearchParams()
  const [items, setItems] = useState<NotificationItem[]>(initialItems)
  const [keyword, setKeyword] = useState('')
  const [source, setSource] = useState<SourceFilter>('all')
  const [level, setLevel] = useState<LevelFilter>('all')
  const [status, setStatus] = useState<StatusFilter>('all')
  const [sort, setSort] = useState<SortMode>('newest')
  const [currentPage, setCurrentPage] = useState(1)
  const [activeId, setActiveId] = useState<string | null>(null)
  const [selectedIds, setSelectedIds] = useState<string[]>([])
  const [batchMode, setBatchMode] = useState(false)
  const [toast, setToast] = useState<string | null>(null)
  const [highlightId, setHighlightId] = useState<string | null>(null)
  const [replyDraft, setReplyDraft] = useState('')

  const PAGE_SIZE = 8

  const filteredSorted = useMemo(() => {
    const list = items.filter((item) => {
      const t = `${item.title} ${item.desc} ${item.detail}`.toLowerCase()
      return (
        t.includes(keyword.toLowerCase()) &&
        (source === 'all' || item.source === source) &&
        (level === 'all' || item.level === level) &&
        (status === 'all' || item.status === status)
      )
    })

    return [...list].sort((a, b) => {
      if (sort === 'pinned_first' && a.pinned !== b.pinned) return a.pinned ? -1 : 1
      if (sort === 'unread_first' && a.status !== b.status) return a.status === 'unread' ? -1 : 1
      if (sort === 'priority_desc' && a.priority !== b.priority) return b.priority - a.priority
      return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    })
  }, [items, keyword, source, level, status, sort])

  const pageCount = Math.max(1, Math.ceil(filteredSorted.length / PAGE_SIZE))
  const pagedItems = useMemo(() => {
    const start = (currentPage - 1) * PAGE_SIZE
    return filteredSorted.slice(start, start + PAGE_SIZE)
  }, [filteredSorted, currentPage])

  const groupedPagedEntries = useMemo<GroupedEntry[]>(() => {
    const entries: GroupedEntry[] = []
    let lastLabel: '今天' | '昨天' | '更早' | null = null
    for (const item of pagedItems) {
      const label = getDateGroupLabel(item.timestamp)
      if (label !== lastLabel) {
        entries.push({ kind: 'header', label })
        lastLabel = label
      }
      entries.push({ kind: 'item', item })
    }
    return entries
  }, [pagedItems])

  const activeItem = useMemo(() => items.find((x) => x.id === activeId) || null, [items, activeId])

  useEffect(() => {
    if (currentPage > pageCount) setCurrentPage(1)
  }, [currentPage, pageCount])

  useEffect(() => {
    const id = searchParams.get('focus')
    if (!id) return
    if (!items.some((x) => x.id === id)) return
    setActiveId(id)
    setHighlightId(id)
    const idx = filteredSorted.findIndex((x) => x.id === id)
    if (idx >= 0) setCurrentPage(Math.floor(idx / PAGE_SIZE) + 1)
    const timer = window.setTimeout(() => {
      const el = document.getElementById(`notice-${id}`)
      if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' })
      setHighlightId((old) => (old === id ? null : old))
    }, 140)
    return () => window.clearTimeout(timer)
  }, [searchParams, items, filteredSorted])

  useEffect(() => {
    if (!toast) return
    const timer = window.setTimeout(() => setToast(null), 1800)
    return () => window.clearTimeout(timer)
  }, [toast])

  const markRead = (id: string) => setItems((prev) => prev.map((x) => (x.id === id ? { ...x, status: 'read' } : x)))

  const togglePin = (id: string) => {
    setItems((prev) => prev.map((x) => (x.id === id ? { ...x, pinned: !x.pinned } : x)))
    setToast('已更新置顶状态')
  }

  const deleteOne = (id: string) => {
    if (!window.confirm('确认删除该消息？')) return
    setItems((prev) => prev.filter((x) => x.id !== id))
    setSelectedIds((prev) => prev.filter((x) => x !== id))
    if (activeId === id) setActiveId(null)
    setToast('消息已删除')
  }

  const toggleSelect = (id: string, checked: boolean) => setSelectedIds((prev) => (checked ? Array.from(new Set([...prev, id])) : prev.filter((x) => x !== id)))

  const selectPage = (checked: boolean) => {
    const ids = pagedItems.map((x) => x.id)
    setSelectedIds((prev) => (checked ? Array.from(new Set([...prev, ...ids])) : prev.filter((x) => !ids.includes(x))))
  }

  const allPageSelected = pagedItems.length > 0 && pagedItems.every((x) => selectedIds.includes(x.id))

  const batchAction = (action: 'read' | 'archive' | 'delete') => {
    if (selectedIds.length === 0) return
    if (action === 'delete' && !window.confirm(`确认删除已选 ${selectedIds.length} 条消息？`)) return
    setItems((prev) => {
      if (action === 'delete') return prev.filter((x) => !selectedIds.includes(x.id))
      if (action === 'archive') return prev.map((x) => (selectedIds.includes(x.id) ? { ...x, status: 'archived' } : x))
      return prev.map((x) => (selectedIds.includes(x.id) ? { ...x, status: 'read' } : x))
    })
    setSelectedIds([])
    setBatchMode(false)
    setToast(action === 'delete' ? '批量删除完成' : action === 'archive' ? '已归档所选消息' : '已标记已读')
  }

  const submitReply = () => {
    if (!activeItem) return
    const content = replyDraft.trim()
    if (!content) return
    setItems((prev) => prev.map((x) => (x.id === activeItem.id
      ? {
          ...x,
          status: 'read',
          replies: [...x.replies, { id: `reply_${Date.now()}`, author: role === 'admin' ? '平台管理员' : role === 'seller' ? '供应商运营' : '买家运营', role, content, createdAt: '刚刚' }],
        }
      : x)))
    setReplyDraft('')
    setToast('回复已发送（前端模拟）')
  }

  const unreadCount = items.filter((x) => x.status === 'unread').length
  const pinnedCount = items.filter((x) => x.pinned).length

  return (
    <div className="p-8 space-y-6">
      <motion.section
        initial={{ opacity: 0, y: 6 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.25, ease: 'easeOut' }}
        className="relative overflow-hidden rounded-[22px] border border-slate-200 bg-gradient-to-br from-slate-50 via-white to-slate-100 px-7 py-6"
      >
        <div className="absolute inset-0 opacity-[0.06]" style={{ backgroundImage: 'radial-gradient(#64748b .6px, transparent .6px)', backgroundSize: '10px 10px' }} />
        <div className="relative flex flex-wrap items-end justify-between gap-4">
          <div>
            <h1 className={UI.title}>{title}</h1>
            <p className="mt-2 text-[15px] leading-6 text-slate-600">{subtitle}</p>
          </div>
          <div className="inline-flex items-center gap-2 rounded-xl border border-slate-200 bg-white/80 px-4 py-2 text-sm text-slate-700 backdrop-blur">
            <Bell className="h-4 w-4" />
            未读 {unreadCount} · 置顶 {pinnedCount}
          </div>
        </div>
      </motion.section>

      <section className="grid grid-cols-12 gap-5">
        <motion.aside
          initial={{ opacity: 0, x: -8 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.22, ease: 'easeOut' }}
          className={`col-span-12 xl:col-span-3 ${UI.radiusPanel} border border-slate-200 bg-white p-4 shadow-[0_12px_36px_-28px_rgba(15,23,42,0.45)]`}
        >
          <h2 className={UI.sectionTitle}>筛选与视图</h2>
          <div className="mt-3 space-y-3">
            <div className="relative">
              <Search className="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-slate-400" />
              <input value={keyword} onChange={(e) => { setKeyword(e.target.value); setCurrentPage(1) }} placeholder="搜索通知" className={`${UI.inputH} w-full rounded-lg border border-slate-300 pl-9 pr-3 text-sm`} />
            </div>
            <select value={source} onChange={(e) => { setSource(e.target.value as SourceFilter); setCurrentPage(1) }} className={`${UI.inputH} w-full rounded-lg border border-slate-300 px-3 text-sm`}>
              <option value="all">全部来源</option>
              <option value="platform_internal">平台内部</option>
              <option value="system_internal">系统内部</option>
              <option value="service_external">外部服务</option>
              <option value="buyer_message">买家消息</option>
              <option value="seller_message">卖家消息</option>
            </select>
            <div className="grid grid-cols-2 gap-2">
              <button onClick={() => setLevel('warning')} className={`${UI.buttonH} rounded-lg border text-xs transition-all duration-200 ${level === 'warning' ? 'border-amber-300 bg-amber-50 text-amber-700 shadow-sm' : 'border-slate-300 text-slate-600 hover:bg-slate-50'}`}>仅告警</button>
              <button onClick={() => setStatus('unread')} className={`${UI.buttonH} rounded-lg border text-xs transition-all duration-200 ${status === 'unread' ? 'border-blue-300 bg-blue-50 text-blue-700 shadow-sm' : 'border-slate-300 text-slate-600 hover:bg-slate-50'}`}>仅未读</button>
            </div>
            <div className="grid grid-cols-2 gap-2">
              <button onClick={() => { setLevel('all'); setStatus('all') }} className={`${UI.buttonH} rounded-lg border border-slate-300 text-xs text-slate-600 hover:bg-slate-50`}>重置</button>
              <button onClick={() => setSort('priority_desc')} className={`${UI.buttonH} rounded-lg border text-xs transition-all duration-200 ${sort === 'priority_desc' ? 'border-slate-400 bg-slate-100 text-slate-800 shadow-sm' : 'border-slate-300 text-slate-600 hover:bg-slate-50'}`}>优先级排序</button>
            </div>
            <select value={sort} onChange={(e) => setSort(e.target.value as SortMode)} className={`${UI.inputH} w-full rounded-lg border border-slate-300 px-3 text-sm`}>
              <option value="newest">最新优先</option>
              <option value="unread_first">未读优先</option>
              <option value="pinned_first">置顶优先</option>
              <option value="priority_desc">优先级优先</option>
            </select>
            <Link href={rulesPath} className={`inline-flex ${UI.inputH} w-full items-center justify-center gap-2 rounded-lg border border-slate-300 text-sm text-slate-700 hover:bg-slate-50`}><Filter className="h-4 w-4" />通知规则</Link>
          </div>
        </motion.aside>

        <motion.main
          layout
          transition={{ duration: 0.26, ease: 'easeInOut' }}
          className={`col-span-12 ${activeItem ? 'xl:col-span-6' : 'xl:col-span-9'} space-y-3`}
        >
          <motion.div layout className={`${UI.radiusPanel} border border-slate-200 bg-white p-3 shadow-[0_12px_36px_-28px_rgba(15,23,42,0.45)]`}>
            <div className="flex flex-wrap items-center justify-between gap-2">
              <div className="inline-flex items-center gap-2">
                <button onClick={() => { setBatchMode((v) => !v); setSelectedIds([]) }} className={`${UI.buttonH} rounded-lg border px-3 text-sm transition-all duration-200 ${batchMode ? 'border-slate-500 bg-slate-800 text-white shadow-sm' : 'border-slate-300 text-slate-700 hover:bg-slate-50'}`}>
                  {batchMode ? '退出批处理' : '进入批处理'}
                </button>
                {batchMode && (
                  <label className="inline-flex items-center gap-2 text-sm text-slate-600">
                    <input type="checkbox" checked={allPageSelected} onChange={(e) => selectPage(e.target.checked)} className="h-4 w-4 rounded border-slate-300" />全选本页
                  </label>
                )}
              </div>
              <div className="text-sm text-slate-500">第 {currentPage}/{pageCount} 页 · 共 {filteredSorted.length} 条</div>
            </div>
          </motion.div>

          <AnimatePresence mode="popLayout">
            {groupedPagedEntries.map((entry, idx) => {
              if (entry.kind === 'header') {
                return (
                  <motion.div
                    layout
                    key={`header-${entry.label}-${idx}`}
                    initial={{ opacity: 0, y: 4 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -4 }}
                    transition={{ duration: 0.2, ease: 'easeOut' }}
                    className="sticky top-0 z-10 rounded-lg border border-slate-200 bg-slate-50/90 px-3 py-1.5 text-xs font-semibold tracking-wide text-slate-500 backdrop-blur"
                  >
                    {entry.label}
                  </motion.div>
                )
              }

              const item = entry.item
              return (
                <motion.article
                  id={`notice-${item.id}`}
                  key={item.id}
                  layout
                  initial={{ opacity: 0, y: 10, scale: 0.995 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: -8, scale: 0.995 }}
                  transition={{ duration: 0.22, ease: 'easeOut' }}
                  onClick={() => { setActiveId(item.id); markRead(item.id) }}
                  className={`group relative overflow-hidden cursor-pointer ${UI.radiusCard} border border-slate-200 border-l-[3px] bg-white p-4 shadow-[0_14px_34px_-30px_rgba(15,23,42,0.5)] transition-all duration-200 hover:-translate-y-[1px] hover:border-slate-300 hover:shadow-[0_20px_36px_-26px_rgba(15,23,42,0.46)] ${levelTone[item.level]} ${highlightId === item.id ? 'ring-2 ring-primary-500 ring-offset-1' : ''} ${activeId === item.id ? 'ring-2 ring-slate-300 ring-offset-1' : ''}`}
                >
                  <div className="pointer-events-none absolute inset-0 bg-gradient-to-r from-white/0 via-slate-100/20 to-white/0 opacity-0 transition-opacity duration-200 group-hover:opacity-100" />
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        {batchMode && (
                          <input type="checkbox" checked={selectedIds.includes(item.id)} onChange={(e) => { e.stopPropagation(); toggleSelect(item.id, e.target.checked) }} className="h-4 w-4 rounded border-slate-300" />
                        )}
                        <h3 className="truncate text-[15px] font-semibold text-slate-900">{item.title}</h3>
                        <span className="rounded-full border border-slate-200 bg-slate-50 px-2 py-0.5 text-[11px] text-slate-600">{sourceLabel[item.source]}</span>
                        <span className="rounded-full border border-slate-200 bg-white px-2 py-0.5 text-[11px] text-slate-500">{levelLabel[item.level]}</span>
                        {role === 'admin' && <span className="rounded-full border border-slate-300 bg-slate-100 px-2 py-0.5 text-[11px] text-slate-700">P{item.priority}</span>}
                        {item.status === 'unread' && <span className="rounded-full border border-indigo-200 bg-indigo-50 px-2 py-0.5 text-[11px] text-indigo-700">未读</span>}
                        {item.pinned && <span className="rounded-full border border-sky-200 bg-sky-50 px-2 py-0.5 text-[11px] text-sky-700">置顶</span>}
                      </div>
                      <p className={`mt-1 line-clamp-2 ${UI.body}`}>{item.desc}</p>
                      <p className={`mt-1 ${UI.meta}`}>{item.time}</p>
                    </div>

                    <div className="flex shrink-0 items-center gap-1 opacity-0 transition-opacity duration-200 group-hover:opacity-100">
                      <button onClick={(e) => { e.stopPropagation(); togglePin(item.id) }} className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-slate-300 text-slate-600 hover:bg-slate-50">{item.pinned ? <PinOff className="h-3.5 w-3.5" /> : <Pin className="h-3.5 w-3.5" />}</button>
                      <button onClick={(e) => { e.stopPropagation(); deleteOne(item.id) }} className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-red-200 text-red-700 hover:bg-red-50"><Trash2 className="h-3.5 w-3.5" /></button>
                      <Link onClick={(e) => e.stopPropagation()} href={item.targetPath} className="inline-flex h-8 w-8 items-center justify-center rounded-md bg-slate-900 text-white hover:bg-slate-800"><ExternalLink className="h-3.5 w-3.5" /></Link>
                    </div>
                  </div>
                </motion.article>
              )
            })}
          </AnimatePresence>

          {pagedItems.length === 0 && (
            <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} className="rounded-2xl border border-dashed border-slate-300 bg-white p-10 text-center text-sm text-slate-500">暂无匹配结果，建议清空筛选或更换关键词。</motion.div>
          )}

          <motion.div layout className={`${UI.radiusPanel} flex items-center justify-between border border-slate-200 bg-white p-3`}>
            <button disabled={currentPage === 1} onClick={() => setCurrentPage((p) => Math.max(1, p - 1))} className={`${UI.buttonH} rounded-lg border border-slate-300 px-3 text-sm disabled:opacity-40`}>上一页</button>
            <button disabled={currentPage === pageCount} onClick={() => setCurrentPage((p) => Math.min(pageCount, p + 1))} className={`${UI.buttonH} rounded-lg border border-slate-300 px-3 text-sm disabled:opacity-40`}>下一页</button>
          </motion.div>
        </motion.main>

        <AnimatePresence>
          {activeItem && (
            <motion.aside
              key="context-panel"
              initial={{ opacity: 0, x: 28, scale: 0.985 }}
              animate={{ opacity: 1, x: 0, scale: 1 }}
              exit={{ opacity: 0, x: 28, scale: 0.985 }}
              transition={{ duration: 0.26, ease: 'easeOut' }}
              className={`hidden xl:block col-span-12 xl:col-span-3 ${UI.radiusPanel} border border-slate-200 bg-white p-4 shadow-[0_18px_42px_-30px_rgba(15,23,42,0.55)]`}
            >
              <div className="mb-3 flex items-center justify-between">
                <h3 className={UI.sectionTitle}>上下文面板</h3>
                <button onClick={() => setActiveId(null)} className="rounded-md p-1 text-slate-500 hover:bg-slate-100"><X className="h-4 w-4" /></button>
              </div>

              <section className="space-y-2">
                <h4 className="text-[15px] font-semibold text-slate-900">{activeItem.title}</h4>
                <p className={UI.body}>{activeItem.detail}</p>
              </section>

              {role === 'admin' && (
                <details className="mt-3 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-700">
                  <summary className="cursor-pointer font-medium text-slate-800">技术详情</summary>
                  <div className="mt-2 space-y-1">
                    <p>消息ID: <span className="font-medium text-slate-900">{activeItem.id}</span></p>
                    <p>优先级: <span className="font-medium text-slate-900">P{activeItem.priority}</span></p>
                    {activeItem.requestId && <p>request_id: <span className="font-medium text-slate-900">{activeItem.requestId}</span></p>}
                    {activeItem.txHash && <p>tx_hash: <span className="font-medium text-slate-900">{activeItem.txHash}</span></p>}
                    {activeItem.relatedEntityType && activeItem.relatedEntityId && <p>关联对象: <span className="font-medium text-slate-900">{activeItem.relatedEntityType}:{activeItem.relatedEntityId}</span></p>}
                  </div>
                </details>
              )}

              <section className="mt-4 space-y-2">
                <Link href={`${detailBasePath[role]}/${activeItem.id}`} className={`inline-flex ${UI.inputH} w-full items-center justify-center rounded-lg border border-slate-300 text-sm font-medium text-slate-700 hover:bg-slate-50`}>会话详情</Link>
                <Link href={activeItem.targetPath} className={`inline-flex ${UI.inputH} w-full items-center justify-center rounded-lg bg-slate-900 text-sm font-medium text-white hover:bg-slate-800`}>跳转业务页</Link>
              </section>

              <section className="mt-4 rounded-xl border border-slate-200 bg-slate-50 p-3">
                <h5 className="inline-flex items-center gap-1 text-sm font-medium text-slate-900"><MessageCircle className="h-4 w-4" />快速回复</h5>
                <div className="mt-2 max-h-40 space-y-2 overflow-auto pr-1.5" style={{ scrollbarWidth: 'thin' }}>
                  {activeItem.replies.length === 0 && <p className="text-xs text-slate-500">暂无回复</p>}
                  {activeItem.replies.map((reply) => (
                    <div key={reply.id} className="rounded-lg border border-slate-200 bg-white p-2">
                      <div className="flex items-center justify-between text-[11px] text-slate-500"><span>{reply.author}</span><span>{reply.createdAt}</span></div>
                      <p className="mt-1 text-xs text-slate-700">{reply.content}</p>
                    </div>
                  ))}
                </div>
                <div className="mt-2 flex gap-2">
                  <input value={replyDraft} onChange={(e) => setReplyDraft(e.target.value)} placeholder="输入回复" className={`${UI.buttonH} flex-1 rounded-lg border border-slate-300 px-3 text-sm`} />
                  <button onClick={submitReply} className={`${UI.buttonH} rounded-lg bg-primary-600 px-3 text-sm font-medium text-white hover:bg-primary-700`}>发送</button>
                </div>
              </section>
            </motion.aside>
          )}
        </AnimatePresence>
      </section>

      <AnimatePresence>
        {activeItem && (
          <>
            <motion.div
              className="fixed inset-0 z-40 bg-slate-900/30 xl:hidden"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => setActiveId(null)}
            />
            <motion.section
              className="fixed inset-x-0 bottom-0 z-50 max-h-[85vh] rounded-t-3xl border border-slate-200 bg-white p-4 shadow-2xl xl:hidden"
              initial={{ y: '100%' }}
              animate={{ y: 0 }}
              exit={{ y: '100%' }}
              transition={{ duration: 0.28, ease: 'easeOut' }}
            >
              <div className="mx-auto mb-3 h-1.5 w-12 rounded-full bg-slate-300" />
              <div className="mb-2 flex items-center justify-between">
                <h3 className={UI.sectionTitle}>消息详情</h3>
                <button onClick={() => setActiveId(null)} className="rounded-md p-1 text-slate-500 hover:bg-slate-100"><X className="h-4 w-4" /></button>
              </div>

              <div className="space-y-3 overflow-auto pb-2" style={{ maxHeight: 'calc(85vh - 56px)', scrollbarWidth: 'thin' }}>
                <section className="space-y-2">
                  <h4 className="text-[15px] font-semibold text-slate-900">{activeItem.title}</h4>
                  <p className={UI.body}>{activeItem.detail}</p>
                </section>

                <section className="space-y-2">
                  <Link href={`${detailBasePath[role]}/${activeItem.id}`} className={`inline-flex ${UI.inputH} w-full items-center justify-center rounded-lg border border-slate-300 text-sm font-medium text-slate-700 hover:bg-slate-50`}>会话详情</Link>
                  <Link href={activeItem.targetPath} className={`inline-flex ${UI.inputH} w-full items-center justify-center rounded-lg bg-slate-900 text-sm font-medium text-white hover:bg-slate-800`}>跳转业务页</Link>
                </section>

                <section className="rounded-xl border border-slate-200 bg-slate-50 p-3">
                  <h5 className="inline-flex items-center gap-1 text-sm font-medium text-slate-900"><MessageCircle className="h-4 w-4" />快速回复</h5>
                  <div className="mt-2 max-h-36 space-y-2 overflow-auto pr-1" style={{ scrollbarWidth: 'thin' }}>
                    {activeItem.replies.length === 0 && <p className="text-xs text-slate-500">暂无回复</p>}
                    {activeItem.replies.map((reply) => (
                      <div key={reply.id} className="rounded-lg border border-slate-200 bg-white p-2">
                        <div className="flex items-center justify-between text-[11px] text-slate-500"><span>{reply.author}</span><span>{reply.createdAt}</span></div>
                        <p className="mt-1 text-xs text-slate-700">{reply.content}</p>
                      </div>
                    ))}
                  </div>
                  <div className="mt-2 flex gap-2">
                    <input value={replyDraft} onChange={(e) => setReplyDraft(e.target.value)} placeholder="输入回复" className={`${UI.buttonH} flex-1 rounded-lg border border-slate-300 px-3 text-sm`} />
                    <button onClick={submitReply} className={`${UI.buttonH} rounded-lg bg-primary-600 px-3 text-sm font-medium text-white hover:bg-primary-700`}>发送</button>
                  </div>
                </section>
              </div>
            </motion.section>
          </>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {batchMode && selectedIds.length > 0 && (
          <motion.div
            initial={{ opacity: 0, y: 14, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 14, scale: 0.98 }}
            transition={{ duration: 0.22, ease: 'easeOut' }}
            className="fixed bottom-5 left-1/2 z-50 -translate-x-1/2 rounded-2xl border border-slate-200 bg-white/95 px-4 py-3 shadow-xl backdrop-blur"
          >
            <div className="flex items-center gap-2 text-sm">
              <span className="text-slate-600">已选 {selectedIds.length} 条</span>
              <button onClick={() => batchAction('read')} className="inline-flex h-8 items-center gap-1 rounded-md border border-slate-300 px-2.5"><CheckCheck className="h-3.5 w-3.5" />已读</button>
              <button onClick={() => batchAction('archive')} className="h-8 rounded-md border border-slate-300 px-2.5">归档</button>
              <button onClick={() => batchAction('delete')} className="h-8 rounded-md border border-red-200 px-2.5 text-red-700">删除</button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      <AnimatePresence>
        {toast && (
          <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: 8 }} className="fixed bottom-5 right-5 z-50 rounded-lg border border-slate-200 bg-white px-4 py-2 text-sm text-slate-800 shadow-lg">
            {toast}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}
