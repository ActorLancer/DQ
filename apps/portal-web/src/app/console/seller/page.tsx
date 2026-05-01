'use client'

import Link from 'next/link'
import { motion } from 'framer-motion'
import { useEffect, useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { AlertFeed, AmbientOrbs, ChartReveal, ConsoleHero, DashboardFadeItem, DashboardStagger, KpiCard, SectionCard } from '@/components/dashboard'
import SellerRevenueTrendChart from '@/components/charts/SellerRevenueTrendChart'
import SellerApiTrendChart from '@/components/charts/SellerApiTrendChart'
import SellerRevenueCompositionChart from '@/components/charts/SellerRevenueCompositionChart'
import SellerStatusCodeChart from '@/components/charts/SellerStatusCodeChart'
import { Activity, AlertCircle, CircleDollarSign, FileText, Layers, Package, Users } from 'lucide-react'

const KPI = [
  { id: 'k1', label: '已发布商品', value: '12', delta: '+2 本周', icon: Package, tone: 'blue' },
  { id: 'k2', label: '待处理申请', value: '3', delta: '+1 今日', icon: FileText, tone: 'amber' },
  { id: 'k3', label: '活跃订阅客户', value: '28', delta: '+5 本月', icon: Users, tone: 'emerald' },
  { id: 'k4', label: '本月收入', value: '¥128,500', delta: '+12%', icon: CircleDollarSign, tone: 'indigo' },
]

const alerts = [
  { id: 's1', level: 'warning' as const, text: '企业工商风险数据 API 调用量接近配额上限', time: '10 分钟前' },
  { id: 's2', level: 'error' as const, text: '订单 order_12345 链上提交失败，等待重试', time: '1 小时前' },
  { id: 's3', level: 'success' as const, text: '新订阅客户已成功激活授权', time: '2 小时前' },
]

const pending = [
  { id: 'p1', buyer: '某某科技有限公司', product: '企业工商风险数据', plan: '标准版', status: '待审核', time: '2026-04-28 14:30' },
  { id: 'p2', buyer: '智慧消费研究院', product: '消费者行为分析数据', plan: '企业版', status: '待审核', time: '2026-04-28 10:15' },
  { id: 'p3', buyer: '金融数据服务', product: '企业工商风险数据', plan: '企业版', status: '需补充', time: '2026-04-27 16:45' },
]

export default function SellerDashboardV2() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()
  const [chartsReady, setChartsReady] = useState(false)

  useEffect(() => {
    const t = window.setTimeout(() => setChartsReady(true), 520)
    return () => window.clearTimeout(t)
  }, [])

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:listings:write" sessionExpiresAt={sessionExpiresAt} userName="张三" />
      <div className="relative p-8 space-y-6">
        <AmbientOrbs />
        <DashboardStagger className="space-y-6">
          <DashboardFadeItem>
            <ConsoleHero title="Seller Operations Console" subtitle="聚焦商品运营、审批履约、收入与调用稳定性" tone="from-blue-50 to-white" />
          </DashboardFadeItem>

          <section className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
            {KPI.map((k) => (
              <DashboardFadeItem key={k.id}>
                <KpiCard label={k.label} value={k.value} delta={k.delta} icon={k.icon} tone={k.tone} />
              </DashboardFadeItem>
            ))}
          </section>

          <section className="grid grid-cols-1 xl:grid-cols-12 gap-4">
            <DashboardFadeItem className="xl:col-span-8"><SectionCard title="近 30 日收入趋势" right={<Activity className="w-4 h-4 text-gray-400" />}><ChartReveal ready={chartsReady} heightClass="h-72"><SellerRevenueTrendChart /></ChartReveal></SectionCard></DashboardFadeItem>
            <DashboardFadeItem className="xl:col-span-4">
              <SectionCard title="运营脉冲" right={<Layers className="w-4 h-4 text-gray-400" />}>
                <div className="space-y-3">
                  <div className="rounded-xl bg-gradient-to-r from-blue-50 to-indigo-50 border border-blue-100 px-4 py-3">
                    <p className="text-xs text-blue-700">今日新申请</p>
                    <p className="text-2xl font-bold text-blue-900">18</p>
                  </div>
                  <div className="rounded-xl bg-gradient-to-r from-emerald-50 to-teal-50 border border-emerald-100 px-4 py-3">
                    <p className="text-xs text-emerald-700">今日通过率</p>
                    <p className="text-2xl font-bold text-emerald-900">86.4%</p>
                  </div>
                  <div className="rounded-xl bg-gradient-to-r from-amber-50 to-orange-50 border border-amber-100 px-4 py-3">
                    <p className="text-xs text-amber-700">SLA 预警</p>
                    <p className="text-2xl font-bold text-amber-900">3</p>
                  </div>
                </div>
              </SectionCard>
            </DashboardFadeItem>
            <DashboardFadeItem className="xl:col-span-8">
              <SectionCard title="待处理申请" right={<Link href="/console/seller/requests" className="text-sm text-primary-600 hover:text-primary-700">查看全部</Link>}>
                <div className="space-y-3">
                  {pending.map((req) => (
                    <motion.div key={req.id} whileHover={{ y: -1 }} className="rounded-xl border border-gray-200 p-4">
                      <div className="flex items-start justify-between">
                        <div>
                          <p className="text-sm font-medium text-gray-900">{req.buyer}</p>
                          <p className="mt-1 text-sm text-gray-600">{req.product} · {req.plan}</p>
                          <p className="mt-1 text-xs text-gray-500">{req.time}</p>
                        </div>
                        <span className={`status-tag text-xs ${req.status === '待审核' ? 'bg-amber-100 text-amber-800' : 'bg-orange-100 text-orange-800'}`}>{req.status}</span>
                      </div>
                    </motion.div>
                  ))}
                </div>
              </SectionCard>
            </DashboardFadeItem>
            <DashboardFadeItem className="xl:col-span-4"><SectionCard title="系统告警" right={<AlertCircle className="w-4 h-4 text-gray-400" />}><AlertFeed items={alerts} /></SectionCard></DashboardFadeItem>
            <DashboardFadeItem className="xl:col-span-6">
              <SectionCard title="收入构成分布" right={<CircleDollarSign className="w-4 h-4 text-gray-400" />}>
                <ChartReveal ready={chartsReady} heightClass="h-64"><SellerRevenueCompositionChart /></ChartReveal>
              </SectionCard>
            </DashboardFadeItem>
            <DashboardFadeItem className="xl:col-span-6">
              <SectionCard title="调用质量分布" right={<Activity className="w-4 h-4 text-gray-400" />}>
                <ChartReveal ready={chartsReady} heightClass="h-64"><SellerStatusCodeChart /></ChartReveal>
              </SectionCard>
            </DashboardFadeItem>
            <DashboardFadeItem className="xl:col-span-12">
              <SectionCard title="调用波峰观察" right={<Link href="/console/seller/analytics" className="text-sm text-primary-600 hover:text-primary-700">进入调用看板</Link>}>
                <ChartReveal ready={chartsReady} heightClass="h-72"><SellerApiTrendChart /></ChartReveal>
              </SectionCard>
            </DashboardFadeItem>
          </section>
        </DashboardStagger>
      </div>
    </>
  )
}
