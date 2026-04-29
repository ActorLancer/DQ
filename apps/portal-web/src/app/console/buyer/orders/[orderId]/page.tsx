'use client'

import Link from 'next/link'
import { notFound, useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { getBuyerApiKeys } from '@/lib/buyer-api-keys-storage'
import {
  INVOICE_STATUS_CONFIG,
  MOCK_ORDERS,
  ORDER_STATUS_CONFIG,
  ORDER_TYPE_CONFIG,
} from '@/lib/buyer-orders-data'
import { ArrowLeft, CheckCircle2, ExternalLink, Receipt, ShieldCheck } from 'lucide-react'

export default function BuyerOrderDetailPage() {
  const params = useParams<{ orderId: string }>()
  const order = MOCK_ORDERS.find((item) => item.orderId === params.orderId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!order) return notFound()

  const linkedKeys = getBuyerApiKeys().filter((item) => item.orderId === order.orderId)

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:orders:read"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/buyer/orders" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900">
            <ArrowLeft className="w-4 h-4" />
            <span>返回订单账单</span>
          </Link>
          <Link
            href={`/console/buyer/billing/${order.orderId}`}
            className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"
          >
            <span>查看账单详情</span>
            <ExternalLink className="w-3.5 h-3.5" />
          </Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <div className="flex items-start justify-between gap-4">
              <div>
                <h1 className="text-2xl font-bold text-gray-900 mb-1">{order.listingTitle}</h1>
                <p className="text-sm text-gray-600">订单号：{order.orderId}</p>
              </div>
              <div className="flex items-center gap-2">
                <span className={`status-tag ${ORDER_TYPE_CONFIG[order.orderType].color}`}>{ORDER_TYPE_CONFIG[order.orderType].label}</span>
                <span className={`status-tag ${ORDER_STATUS_CONFIG[order.status].color}`}>{ORDER_STATUS_CONFIG[order.status].label}</span>
              </div>
            </div>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-5">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">供应商</div>
                  <div className="font-medium text-gray-900">{order.supplierName}</div>
                </div>
                <div className="rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">套餐</div>
                  <div className="font-medium text-gray-900">{order.plan}</div>
                </div>
                <div className="rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">订单金额</div>
                  <div className="font-bold text-gray-900 text-xl">¥{order.amount.toLocaleString()}</div>
                </div>
                <div className="rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">支付方式</div>
                  <div className="font-medium text-gray-900">{order.paymentMethod || '待支付'}</div>
                </div>
              </div>

              <div className="rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-3">订单履约时间线</div>
                <div className="space-y-3 text-sm">
                  <div className="flex items-center gap-2 text-gray-700">
                    <CheckCircle2 className="w-4 h-4 text-green-600" />
                    <span>订单创建：{order.createdAt}</span>
                  </div>
                  <div className="flex items-center gap-2 text-gray-700">
                    <CheckCircle2 className={`w-4 h-4 ${order.paidAt ? 'text-green-600' : 'text-gray-300'}`} />
                    <span>支付完成：{order.paidAt || '待支付'}</span>
                  </div>
                  <div className="flex items-center gap-2 text-gray-700">
                    <CheckCircle2 className={`w-4 h-4 ${order.status === 'PAID' ? 'text-green-600' : 'text-gray-300'}`} />
                    <span>交付授权：{order.status === 'PAID' ? '已开通' : '未开通'}</span>
                  </div>
                </div>
              </div>
            </section>

            <aside className="space-y-4">
              <div className="rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-3">账单状态</div>
                <span className={`status-tag ${INVOICE_STATUS_CONFIG[order.invoiceStatus].color}`}>{INVOICE_STATUS_CONFIG[order.invoiceStatus].label}</span>
                {order.invoiceNumber && (
                  <div className="mt-3 text-xs text-gray-600">发票号：{order.invoiceNumber}</div>
                )}
              </div>

              <div className="rounded-xl border border-blue-200 bg-blue-50 p-5">
                <div className="text-sm font-semibold text-blue-900 mb-2">关联 API 密钥</div>
                <div className="text-sm text-blue-800 mb-3">共 {linkedKeys.length} 个</div>
                <Link href={`/console/buyer/api-keys?orderId=${order.orderId}`} className="text-sm text-primary-700 hover:text-primary-800">
                  前往 API 密钥管理 →
                </Link>
              </div>

              <div className="rounded-xl border border-emerald-200 bg-emerald-50 p-5">
                <div className="flex items-center gap-2 text-emerald-900 text-sm font-semibold mb-1">
                  <ShieldCheck className="w-4 h-4" />
                  <span>交易合规记录</span>
                </div>
                <div className="text-xs text-emerald-800">当前为前端演示数据，后续将接入审计与链上凭证。</div>
              </div>

              <button className="w-full rounded-lg bg-primary-600 text-white py-2.5 hover:bg-primary-700 font-medium">
                下载订单凭证
              </button>
              <button className="w-full rounded-lg border border-gray-300 text-gray-700 py-2.5 hover:bg-gray-50 font-medium inline-flex items-center justify-center gap-2">
                <Receipt className="w-4 h-4" />
                <span>申请发票</span>
              </button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
