import { ReactNode } from 'react'

export default function SectionCard({
  title,
  right,
  children,
  className = '',
}: {
  title: string
  right?: ReactNode
  children: ReactNode
  className?: string
}) {
  return (
    <section className={`rounded-2xl border border-gray-200/90 bg-white/95 p-5 shadow-sm backdrop-blur-[1px] transition-all duration-200 hover:shadow-md ${className}`}>
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-lg font-semibold tracking-tight text-gray-900">{title}</h2>
        {right}
      </div>
      {children}
    </section>
  )
}
