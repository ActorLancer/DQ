'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { INVOICE_STATUS_CONFIG, MOCK_ORDERS } from '@/lib/buyer-orders-data'
import { ArrowLeft, CircleDollarSign, ExternalLink, FileSpreadsheet, ReceiptText } from 'lucide-react'

export default function BuyerBillingDetailPage() {
  const params = useParams<{ orderId: string }>()
  const order = MOCK_ORDERS.find((item) => item.orderId === params.orderId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!order) {
    return (
      <div className="p-8">
        <div className="rounded-xl border border-gray-200 bg-white p-8 text-center">
          <h1 className="text-xl font-bold text-gray-900 mb-2">未找到账单</h1>
          <Link href="/console/buyer/orders" className="text-primary-600 hover:text-primary-700">
            返回订单账单
          </Link>
        </div>
      </div>
    )
  }

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
          <Link href={`/console/buyer/orders/${order.orderId}`} className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700">
            <span>查看订单详情</span>
            <ExternalLink className="w-3.5 h-3.5" />
          </Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-blue-50 to-white">
            <h1 className="text-2xl font-bold text-gray-900 mb-1">账单详情</h1>
            <p className="text-sm text-gray-600">订单号：{order.orderId}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-5">
              <div className="rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-4">账单摘要</div>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                  <div>
                    <div className="text-gray-500 mb-1">商品名称</div>
                    <div className="font-medium text-gray-900">{order.listingTitle}</div>
                  </div>
                  <div>
                    <div className="text-gray-500 mb-1">供应商</div>
                    <div className="font-medium text-gray-900">{order.supplierName}</div>
                  </div>
                  <div>
                    <div className="text-gray-500 mb-1">账单金额</div>
                    <div className="font-bold text-gray-900 text-xl">¥{order.amount.toLocaleString()}</div>
                  </div>
                  <div>
                    <div className="text-gray-500 mb-1">支付状态</div>
                    <div className="font-medium text-gray-900">{order.status === 'PAID' ? '已支付' : '未支付'}</div>
                  </div>
                </div>
              </div>

              <div className="rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-4">费用明细（示例）</div>
                <div className="space-y-2 text-sm">
                  <div className="flex items-center justify-between">
                    <span className="text-gray-600">数据服务费</span>
                    <span className="font-medium text-gray-900">¥{(order.amount * 0.94).toFixed(2)}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <span className="text-gray-600">平台服务费</span>
                    <span className="font-medium text-gray-900">¥{(order.amount * 0.06).toFixed(2)}</span>
                  </div>
                  <div className="border-t border-gray-200 pt-2 mt-2 flex items-center justify-between">
                    <span className="text-gray-900 font-semibold">合计</span>
                    <span className="text-gray-900 font-bold">¥{order.amount.toLocaleString()}</span>
                  </div>
                </div>
              </div>
            </section>

            <aside className="space-y-4">
              <div className="rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-3">发票状态</div>
                <span className={`status-tag ${INVOICE_STATUS_CONFIG[order.invoiceStatus].color}`}>{INVOICE_STATUS_CONFIG[order.invoiceStatus].label}</span>
                <div className="mt-3 text-xs text-gray-600">发票号：{order.invoiceNumber || '暂无'}</div>
              </div>

              <div className="rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-3">结算信息</div>
                <div className="space-y-2 text-xs text-gray-700">
                  <div>支付方式：{order.paymentMethod || '待支付'}</div>
                  <div>支付时间：{order.paidAt || '待支付'}</div>
                  <div>币种：{order.currency}</div>
                </div>
              </div>

              <button className="w-full rounded-lg bg-primary-600 text-white py-2.5 hover:bg-primary-700 font-medium inline-flex items-center justify-center gap-2">
                <ReceiptText className="w-4 h-4" />
                <span>下载账单 PDF</span>
              </button>
              <button className="w-full rounded-lg border border-blue-300 text-blue-700 py-2.5 hover:bg-blue-50 font-medium inline-flex items-center justify-center gap-2">
                <FileSpreadsheet className="w-4 h-4" />
                <span>导出对账明细</span>
              </button>
              <button className="w-full rounded-lg border border-gray-300 text-gray-700 py-2.5 hover:bg-gray-50 font-medium inline-flex items-center justify-center gap-2">
                <CircleDollarSign className="w-4 h-4" />
                <span>申请开票</span>
              </button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
