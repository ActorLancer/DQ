import Link from 'next/link'
import { ArrowLeft, BellRing, ShieldCheck } from 'lucide-react'

const BUYER_RULES = [
  { id: 'br1', name: '订阅到期预警', source: 'platform_internal', condition: '到期 <= 30 天', action: '站内通知 + 邮件', priority: '高', status: '启用' },
  { id: 'br2', name: '申请补件提醒', source: 'service_external', condition: '状态=NEED_MORE_INFO', action: '站内通知', priority: '高', status: '启用' },
  { id: 'br3', name: '账单回执', source: 'system_internal', condition: '发票开具完成', action: '站内通知', priority: '中', status: '启用' },
]

export default function BuyerNotificationRulesPage() {
  return (
    <div className="p-8 space-y-6">
      <section className="rounded-2xl border border-gray-200 bg-white p-6">
        <div className="flex items-center justify-between gap-3">
          <div>
            <Link href="/console/buyer/notifications" className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900"><ArrowLeft className="w-4 h-4" />返回通知中心</Link>
            <h1 className="mt-2 text-3xl font-bold text-gray-900">买家通知规则</h1>
            <p className="mt-1 text-gray-600">管理消息来源、优先级与触达方式（前端框架模式）。</p>
          </div>
          <button className="h-11 rounded-lg bg-primary-600 px-4 text-sm font-medium text-white hover:bg-primary-700">新建规则</button>
        </div>
      </section>

      <section className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="rounded-2xl border border-blue-200 bg-blue-50 p-4"><p className="text-sm text-blue-900">启用规则</p><p className="text-3xl font-bold text-blue-900 mt-2">{BUYER_RULES.length}</p></div>
        <div className="rounded-2xl border border-amber-200 bg-amber-50 p-4"><p className="text-sm text-amber-900">高优先级</p><p className="text-3xl font-bold text-amber-900 mt-2">2</p></div>
        <div className="rounded-2xl border border-emerald-200 bg-emerald-50 p-4"><p className="text-sm text-emerald-900">静默时段</p><p className="text-2xl font-bold text-emerald-900 mt-2">23:00 - 07:00</p></div>
      </section>

      <section className="rounded-2xl border border-gray-200 bg-white p-5">
        <div className="overflow-x-auto">
          <table className="w-full min-w-[860px] text-sm">
            <thead><tr className="text-left text-gray-500 border-b border-gray-200"><th className="py-3 px-3">规则名</th><th className="py-3 px-3">来源</th><th className="py-3 px-3">触发条件</th><th className="py-3 px-3">动作</th><th className="py-3 px-3">优先级</th><th className="py-3 px-3">状态</th><th className="py-3 px-3">操作</th></tr></thead>
            <tbody>
              {BUYER_RULES.map((rule) => (
                <tr key={rule.id} className="border-b border-gray-100">
                  <td className="py-3 px-3 font-medium text-gray-900">{rule.name}</td>
                  <td className="py-3 px-3 text-gray-700">{rule.source}</td>
                  <td className="py-3 px-3 text-gray-700">{rule.condition}</td>
                  <td className="py-3 px-3 text-gray-700">{rule.action}</td>
                  <td className="py-3 px-3"><span className="rounded-full border border-amber-300 bg-amber-100 px-2 py-0.5 text-xs text-amber-700">{rule.priority}</span></td>
                  <td className="py-3 px-3"><span className="rounded-full border border-emerald-300 bg-emerald-100 px-2 py-0.5 text-xs text-emerald-700">{rule.status}</span></td>
                  <td className="py-3 px-3"><button className="mr-2 text-primary-700 hover:text-primary-800">编辑</button><button className="text-gray-600 hover:text-gray-800">停用</button></td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="rounded-2xl border border-gray-200 bg-white p-5"><div className="inline-flex items-center gap-2 text-gray-900 font-semibold"><BellRing className="w-4 h-4" />通知通道</div><p className="text-sm text-gray-600 mt-2">站内消息、邮件、Webhook（后续接入）</p></div>
        <div className="rounded-2xl border border-gray-200 bg-white p-5"><div className="inline-flex items-center gap-2 text-gray-900 font-semibold"><ShieldCheck className="w-4 h-4" />策略说明</div><p className="text-sm text-gray-600 mt-2">高风险消息强制触达，低优先级遵循静默策略。</p></div>
      </section>
    </div>
  )
}
