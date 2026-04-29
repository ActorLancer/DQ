import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'

const RULES = [
  { id: 'ar1', name: '一致性异常强提醒', source: 'system_internal', priority: '最高', action: '站内+短信+邮件', status: '启用' },
  { id: 'ar2', name: '链失败重试告警', source: 'service_external', priority: '最高', action: '站内+值班群Webhook', status: '启用' },
  { id: 'ar3', name: '通知重放失败', source: 'platform_internal', priority: '高', action: '站内+邮件', status: '启用' },
]

export default function AdminNotificationRulesPage() {
  return (
    <div className="p-8 space-y-6">
      <section className="rounded-2xl border border-gray-200 bg-white p-6">
        <Link href="/admin/console/notifications" className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900"><ArrowLeft className="w-4 h-4" />返回通知中心</Link>
        <h1 className="mt-2 text-3xl font-bold text-gray-900">平台通知规则</h1>
        <p className="mt-1 text-gray-600">统一管控风险、审计、链路与跨系统事件触发。</p>
      </section>
      <section className="rounded-2xl border border-gray-200 bg-white p-5 overflow-x-auto">
        <table className="w-full min-w-[760px] text-sm">
          <thead><tr className="text-left text-gray-500 border-b border-gray-200"><th className="py-3 px-3">规则名</th><th className="py-3 px-3">来源</th><th className="py-3 px-3">优先级</th><th className="py-3 px-3">动作</th><th className="py-3 px-3">状态</th><th className="py-3 px-3">操作</th></tr></thead>
          <tbody>{RULES.map((r) => <tr key={r.id} className="border-b border-gray-100"><td className="py-3 px-3 font-medium text-gray-900">{r.name}</td><td className="py-3 px-3">{r.source}</td><td className="py-3 px-3">{r.priority}</td><td className="py-3 px-3">{r.action}</td><td className="py-3 px-3">{r.status}</td><td className="py-3 px-3"><button className="text-primary-700">编辑</button></td></tr>)}</tbody>
        </table>
      </section>
    </div>
  )
}
