import { LucideIcon, TrendingUp } from 'lucide-react'
import MetricCountUp from './MetricCountUp'

export default function KpiCard({
  label,
  value,
  delta,
  icon: Icon,
  tone = 'blue',
}: {
  label: string
  value: string
  delta?: string
  icon: LucideIcon
  tone?: string
}) {
  const toneMap: Record<string, string> = {
    blue: 'bg-blue-50 text-blue-700 border-blue-100',
    amber: 'bg-amber-50 text-amber-700 border-amber-100',
    emerald: 'bg-emerald-50 text-emerald-700 border-emerald-100',
    indigo: 'bg-indigo-50 text-indigo-700 border-indigo-100',
    red: 'bg-red-50 text-red-700 border-red-100',
    slate: 'bg-slate-50 text-slate-700 border-slate-100',
  }

  return (
    <article className="group rounded-2xl border border-gray-200 bg-white p-5 shadow-sm transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md">
      <div className="mb-4 flex items-start justify-between">
        <span className={`inline-flex h-11 w-11 items-center justify-center rounded-xl border transition-transform duration-200 group-hover:scale-105 ${toneMap[tone] || toneMap.blue}`}>
          <Icon className="h-5 w-5" />
        </span>
        {delta && (
          <span className="inline-flex items-center gap-1 text-xs font-medium text-emerald-600">
            <TrendingUp className="h-3.5 w-3.5" />
            {delta}
          </span>
        )}
      </div>
      <div className="mb-1 text-sm font-medium text-gray-600">{label}</div>
      <div className="text-[30px] leading-9 font-semibold tracking-tight text-gray-900"><MetricCountUp value={value} /></div>
    </article>
  )
}
