'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { SELLER_REQUESTS } from '@/lib/seller-requests-data'
import { ArrowLeft, CheckCircle, ExternalLink, FileText, Shield, XCircle } from 'lucide-react'

export default function SellerRequestDetailPage() {
  const params = useParams<{ requestId: string }>()
  const req = SELLER_REQUESTS.find((item) => item.requestId === params.requestId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!req) return <div className="p-8 text-lg font-bold">申请不存在</div>

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:requests:write" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/seller/requests" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900"><ArrowLeft className="w-4 h-4" /><span>返回申请审批</span></Link>
          <Link href="/console/seller/customers" className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"><span>前往订阅客户</span><ExternalLink className="w-3.5 h-3.5" /></Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <h1 className="text-3xl font-bold text-gray-900 mb-1">{req.listingTitle}</h1>
            <p className="text-sm text-gray-600">申请编号：{req.requestId} · 买方：{req.buyerName}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-5">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">申请状态</div><div className="font-semibold text-gray-900">{req.workflowStatus}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">买方主体</div><div className="font-semibold text-gray-900">{req.buyerSubject}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">套餐</div><div className="font-semibold text-gray-900">{req.planName}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">预计用量</div><div className="font-semibold text-gray-900">{req.expectedUsage}</div></div>
                <div className="rounded-xl border border-gray-200 p-4 md:col-span-2"><div className="text-xs text-gray-500 mb-1">使用用途</div><div className="text-gray-900">{req.usagePurpose}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">链状态</div><div className="font-semibold text-gray-900">{req.chainStatus}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">投影状态</div><div className="font-semibold text-gray-900">{req.projectionStatus}</div></div>
              </div>
            </section>

            <aside className="space-y-3">
              <div className="rounded-xl border border-green-200 bg-green-50 p-4 inline-flex items-center gap-2 text-green-800 text-sm font-medium"><Shield className="w-4 h-4" /><span>审批链路可追踪</span></div>
              <button className="w-full h-10 px-4 bg-success-600 text-white rounded-lg hover:bg-success-700 text-sm font-medium inline-flex items-center justify-center gap-2"><CheckCircle className="w-4 h-4" /><span>通过申请</span></button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><FileText className="w-4 h-4" /><span>要求补充材料</span></button>
              <button className="w-full h-10 px-4 border border-red-300 text-red-700 rounded-lg hover:bg-red-50 text-sm font-medium inline-flex items-center justify-center gap-2"><XCircle className="w-4 h-4" /><span>拒绝申请</span></button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
