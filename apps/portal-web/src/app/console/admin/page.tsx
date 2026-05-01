'use client'

import Link from 'next/link'
import { useEffect, useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { AlertFeed, AmbientOrbs, ChartReveal, ConsoleHero, DashboardFadeItem, DashboardStagger, KpiCard, SectionCard } from '@/components/dashboard'
import AdminRiskTrendChart from '@/components/charts/AdminRiskTrendChart'
import AdminWorkflowFunnelChart from '@/components/charts/AdminWorkflowFunnelChart'
import { Activity, AlertTriangle, Layers, Package, Shield, Users } from 'lucide-react'

const KPI = [
  { id: 'a1', label: '待审核主体', value: '5', delta: '+2 今日', icon: Users, tone: 'amber' },
  { id: 'a2', label: '待审核商品', value: '8', delta: '+3 今日', icon: Package, tone: 'indigo' },
  { id: 'a3', label: '风险告警', value: '12', delta: '-2 较昨日', icon: AlertTriangle, tone: 'red' },
  { id: 'a4', label: '链上失败', value: '3', delta: '0 波动', icon: Shield, tone: 'slate' },
]

const alerts = [
  { id: 'r1', level: 'error' as const, text: '订单 order_12345 链上提交失败，已重试 3 次', time: '10 分钟前' },
  { id: 'r2', level: 'warning' as const, text: '申请 req_67890 投影状态不一致', time: '30 分钟前' },
  { id: 'r3', level: 'info' as const, text: '通知 notif_11111 发送失败，等待 replay', time: '1 小时前' },
]

const pendingSubjects = [
  { id: 's1', name: '某某数据科技有限公司', type: '供应商', risk: '低风险', time: '2026-04-28 10:30' },
  { id: 's2', name: '智慧金融服务有限公司', type: '买方', risk: '中风险', time: '2026-04-28 09:15' },
  { id: 's3', name: '大数据研究院', type: '供应商', risk: '高风险', time: '2026-04-27 16:45' },
]

const pendingListings = [
  { id: 'l1', title: '新能源汽车数据', supplier: '某某数据科技', industry: '能源', risk: '低风险', time: '2026-04-28 14:20' },
  { id: 'l2', title: '医疗影像数据集', supplier: '医疗大数据中心', industry: '医疗', risk: '高风险', time: '2026-04-28 11:30' },
]

export default function AdminDashboardV2() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()
  const [chartsReady, setChartsReady] = useState(false)

  useEffect(() => {
    const t = window.setTimeout(() => setChartsReady(true), 520)
    return () => window.clearTimeout(t)
  }, [])

  return (
    <>
      <SessionIdentityBar subjectName="数据交易平台运营中心" roleName="平台管理员" tenantId="tenant_platform_001" scope="admin:all:write" sessionExpiresAt={sessionExpiresAt} userName="管理员" />
      <div className="relative p-8 space-y-6">
        <AmbientOrbs />
        <DashboardStagger className="space-y-6">
          <DashboardFadeItem>
            <ConsoleHero title="Platform Governance Console" subtitle="聚焦平台审核、风控告警与系统一致性治理" tone="from-amber-50 to-white" />
          </DashboardFadeItem>

          <section className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
            {KPI.map((k) => (
              <DashboardFadeItem key={k.id}>
                <KpiCard label={k.label} value={k.value} delta={k.delta} icon={k.icon} tone={k.tone} />
              </DashboardFadeItem>
            ))}
          </section>

          <section className="grid grid-cols-1 xl:grid-cols-12 gap-4">
            <DashboardFadeItem className="xl:col-span-8">
              <SectionCard title="风险事件趋势（7 日）" right={<Activity className="w-4 h-4 text-gray-400" />}>
                <ChartReveal ready={chartsReady} heightClass="h-72"><AdminRiskTrendChart /></ChartReveal>
              </SectionCard>
            </DashboardFadeItem>

            <DashboardFadeItem className="xl:col-span-4">
              <SectionCard title="治理漏斗" right={<Layers className="w-4 h-4 text-gray-400" />}>
                <ChartReveal ready={chartsReady} heightClass="h-72"><AdminWorkflowFunnelChart /></ChartReveal>
              </SectionCard>
            </DashboardFadeItem>

            <DashboardFadeItem className="xl:col-span-4">
              <SectionCard title="待审核主体" right={<Link href="/admin/console/subjects" className="text-sm text-primary-600 hover:text-primary-700">查看全部</Link>}>
                <div className="space-y-3">
                  {pendingSubjects.map((subject) => (
                    <div key={subject.id} className="rounded-xl border border-gray-200 p-4 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-sm">
                      <p className="text-sm font-medium text-gray-900">{subject.name}</p>
                      <p className="mt-1 text-xs text-gray-600">{subject.type} · {subject.time}</p>
                      <span className={`mt-2 inline-flex rounded-full px-2 py-1 text-xs ${subject.risk === '高风险' ? 'bg-red-100 text-red-800' : subject.risk === '中风险' ? 'bg-amber-100 text-amber-800' : 'bg-emerald-100 text-emerald-800'}`}>{subject.risk}</span>
                    </div>
                  ))}
                </div>
              </SectionCard>
            </DashboardFadeItem>

            <DashboardFadeItem className="xl:col-span-4">
              <SectionCard title="待审核商品" right={<Link href="/admin/console/listings" className="text-sm text-primary-600 hover:text-primary-700">查看全部</Link>}>
                <div className="space-y-3">
                  {pendingListings.map((listing) => (
                    <div key={listing.id} className="rounded-xl border border-gray-200 p-4 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-sm">
                      <p className="text-sm font-medium text-gray-900">{listing.title}</p>
                      <p className="mt-1 text-xs text-gray-600">{listing.supplier} · {listing.industry}</p>
                      <p className="mt-1 text-xs text-gray-500">{listing.time}</p>
                      <span className={`mt-2 inline-flex rounded-full px-2 py-1 text-xs ${listing.risk === '高风险' ? 'bg-red-100 text-red-800' : 'bg-emerald-100 text-emerald-800'}`}>{listing.risk}</span>
                    </div>
                  ))}
                </div>
              </SectionCard>
            </DashboardFadeItem>

            <DashboardFadeItem className="xl:col-span-4">
              <SectionCard title="风险告警" right={<Activity className="w-4 h-4 text-gray-400" />}>
                <AlertFeed items={alerts} />
              </SectionCard>
            </DashboardFadeItem>

            <DashboardFadeItem className="xl:col-span-12">
              <SectionCard title="治理优先级队列" right={<Link href="/admin/console/audit" className="text-sm text-primary-600 hover:text-primary-700">进入风险审计</Link>}>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
                  <div className="rounded-xl border border-red-200 bg-red-50 p-4 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-sm">
                    <p className="text-xs text-red-700">P0 紧急</p>
                    <p className="mt-1 text-2xl font-bold text-red-900">3</p>
                    <p className="mt-1 text-xs text-red-700">链上失败重试超限 / 高危主体拦截</p>
                  </div>
                  <div className="rounded-xl border border-amber-200 bg-amber-50 p-4 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-sm">
                    <p className="text-xs text-amber-700">P1 高优先</p>
                    <p className="mt-1 text-2xl font-bold text-amber-900">9</p>
                    <p className="mt-1 text-xs text-amber-700">投影异常 / 审核超时 / 回执缺失</p>
                  </div>
                  <div className="rounded-xl border border-emerald-200 bg-emerald-50 p-4 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-sm">
                    <p className="text-xs text-emerald-700">P2 常规</p>
                    <p className="mt-1 text-2xl font-bold text-emerald-900">14</p>
                    <p className="mt-1 text-xs text-emerald-700">常规复核任务与低风险抽检</p>
                  </div>
                </div>
              </SectionCard>
            </DashboardFadeItem>
          </section>
        </DashboardStagger>
      </div>
    </>
  )
}
