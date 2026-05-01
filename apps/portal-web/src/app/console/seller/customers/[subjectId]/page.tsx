'use client'

import Link from 'next/link'
import { useParams } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { SELLER_CUSTOMERS } from '@/lib/seller-customers-data'
import { Activity, ArrowLeft, ExternalLink, Mail, Phone, Shield, Users } from 'lucide-react'

export default function SellerCustomerDetailPage() {
  const params = useParams<{ subjectId: string }>()
  const customer = SELLER_CUSTOMERS.find((item) => item.subjectId === params.subjectId)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!customer) return <div className="p-8 text-lg font-bold">客户不存在</div>

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:customers:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="flex items-center justify-between mb-6">
          <Link href="/console/seller/customers" className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900"><ArrowLeft className="w-4 h-4" /><span>返回订阅客户</span></Link>
          <Link href="/console/seller/contracts" className="inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700"><span>前往合同发票</span><ExternalLink className="w-3.5 h-3.5" /></Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-6 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <h1 className="text-3xl font-bold text-gray-900 mb-1">{customer.name}</h1>
            <p className="text-sm text-gray-600">客户主体：{customer.subjectId}</p>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-5">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">客户类型</div><div className="font-semibold text-gray-900">{customer.type === 'ENTERPRISE' ? '企业客户' : '个人客户'}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">所属行业</div><div className="font-semibold text-gray-900">{customer.industry}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">活跃订阅</div><div className="text-lg font-semibold text-gray-900">{customer.activeSubscriptions}/{customer.subscriptionCount}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">累计收入</div><div className="text-lg font-semibold text-gray-900">¥{customer.totalRevenue.toLocaleString()}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">总调用</div><div className="text-lg font-semibold text-gray-900">{customer.totalCalls.toLocaleString()}</div></div>
                <div className="rounded-xl border border-gray-200 p-4"><div className="text-xs text-gray-500 mb-1">成功率</div><div className="text-lg font-semibold text-green-700">{customer.successRate}%</div></div>
              </div>
            </section>

            <aside className="space-y-3">
              <div className="rounded-xl border border-blue-200 bg-blue-50 p-4 text-blue-900 text-sm space-y-2">
                <div className="inline-flex items-center gap-2 font-medium"><Users className="w-4 h-4" />联系人：{customer.contactPerson}</div>
                <div className="inline-flex items-center gap-2"><Mail className="w-4 h-4" />{customer.contactEmail}</div>
                <div className="inline-flex items-center gap-2"><Phone className="w-4 h-4" />{customer.contactPhone}</div>
              </div>
              <div className="rounded-xl border border-green-200 bg-green-50 p-4 inline-flex items-center gap-2 text-green-800 text-sm font-medium"><Shield className="w-4 h-4" /><span>客户履约风险：{customer.riskLevel}</span></div>
              <button className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium inline-flex items-center justify-center gap-2"><Mail className="w-4 h-4" /><span>发送运营消息</span></button>
              <button className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><Activity className="w-4 h-4" /><span>查看调用看板</span></button>
            </aside>
          </div>
        </div>
      </div>
    </>
  )
}
