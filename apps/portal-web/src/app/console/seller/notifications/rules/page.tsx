import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'

const RULES = [
  { id: 'sr1', name: '链上失败告警', source: 'service_external', priority: '高', action: '站内+短信', status: '启用' },
  { id: 'sr2', name: '买家补件提醒', source: 'buyer_message', priority: '中', action: '站内', status: '启用' },
  { id: 'sr3', name: '调用阈值预警', source: 'system_internal', priority: '高', action: '站内+邮件', status: '启用' },
]

export default function SellerNotificationRulesPage() {
  return (
    <div className="p-8 space-y-6">
      <section className="rounded-2xl border border-gray-200 bg-white p-6">
        <Link href="/console/seller/notifications" className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900"><ArrowLeft className="w-4 h-4" />返回通知中心</Link>
        <h1 className="mt-2 text-3xl font-bold text-gray-900">供应商通知规则</h1>
        <p className="mt-1 text-gray-600">聚焦交付、链路与客户沟通告警策略。</p>
      </section>
      <section className="rounded-2xl border border-gray-200 bg-white p-5 overflow-x-auto">
        <table className="w-full min-w-[720px] text-sm">
          <thead><tr className="text-left text-gray-500 border-b border-gray-200"><th className="py-3 px-3">规则名</th><th className="py-3 px-3">来源</th><th className="py-3 px-3">优先级</th><th className="py-3 px-3">触达动作</th><th className="py-3 px-3">状态</th><th className="py-3 px-3">操作</th></tr></thead>
          <tbody>{RULES.map((r) => <tr key={r.id} className="border-b border-gray-100"><td className="py-3 px-3 font-medium text-gray-900">{r.name}</td><td className="py-3 px-3">{r.source}</td><td className="py-3 px-3">{r.priority}</td><td className="py-3 px-3">{r.action}</td><td className="py-3 px-3">{r.status}</td><td className="py-3 px-3"><button className="text-primary-700">编辑</button></td></tr>)}</tbody>
        </table>
      </section>
    </div>
  )
}
