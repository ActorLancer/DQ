'use client'

import { useMemo, useState, type ComponentType } from 'react'
import { motion } from 'framer-motion'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import BuyerTrafficHeatmapChart from '@/components/charts/BuyerTrafficHeatmapChart'
import BuyerErrorBreakdownChart from '@/components/charts/BuyerErrorBreakdownChart'
import BuyerQuotaForecastChart from '@/components/charts/BuyerQuotaForecastChart'
import BuyerLatencyPercentileChart from '@/components/charts/BuyerLatencyPercentileChart'
import { Activity, AlertTriangle, Clock3, Download, Gauge, Layers, Sparkles } from 'lucide-react'

type Period = '7d' | '30d' | '90d'

interface EndpointPerf {
  endpoint: string
  p95: number
  success: number
  qps: number
  trend: 'up' | 'down' | 'flat'
}

const ENDPOINTS: EndpointPerf[] = [
  { endpoint: '/v1/risk/profile', p95: 312, success: 99.32, qps: 52, trend: 'up' },
  { endpoint: '/v1/enterprise/search', p95: 278, success: 99.61, qps: 46, trend: 'flat' },
  { endpoint: '/v1/legal/cases', p95: 348, success: 98.72, qps: 31, trend: 'down' },
  { endpoint: '/v1/credit/insight', p95: 221, success: 99.85, qps: 23, trend: 'up' },
  { endpoint: '/v1/person/relation', p95: 401, success: 98.12, qps: 18, trend: 'down' },
]

export default function BuyerUsagePage() {
  const [period, setPeriod] = useState<Period>('30d')
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const metrics = useMemo(() => {
    if (period === '7d') {
      return { uniqueApis: 23, p95Latency: 286, errBudget: 82, saturation: 64 }
    }
    if (period === '90d') {
      return { uniqueApis: 39, p95Latency: 334, errBudget: 68, saturation: 88 }
    }
    return { uniqueApis: 31, p95Latency: 302, errBudget: 76, saturation: 79 }
  }, [period])

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:usage:read"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8 space-y-6">
        <motion.section
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.24, ease: 'easeOut' }}
          className="relative overflow-hidden rounded-[22px] border border-slate-200 bg-gradient-to-br from-slate-50 via-white to-slate-100 px-7 py-6"
        >
          <div className="absolute inset-0 opacity-[0.06]" style={{ backgroundImage: 'radial-gradient(#64748b .6px, transparent .6px)', backgroundSize: '10px 10px' }} />
          <div className="relative flex flex-wrap items-end justify-between gap-3">
            <div>
              <h1 className="text-[30px] font-bold tracking-tight text-slate-900">Usage Intelligence Studio</h1>
              <p className="mt-2 text-[15px] text-slate-600">关注调用结构、错误预算、容量风险与端点性能，不再与仪表盘重复。</p>
            </div>
            <div className="flex items-center gap-2">
              <div className="inline-flex rounded-xl border border-slate-300 bg-white p-1">
                {(['7d', '30d', '90d'] as const).map((p) => (
                  <button key={p} onClick={() => setPeriod(p)} className={`px-3 py-1.5 text-sm rounded-lg transition ${period === p ? 'bg-slate-900 text-white' : 'text-slate-700 hover:bg-slate-100'}`}>{p}</button>
                ))}
              </div>
              <button className="inline-flex h-10 items-center gap-2 rounded-lg border border-slate-300 bg-white px-3 text-sm text-slate-700 hover:bg-slate-50">
                <Download className="h-4 w-4" />导出
              </button>
            </div>
          </div>
        </motion.section>

        <motion.section
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.05, duration: 0.22 }}
          className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-4"
        >
          <MetricCard icon={Layers} label="活跃 API 端点" value={`${metrics.uniqueApis}`} sub="较上周期 +4" tone="slate" />
          <MetricCard icon={Clock3} label="P95 响应时延" value={`${metrics.p95Latency}ms`} sub="波动区间收敛" tone="blue" />
          <MetricCard icon={AlertTriangle} label="错误预算剩余" value={`${metrics.errBudget}%`} sub="需重点关注 5xx" tone="amber" />
          <MetricCard icon={Gauge} label="容量饱和度" value={`${metrics.saturation}%`} sub="下周可能触发预警" tone="rose" />
        </motion.section>

        <section className="grid grid-cols-1 gap-5 xl:grid-cols-12">
          <motion.article
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.08, duration: 0.22 }}
            className="xl:col-span-7 rounded-2xl border border-slate-200 bg-white p-5 shadow-[0_12px_36px_-28px_rgba(15,23,42,0.45)]"
          >
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-lg font-semibold text-slate-900">调用活跃热力图（小时 × 星期）</h2>
              <Activity className="h-4 w-4 text-slate-400" />
            </div>
            <div className="h-[310px]"><BuyerTrafficHeatmapChart /></div>
          </motion.article>

          <motion.article
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1, duration: 0.22 }}
            className="xl:col-span-5 rounded-2xl border border-slate-200 bg-white p-5 shadow-[0_12px_36px_-28px_rgba(15,23,42,0.45)]"
          >
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-lg font-semibold text-slate-900">错误结构与来源</h2>
              <AlertTriangle className="h-4 w-4 text-slate-400" />
            </div>
            <div className="h-[310px]"><BuyerErrorBreakdownChart /></div>
          </motion.article>
        </section>

        <section className="grid grid-cols-1 gap-5 xl:grid-cols-12">
          <motion.div
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.12, duration: 0.22 }}
            className="xl:col-span-5 grid grid-cols-1 gap-5"
          >
            <article className="rounded-2xl border border-slate-200 bg-white p-5 shadow-[0_12px_36px_-28px_rgba(15,23,42,0.45)]">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-lg font-semibold text-slate-900">配额消耗预测</h2>
                <Gauge className="h-4 w-4 text-slate-400" />
              </div>
              <div className="h-[190px]"><BuyerQuotaForecastChart /></div>
            </article>

            <article className="rounded-2xl border border-slate-200 bg-white p-5 shadow-[0_12px_36px_-28px_rgba(15,23,42,0.45)]">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-lg font-semibold text-slate-900">时延分位结构</h2>
                <Clock3 className="h-4 w-4 text-slate-400" />
              </div>
              <div className="h-[190px]"><BuyerLatencyPercentileChart /></div>
            </article>
          </motion.div>

          <motion.article
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.14, duration: 0.22 }}
            className="xl:col-span-7 rounded-2xl border border-slate-200 bg-white p-5 shadow-[0_12px_36px_-28px_rgba(15,23,42,0.45)]"
          >
            <div className="mb-4 flex items-center justify-between">
              <h2 className="text-lg font-semibold text-slate-900">端点性能剖析（Top 5）</h2>
              <Sparkles className="h-4 w-4 text-slate-400" />
            </div>
            <div className="space-y-3">
              {ENDPOINTS.map((ep, i) => (
                <motion.div
                  key={ep.endpoint}
                  initial={{ opacity: 0, x: -8 }}
                  animate={{ opacity: 1, x: 0 }}
                  transition={{ delay: 0.18 + i * 0.03, duration: 0.2 }}
                  className="rounded-xl border border-slate-200 bg-slate-50/70 p-3"
                >
                  <div className="mb-2 flex items-center justify-between">
                    <p className="text-sm font-semibold text-slate-900">{ep.endpoint}</p>
                    <span className={`text-xs font-medium ${ep.trend === 'up' ? 'text-emerald-600' : ep.trend === 'down' ? 'text-rose-600' : 'text-slate-500'}`}>
                      {ep.trend === 'up' ? '趋势改善' : ep.trend === 'down' ? '趋势下降' : '基本稳定'}
                    </span>
                  </div>
                  <div className="grid grid-cols-3 gap-3 text-sm">
                    <StatPill label="P95" value={`${ep.p95}ms`} />
                    <StatPill label="成功率" value={`${ep.success}%`} />
                    <StatPill label="QPS" value={`${ep.qps}`} />
                  </div>
                  <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-slate-200">
                    <motion.div
                      initial={{ width: 0 }}
                      animate={{ width: `${Math.min(100, Math.round((520 - ep.p95) / 5.2))}%` }}
                      transition={{ delay: 0.2 + i * 0.04, duration: 0.45, ease: 'easeOut' }}
                      className="h-full rounded-full bg-slate-900"
                    />
                  </div>
                </motion.div>
              ))}
            </div>
          </motion.article>
        </section>
      </div>
    </>
  )
}

function MetricCard({ icon: Icon, label, value, sub, tone }: { icon: ComponentType<{ className?: string }>; label: string; value: string; sub: string; tone: 'slate' | 'blue' | 'amber' | 'rose' }) {
  const toneCls = tone === 'slate'
    ? 'bg-slate-50 text-slate-700'
    : tone === 'blue'
      ? 'bg-blue-50 text-blue-700'
      : tone === 'amber'
        ? 'bg-amber-50 text-amber-700'
        : 'bg-rose-50 text-rose-700'

  return (
    <article className="rounded-2xl border border-slate-200 bg-white p-5 shadow-[0_12px_30px_-28px_rgba(15,23,42,0.45)]">
      <div className="mb-3 flex items-center justify-between">
        <div className={`inline-flex h-11 w-11 items-center justify-center rounded-lg ${toneCls}`}><Icon className="h-5 w-5" /></div>
      </div>
      <p className="text-sm text-slate-600">{label}</p>
      <p className="mt-1 text-2xl font-bold tracking-tight text-slate-900">{value}</p>
      <p className="mt-1 text-xs text-slate-500">{sub}</p>
    </article>
  )
}

function StatPill({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg border border-slate-200 bg-white px-2 py-1.5">
      <p className="text-[11px] text-slate-500">{label}</p>
      <p className="text-sm font-semibold text-slate-900">{value}</p>
    </div>
  )
}
