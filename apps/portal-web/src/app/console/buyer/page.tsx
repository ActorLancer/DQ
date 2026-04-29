'use client'

import Link from 'next/link'
import { useMemo, useRef, useState } from 'react'
import { LayoutGroup, motion } from 'framer-motion'
import gsap from 'gsap'
import { Flip } from 'gsap/Flip'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import ApiCallsTrendChart from '@/components/charts/ApiCallsTrendChart'
import ResponseTimeChart from '@/components/charts/ResponseTimeChart'
import UsageDistributionChart from '@/components/charts/UsageDistributionChart'
import { AlertFeed, ConsoleHero, KpiCard, SectionCard } from '@/components/dashboard'
import { Activity, BadgeCheck, Bell, Calendar, CircleDollarSign, Clock3, FileText, Package, Sparkles } from 'lucide-react'
import { BUYER_NOTIFICATIONS } from '@/lib/notifications-data'

gsap.registerPlugin(Flip)

type FocusMode = 'overview' | 'subscriptions' | 'requests' | 'cost' | 'ops'

const KPI_CARDS = [
  { id: 'active_subscriptions', label: '活跃订阅', value: '8', delta: '+2 本周', icon: Package, tone: 'blue', focus: ['overview', 'subscriptions'] },
  { id: 'pending_requests', label: '待处理申请', value: '2', delta: '48h SLA 内', icon: FileText, tone: 'amber', focus: ['overview', 'requests'] },
  { id: 'api_calls_month', label: '本月 API 调用', value: '125,680', delta: '+18.2%', icon: Activity, tone: 'emerald', focus: ['overview', 'ops'] },
  { id: 'spend_month', label: '本月支出', value: '¥28,500', delta: '+8.0%', icon: CircleDollarSign, tone: 'indigo', focus: ['overview', 'cost'] },
]

const REQUEST_PIPELINE = [
  { label: '待供应商审核', value: 2 },
  { label: '待平台复核', value: 1 },
  { label: '需补充材料', value: 1 },
  { label: '已通过', value: 12 },
]

const SUB_HEALTH = [
  { title: '企业工商风险数据', usage: 65, status: '稳态', eta: '29 天到期' },
  { title: '消费者行为分析数据', usage: 25, status: '健康', eta: '246 天到期' },
  { title: '物流轨迹实时数据', usage: 42, status: '稳态', eta: '无限期' },
]

const focusButtons: Array<{ id: FocusMode; label: string }> = [
  { id: 'overview', label: '总览' },
  { id: 'subscriptions', label: '订阅' },
  { id: 'requests', label: '申请' },
  { id: 'cost', label: '成本' },
  { id: 'ops', label: '调用' },
]

export default function BuyerDashboardV2() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()
  const [focus, setFocus] = useState<FocusMode>('overview')
  const [range, setRange] = useState<'7d' | '30d' | '90d'>('30d')
  const bentoRef = useRef<HTMLDivElement>(null)

  const visibleKpis = useMemo(() => {
    if (focus === 'overview') return KPI_CARDS
    return KPI_CARDS.filter((k) => k.focus.includes(focus))
  }, [focus])

  const animateFlip = () => {
    if (!bentoRef.current) return
    const state = Flip.getState(bentoRef.current.querySelectorAll('[data-flip-node]'))
    requestAnimationFrame(() => {
      Flip.from(state, { duration: 0.46, ease: 'power2.out', absolute: false, stagger: 0.015 })
    })
  }

  const focusTone: Record<FocusMode, string> = {
    overview: 'from-slate-50 to-white',
    subscriptions: 'from-blue-50 to-white',
    requests: 'from-amber-50 to-white',
    cost: 'from-indigo-50 to-white',
    ops: 'from-emerald-50 to-white',
  }

  return (
    <>
      <SessionIdentityBar subjectName="某某科技有限公司" roleName="买家管理员" tenantId="tenant_buyer_001" scope="buyer:subscriptions:read" sessionExpiresAt={sessionExpiresAt} userName="李四" />

      <div className="p-8 space-y-6">
        <ConsoleHero
          title="Buyer Intelligence Console"
          subtitle="以订阅健康、申请流转与成本效率为核心的交易驾驶舱"
          tone={focusTone[focus]}
          right={
            <div className="flex flex-wrap items-center gap-2">
              <div className="inline-flex rounded-xl border border-gray-200 bg-white p-1">
                {focusButtons.map((item) => (
                  <button
                    key={item.id}
                    onClick={() => {
                      if (item.id !== focus) {
                        animateFlip()
                        setFocus(item.id)
                      }
                    }}
                    className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${focus === item.id ? 'bg-primary-600 text-white' : 'text-gray-700 hover:bg-gray-100'}`}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
              <div className="inline-flex rounded-xl border border-gray-200 bg-white p-1">
                {(['7d', '30d', '90d'] as const).map((r) => (
                  <button key={r} onClick={() => { if (r !== range) { animateFlip(); setRange(r) } }} className={`px-3 py-1.5 text-sm rounded-lg transition-colors ${range === r ? 'bg-gray-900 text-white' : 'text-gray-700 hover:bg-gray-100'}`}>
                    {r}
                  </button>
                ))}
              </div>
            </div>
          }
        />

        <LayoutGroup>
          <div ref={bentoRef} className="space-y-4">
            <motion.section layout className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-4">
              {visibleKpis.map((card) => (
                <motion.div key={card.id} layout data-flip-node>
                  <KpiCard label={card.label} value={card.value} delta={card.delta} icon={card.icon} tone={card.tone} />
                </motion.div>
              ))}
            </motion.section>

            <motion.section layout className="grid grid-cols-1 xl:grid-cols-12 gap-4">
              <motion.div layout data-flip-node className="xl:col-span-7"><SectionCard title="调用趋势与成功质量" right={<span className="text-xs text-gray-500">范围：{range}</span>}><div className="h-80"><ApiCallsTrendChart /></div></SectionCard></motion.div>
              <motion.div layout data-flip-node className="xl:col-span-5"><SectionCard title="调用延迟分布" right={<Clock3 className="w-4 h-4 text-gray-400" />}><div className="h-80"><ResponseTimeChart /></div></SectionCard></motion.div>
              <motion.div layout data-flip-node className="xl:col-span-4">
                <SectionCard title="订阅健康矩阵" right={<BadgeCheck className="w-4 h-4 text-emerald-500" />}>
                  <div className="space-y-3">
                    {SUB_HEALTH.map((item) => (
                      <div key={item.title} className="rounded-xl border border-gray-200 p-3">
                        <div className="flex items-center justify-between mb-2"><p className="text-sm font-medium text-gray-900">{item.title}</p><span className="text-xs text-gray-500">{item.eta}</span></div>
                        <div className="h-2 rounded-full bg-gray-200 overflow-hidden mb-2"><div className={`h-full ${item.usage >= 80 ? 'bg-red-500' : item.usage >= 60 ? 'bg-yellow-500' : 'bg-emerald-500'}`} style={{ width: `${item.usage}%` }} /></div>
                        <div className="flex items-center justify-between text-xs text-gray-600"><span>使用率 {item.usage}%</span><span>{item.status}</span></div>
                      </div>
                    ))}
                  </div>
                </SectionCard>
              </motion.div>
              <motion.div layout data-flip-node className="xl:col-span-4">
                <SectionCard title="申请进度看板" right={<FileText className="w-4 h-4 text-gray-400" />}>
                  <div className="space-y-3">
                    {REQUEST_PIPELINE.map((step, idx) => (
                      <div key={step.label} className="rounded-xl border border-gray-200 p-3"><div className="flex items-center justify-between"><span className="text-sm text-gray-700">{idx + 1}. {step.label}</span><span className="text-sm font-semibold text-gray-900">{step.value}</span></div></div>
                    ))}
                  </div>
                  <Link href="/console/buyer/requests" className="mt-4 inline-flex items-center gap-1 text-sm text-primary-600 hover:text-primary-700">进入申请中心</Link>
                </SectionCard>
              </motion.div>
              <motion.div layout data-flip-node className="xl:col-span-4"><SectionCard title="调用构成分布" right={<Activity className="w-4 h-4 text-gray-400" />}><div className="h-64"><UsageDistributionChart /></div></SectionCard></motion.div>
            </motion.section>

            <motion.section layout className="grid grid-cols-1 xl:grid-cols-12 gap-4">
              <motion.div layout data-flip-node className="xl:col-span-7">
                <SectionCard title="事件通知与告警" right={<Bell className="w-4 h-4 text-gray-400" />}>
                  <AlertFeed
                    items={BUYER_NOTIFICATIONS.map((x) => ({
                      id: x.id,
                      level: x.level,
                      text: x.title,
                      time: x.time,
                      href: `/console/buyer/notifications?focus=${encodeURIComponent(x.id)}`,
                    }))}
                  />
                </SectionCard>
              </motion.div>
              <motion.div layout data-flip-node className="xl:col-span-5">
                <SectionCard title="本周关键行动">
                  <div className="space-y-3 text-sm">
                    <Link href="/console/buyer/subscriptions" className="block rounded-xl border border-gray-200 p-3 hover:border-primary-300 hover:bg-primary-50/40">
                      <p className="font-medium text-gray-900">检查即将到期订阅</p>
                      <p className="text-gray-600 mt-1">2 个订阅将在 30 天内到期，建议尽快续订。</p>
                    </Link>
                    <Link href="/console/buyer/requests" className="block rounded-xl border border-gray-200 p-3 hover:border-primary-300 hover:bg-primary-50/40">
                      <p className="font-medium text-gray-900">处理待补充申请</p>
                      <p className="text-gray-600 mt-1">1 个申请等待补充材料，可能影响交付上线。</p>
                    </Link>
                    <Link href="/console/buyer/orders" className="block rounded-xl border border-gray-200 p-3 hover:border-primary-300 hover:bg-primary-50/40">
                      <p className="font-medium text-gray-900">核对账单与发票</p>
                      <p className="text-gray-600 mt-1">建议每周核对一次订单账单与发票状态。</p>
                    </Link>
                  </div>
                </SectionCard>
              </motion.div>
            </motion.section>
          </div>
        </LayoutGroup>

        <motion.section initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.05, duration: 0.2 }} className="rounded-2xl border border-dashed border-gray-300 bg-white px-6 py-4">
          <div className="flex flex-wrap items-center gap-5 text-sm text-gray-600">
            <div className="inline-flex items-center gap-2"><Calendar className="w-4 h-4" /><span>数据更新时间：每 5 分钟（模拟）</span></div>
            <div className="inline-flex items-center gap-2"><Activity className="w-4 h-4" /><span>图表已接入 ECharts，可直接替换 API 数据</span></div>
            <div className="inline-flex items-center gap-2"><Sparkles className="w-4 h-4" /><span>当前为前端框架模式，未接后端鉴权与链路服务</span></div>
          </div>
        </motion.section>
      </div>
    </>
  )
}
