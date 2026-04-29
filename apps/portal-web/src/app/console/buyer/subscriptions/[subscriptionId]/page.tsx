'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { MOCK_SUBSCRIPTIONS, SUB_STATUS_CONFIG } from '@/lib/buyer-subscriptions-data'
import { ArrowLeft, ExternalLink, KeyRound, Activity, Shield } from 'lucide-react'

export default function BuyerSubscriptionDetailPage() {
  const params = useParams<{ subscriptionId: string }>()
  const sub = MOCK_SUBSCRIPTIONS.find((item) => item.subscriptionId === params.subscriptionId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!sub) return <div className="p-8 text-lg font-bold">订阅不存在</div>

  return (
    <>
      <SessionIdentityBar subjectName="某某科技有限公司" roleName="买家管理员" tenantId="tenant_buyer_001" scope="buyer:subscriptions:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/buyer/subscriptions" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900"><ArrowLeft className="w-4 h-4" /><span>返回我的订阅</span></Link>
          <Link href="/console/buyer/api-keys" className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"><span>前往 API 密钥</span><ExternalLink className="w-3.5 h-3.5" /></Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <h1 className="text-3xl font-bold text-gray-900 mb-1">{sub.listingTitle}</h1>
            <p className="text-sm text-gray-600">订阅编号：{sub.subscriptionId}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-5">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">状态</div><div className={`status-tag ${SUB_STATUS_CONFIG[sub.status].color}`}>{SUB_STATUS_CONFIG[sub.status].label}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">供应商</div><div className="font-semibold text-gray-900">{sub.supplierName}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">套餐</div><div className="font-semibold text-gray-900">{sub.plan}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">授权区间</div><div className="font-semibold text-gray-900">{sub.startsAt} ~ {sub.endsAt || '无限期'}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">今日调用</div><div className="text-lg font-semibold text-gray-900">{sub.apiCallsToday}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">累计调用</div><div className="text-lg font-semibold text-gray-900">{sub.apiCallsTotal.toLocaleString()}</div></div>
              </div>
            </section>

            <aside className="space-y-4">
              <div className="rounded-xl border border-green-200 bg-green-50 p-4">
                <div className="inline-flex items-center gap-2 text-green-800 text-sm font-medium"><Shield className="w-4 h-4" /><span>链上凭证：{sub.chainProofId}</span></div>
              </div>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><KeyRound className="w-4 h-4" /><span>查看 API 密钥</span></button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><Activity className="w-4 h-4" /><span>查看调用分析</span></button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
