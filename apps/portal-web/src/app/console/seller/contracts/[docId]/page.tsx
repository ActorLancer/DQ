'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { SELLER_CONTRACT_DOCS } from '@/lib/seller-contracts-data'
import { ArrowLeft, Download, ExternalLink, FileText, Receipt, Shield } from 'lucide-react'

export default function SellerContractDetailPage() {
  const params = useParams<{ docId: string }>()
  const doc = SELLER_CONTRACT_DOCS.find((item) => item.id === params.docId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!doc) return <div className="p-8 text-lg font-bold">文档不存在</div>

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:contracts:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/seller/contracts" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900"><ArrowLeft className="w-4 h-4" /><span>返回合同发票</span></Link>
          <Link href="/console/seller/customers" className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"><span>查看相关客户</span><ExternalLink className="w-3.5 h-3.5" /></Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <h1 className="text-3xl font-bold text-gray-900 mb-1">{doc.docNo}</h1>
            <p className="text-sm text-gray-600">{doc.type === 'CONTRACT' ? '合同文档' : '发票文档'} · 关联订单：{doc.relatedOrderId}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-5">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">客户</div><div className="font-semibold text-gray-900">{doc.customerName}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">商品</div><div className="font-semibold text-gray-900">{doc.listingTitle}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">金额</div><div className="text-lg font-semibold text-gray-900">¥{doc.amount.toLocaleString()}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">状态</div><div className="font-semibold text-gray-900">{doc.status}</div></div>
                <div className="rounded-xl border border-gray-200 p-4 md:col-span-2"><div className="text-xs text-gray-500 mb-1">创建时间</div><div className="font-semibold text-gray-900">{doc.createdAt}</div></div>
              </div>
            </section>

            <aside className="space-y-3">
              <div className="rounded-xl border border-green-200 bg-green-50 p-4 inline-flex items-center gap-2 text-green-800 text-sm font-medium"><Shield className="w-4 h-4" /><span>文档审计状态可追踪</span></div>
              <button className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium inline-flex items-center justify-center gap-2"><Download className="w-4 h-4" /><span>下载文档</span></button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><Receipt className="w-4 h-4" /><span>开票处理</span></button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><FileText className="w-4 h-4" /><span>查看合同条款</span></button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
