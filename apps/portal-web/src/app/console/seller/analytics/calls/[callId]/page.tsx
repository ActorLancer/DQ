'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { SELLER_API_CALLS } from '@/lib/seller-analytics-data'
import { AlertTriangle, ArrowLeft, CheckCircle, ExternalLink, Shield } from 'lucide-react'

export default function SellerApiCallDetailPage() {
  const params = useParams<{ callId: string }>()
  const call = SELLER_API_CALLS.find((item) => item.id === params.callId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!call) return <div className="p-8 text-lg font-bold">调用记录不存在</div>

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:analytics:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/seller/analytics" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900"><ArrowLeft className="w-4 h-4" /><span>返回调用看板</span></Link>
          <Link href="/console/seller/customers" className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"><span>查看客户详情</span><ExternalLink className="w-3.5 h-3.5" /></Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <h1 className="text-3xl font-bold text-gray-900 mb-1">API 调用明细 {call.id}</h1>
            <p className="text-sm text-gray-600">{call.customer} · {call.listing}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">请求时间</div><div className="font-semibold text-gray-900">{call.timestamp}</div></div>
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">接口地址</div><code className="text-xs font-mono text-gray-900">{call.endpoint}</code></div>
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">请求方法</div><div className="font-semibold text-gray-900">{call.method}</div></div>
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">状态码</div><div className="font-semibold text-gray-900">{call.statusCode}</div></div>
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">响应耗时</div><div className="font-semibold text-gray-900">{call.responseTime}ms</div></div>
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">调用结果</div><div className="font-semibold text-gray-900">{call.success ? 'SUCCESS' : 'FAILED'}</div></div>
            </section>

            <aside className="space-y-3">
              <div className="rounded-xl border border-green-200 bg-green-50 p-4 inline-flex items-center gap-2 text-green-800 text-sm font-medium"><Shield className="w-4 h-4" /><span>调用审计可追踪</span></div>
              {call.success ? (
                <div className="rounded-xl border border-green-200 bg-green-50 p-4 inline-flex items-center gap-2 text-green-800 text-sm font-medium"><CheckCircle className="w-4 h-4" /><span>调用成功</span></div>
              ) : (
                <div className="rounded-xl border border-red-200 bg-red-50 p-4 inline-flex items-center gap-2 text-red-800 text-sm font-medium"><AlertTriangle className="w-4 h-4" /><span>调用失败，建议排查</span></div>
              )}
              <button className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium">重放请求</button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium">查看上下文日志</button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
