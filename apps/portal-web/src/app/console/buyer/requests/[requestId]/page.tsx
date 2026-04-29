'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { MOCK_REQUESTS, REQ_STATUS_CONFIG } from '@/lib/buyer-requests-data'
import { ArrowLeft, ExternalLink, FileText, Shield, TrendingUp } from 'lucide-react'

export default function BuyerRequestDetailPage() {
  const params = useParams<{ requestId: string }>()
  const req = MOCK_REQUESTS.find((item) => item.requestId === params.requestId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!req) return <div className="p-8 text-lg font-bold">申请不存在</div>

  return (
    <>
      <SessionIdentityBar subjectName="某某科技有限公司" roleName="买家管理员" tenantId="tenant_buyer_001" scope="buyer:requests:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/buyer/requests" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900"><ArrowLeft className="w-4 h-4" /><span>返回我的申请</span></Link>
          <Link href="/console/buyer/subscriptions" className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"><span>前往我的订阅</span><ExternalLink className="w-3.5 h-3.5" /></Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <h1 className="text-3xl font-bold text-gray-900 mb-1">{req.listingTitle}</h1>
            <p className="text-sm text-gray-600">申请编号：{req.requestId}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-5">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">审核状态</div><div className={`status-tag ${REQ_STATUS_CONFIG[req.status].color}`}>{REQ_STATUS_CONFIG[req.status].label}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">供应商</div><div className="font-semibold text-gray-900">{req.supplierName}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">申请套餐</div><div className="font-semibold text-gray-900">{req.plan}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">预计用量</div><div className="font-semibold text-gray-900">{req.expectedUsage}</div></div>
                <div className="rounded-xl border border-gray-200 p-4 md:col-span-2"><div className="text-xs text-gray-500 mb-1">使用用途</div><div className="text-gray-900">{req.usagePurpose}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">链状态</div><div className="font-semibold text-gray-900">{req.chainStatus}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">投影状态</div><div className="font-semibold text-gray-900">{req.projectionStatus}</div></div>
              </div>

              {req.reviewNotes && <div className="rounded-xl border border-orange-200 bg-orange-50 p-4 text-sm text-orange-900">{req.reviewNotes}</div>}
            </section>

            <aside className="space-y-4">
              <div className="rounded-xl border border-green-200 bg-green-50 p-4 inline-flex items-center gap-2 text-green-800 text-sm font-medium"><Shield className="w-4 h-4" /><span>申请链路可追踪</span></div>
              {req.status === 'NEED_MORE_INFO' && <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><FileText className="w-4 h-4" /><span>补充材料</span></button>}
              {req.status === 'APPROVED' && <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><TrendingUp className="w-4 h-4" /><span>前往订阅管理</span></button>}
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
