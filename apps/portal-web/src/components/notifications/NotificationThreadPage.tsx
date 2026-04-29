'use client'

import Link from 'next/link'
import { useMemo, useState } from 'react'
import { ArrowLeft, ExternalLink, MessageCircle, Send, ShieldCheck } from 'lucide-react'
import type { NotificationItem } from '@/types/notifications'

type Role = 'buyer' | 'seller' | 'admin'

const roleTitle: Record<Role, string> = {
  buyer: '买家',
  seller: '供应商',
  admin: '平台',
}

interface Props {
  role: Role
  notification: NotificationItem
  backPath: string
}

export default function NotificationThreadPage({ role, notification, backPath }: Props) {
  const [draft, setDraft] = useState('')
  const [localReplies, setLocalReplies] = useState(notification.replies)
  const [status, setStatus] = useState<'pending' | 'processing' | 'resolved'>('pending')

  const summary = useMemo(() => {
    if (status === 'resolved') return '已处理'
    if (status === 'processing') return '处理中'
    return '待处理'
  }, [status])

  return (
    <div className="p-8 space-y-6">
      <section className="rounded-2xl border border-gray-200 bg-white p-6">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <Link href={backPath} className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900"><ArrowLeft className="h-4 w-4" />返回通知中心</Link>
            <h1 className="mt-2 text-3xl font-bold text-gray-900">通知会话详情</h1>
            <p className="mt-1 text-gray-600">{roleTitle[role]}端消息处理线程（前端模拟）。</p>
          </div>
          <div className="rounded-xl border border-gray-200 bg-gray-50 px-3 py-2 text-sm text-gray-700">状态：<span className="font-semibold text-gray-900">{summary}</span></div>
        </div>
      </section>

      <section className="rounded-2xl border border-gray-200 bg-white p-6 space-y-4">
        <div>
          <h2 className="text-xl font-bold text-gray-900">{notification.title}</h2>
          <p className="mt-2 text-sm text-gray-700">{notification.detail}</p>
        </div>
        {role === 'admin' && (
          <details className="rounded-xl border border-gray-200 bg-gray-50 p-3 text-sm text-gray-700">
            <summary className="cursor-pointer font-semibold text-gray-900">技术详情（平台联查）</summary>
            <div className="mt-3 grid grid-cols-1 gap-3 md:grid-cols-2">
              <div className="rounded-lg border border-gray-200 bg-white p-3">消息ID：<span className="font-semibold text-gray-900">{notification.id}</span></div>
              <div className="rounded-lg border border-gray-200 bg-white p-3">优先级：<span className="font-semibold text-gray-900">P{notification.priority}</span></div>
              {notification.requestId && <div className="rounded-lg border border-gray-200 bg-white p-3">request_id：<span className="font-semibold text-gray-900">{notification.requestId}</span></div>}
              {notification.txHash && <div className="rounded-lg border border-gray-200 bg-white p-3">tx_hash：<span className="font-semibold text-gray-900">{notification.txHash}</span></div>}
              {notification.relatedEntityType && notification.relatedEntityId && <div className="rounded-lg border border-gray-200 bg-white p-3">关联对象：<span className="font-semibold text-gray-900">{notification.relatedEntityType}:{notification.relatedEntityId}</span></div>}
            </div>
          </details>
        )}

        <div className="flex flex-wrap gap-2">
          <button onClick={() => setStatus('processing')} className="h-10 rounded-lg border border-gray-300 px-3 text-sm hover:bg-gray-50">标记处理中</button>
          <button onClick={() => setStatus('resolved')} className="h-10 rounded-lg bg-emerald-600 px-3 text-sm font-medium text-white hover:bg-emerald-700 inline-flex items-center gap-1"><ShieldCheck className="h-4 w-4" />标记已处理</button>
          <Link href={notification.targetPath} className="h-10 rounded-lg bg-gray-900 px-3 text-sm font-medium text-white hover:bg-gray-800 inline-flex items-center gap-1">跳转业务页<ExternalLink className="h-4 w-4" /></Link>
        </div>
      </section>

      <section className="rounded-2xl border border-gray-200 bg-white p-6">
        <h3 className="text-lg font-semibold text-gray-900 inline-flex items-center gap-2"><MessageCircle className="h-4 w-4" />处理会话</h3>
        <div className="mt-4 space-y-3">
          {localReplies.length === 0 && <p className="text-sm text-gray-500">暂无会话回复</p>}
          {localReplies.map((reply) => (
            <article key={reply.id} className="rounded-xl border border-gray-200 bg-gray-50 p-3">
              <div className="flex items-center justify-between text-xs text-gray-500"><span>{reply.author} · {reply.role}</span><span>{reply.createdAt}</span></div>
              <p className="mt-1 text-sm text-gray-800">{reply.content}</p>
            </article>
          ))}
        </div>

        <div className="mt-4 rounded-xl border border-gray-200 p-3">
          <p className="text-xs text-gray-500 mb-2">支持 @平台审核员 / @供应商运营 / @买家管理员（前端占位）</p>
          <div className="flex gap-2">
            <input value={draft} onChange={(e) => setDraft(e.target.value)} placeholder="输入回复内容" className="h-10 flex-1 rounded-lg border border-gray-300 px-3 text-sm" />
            <button
              onClick={() => {
                const text = draft.trim()
                if (!text) return
                setLocalReplies((prev) => [...prev, { id: `local_${Date.now()}`, author: `${roleTitle[role]}端处理人`, role, content: text, createdAt: '刚刚' }])
                setDraft('')
              }}
              className="h-10 rounded-lg bg-primary-600 px-3 text-sm font-medium text-white hover:bg-primary-700 inline-flex items-center gap-1"
            >
              <Send className="h-4 w-4" />发送
            </button>
          </div>
        </div>
      </section>
    </div>
  )
}
