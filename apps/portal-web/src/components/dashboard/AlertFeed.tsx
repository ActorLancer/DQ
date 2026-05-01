import { AlertCircle } from 'lucide-react'
import Link from 'next/link'

export interface AlertItem {
  id: string
  level: 'info' | 'warning' | 'error' | 'success'
  text: string
  time: string
  href?: string
}

export default function AlertFeed({ items }: { items: AlertItem[] }) {
  return (
    <div className="space-y-3">
      {items.map((item) => {
        const cls =
          item.level === 'error'
            ? 'border-red-200 bg-red-50 text-red-900'
            : item.level === 'warning'
              ? 'border-amber-200 bg-amber-50 text-amber-900'
              : item.level === 'success'
                ? 'border-emerald-200 bg-emerald-50 text-emerald-900'
                : 'border-blue-200 bg-blue-50 text-blue-900'

        const iconColor =
          item.level === 'error'
            ? 'text-red-600'
            : item.level === 'warning'
              ? 'text-amber-600'
              : item.level === 'success'
                ? 'text-emerald-600'
                : 'text-blue-600'

        const content = (
          <div className="flex items-start gap-2">
            <AlertCircle className={`mt-0.5 h-4 w-4 flex-shrink-0 ${iconColor}`} />
            <div>
              <p className="text-sm">{item.text}</p>
              <p className="mt-1 text-xs text-gray-500">{item.time}</p>
            </div>
          </div>
        )

        if (item.href) {
          return (
            <Link
              key={item.id}
              href={item.href}
              className={`group block cursor-pointer rounded-xl border p-3 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 ${cls}`}
            >
              {content}
              <div className="mt-2 text-right text-xs text-primary-700">查看通知</div>
            </Link>
          )
        }

        return (
          <div key={item.id} className={`rounded-xl border p-3 transition-all duration-200 hover:-translate-y-0.5 hover:shadow-sm ${cls}`}>
            {content}
          </div>
        )
      })}
    </div>
  )
}
