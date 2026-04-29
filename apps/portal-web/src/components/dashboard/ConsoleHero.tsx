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
    <section className={`rounded-2xl border border-gray-200 bg-gradient-to-r ${tone} px-6 py-6`}>
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">{title}</h1>
          <p className="mt-1 text-gray-600">{subtitle}</p>
        </div>
        {right}
      </div>
    </section>
  )
}
