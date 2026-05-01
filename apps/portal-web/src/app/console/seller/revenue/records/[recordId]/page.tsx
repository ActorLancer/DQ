'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { SELLER_REVENUE_RECORDS } from '@/lib/seller-revenue-data'
import { ArrowLeft, ExternalLink, FileText, Receipt, Shield } from 'lucide-react'

export default function SellerRevenueRecordDetailPage() {
  const params = useParams<{ recordId: string }>()
  const record = SELLER_REVENUE_RECORDS.find((item) => item.id === params.recordId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!record) return <div className="p-8 text-lg font-bold">记录不存在</div>

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:revenue:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/seller/revenue" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900"><ArrowLeft className="w-4 h-4" /><span>返回收入看板</span></Link>
          <Link href="/console/seller/contracts" className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"><span>查看合同发票</span><ExternalLink className="w-3.5 h-3.5" /></Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <h1 className="text-3xl font-bold text-gray-900 mb-1">收入记录 {record.id}</h1>
            <p className="text-sm text-gray-600">客户：{record.customer} · 商品：{record.listing}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">日期</div><div className="font-semibold text-gray-900">{record.date}</div></div>
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">金额</div><div className="text-lg font-semibold text-gray-900">¥{record.amount.toLocaleString()}</div></div>
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">类型</div><div className="font-semibold text-gray-900">{record.type}</div></div>
              <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">状态</div><div className="font-semibold text-gray-900">{record.status}</div></div>
              <div className="rounded-xl border border-gray-200 p-4 md:col-span-2"><div className="text-xs text-gray-500 mb-1">套餐</div><div className="font-semibold text-gray-900">{record.plan}</div></div>
            </section>

            <aside className="space-y-3">
              <div className="rounded-xl border border-green-200 bg-green-50 p-4 inline-flex items-center gap-2 text-green-800 text-sm font-medium"><Shield className="w-4 h-4" /><span>收入记录已纳入审计</span></div>
              <button className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium inline-flex items-center justify-center gap-2"><FileText className="w-4 h-4" /><span>导出记录</span></button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><Receipt className="w-4 h-4" /><span>关联发票</span></button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
