'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { SELLER_LISTINGS } from '@/lib/seller-listings-data'
import { ArrowLeft, BarChart3, ExternalLink, FileText, Shield } from 'lucide-react'

export default function SellerListingDetailPage() {
  const params = useParams<{ listingId: string }>()
  const listing = SELLER_LISTINGS.find((item) => item.id === params.listingId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!listing) return <div className="p-8 text-lg font-bold">商品不存在</div>

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:listings:write" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/seller/listings" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900"><ArrowLeft className="w-4 h-4" /><span>返回商品管理</span></Link>
          <Link href="/console/seller/requests" className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"><span>查看相关申请</span><ExternalLink className="w-3.5 h-3.5" /></Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <h1 className="text-3xl font-bold text-gray-900 mb-1">{listing.title}</h1>
            <p className="text-sm text-gray-600">商品ID：{listing.id} · 行业：{listing.industry}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-5">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">状态</div><div className="font-semibold text-gray-900">{listing.status}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">价格模式</div><div className="font-semibold text-gray-900">{listing.pricingModel}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">交付方式</div><div className="font-semibold text-gray-900">{listing.deliveryMethods.join(' / ')}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">质量评分</div><div className="text-lg font-semibold text-gray-900">{listing.qualityScore || '-'}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">申请数</div><div className="text-lg font-semibold text-gray-900">{listing.requestCount}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">订阅客户</div><div className="text-lg font-semibold text-gray-900">{listing.subscriberCount}</div></div>
              </div>
            </section>

            <aside className="space-y-3">
              <div className="rounded-xl border border-green-200 bg-green-50 p-4 inline-flex items-center gap-2 text-green-800 text-sm font-medium"><Shield className="w-4 h-4" /><span>链状态：{listing.chainStatus}</span></div>
              <button className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium">编辑商品</button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><FileText className="w-4 h-4" /><span>查看审核材料</span></button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><BarChart3 className="w-4 h-4" /><span>查看调用看板</span></button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
