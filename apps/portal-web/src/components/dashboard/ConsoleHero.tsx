import { ReactNode } from 'react'

export default function ConsoleHero({
  title,
  subtitle,
  right,
  tone = 'from-slate-50 to-white',
}: {
  title: string
  subtitle: string
  right?: ReactNode
  tone?: string
}) {
  return (
    <section className={`relative overflow-hidden rounded-2xl border border-gray-200 bg-gradient-to-r ${tone} px-6 py-6 shadow-sm`}>
      <div className="pointer-events-none absolute -right-10 -top-10 h-36 w-36 rounded-full bg-white/50 blur-2xl" />
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight text-gray-900">{title}</h1>
          <p className="mt-2 text-[15px] text-gray-600">{subtitle}</p>
        </div>
        {right}
      </div>
    </section>
  )
}
