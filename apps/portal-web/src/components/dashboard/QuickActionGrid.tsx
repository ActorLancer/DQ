import Link from 'next/link'
import { LucideIcon, ArrowUpRight } from 'lucide-react'

export interface QuickActionItem {
  id: string
  label: string
  href: string
  icon: LucideIcon
}

export default function QuickActionGrid({ items }: { items: QuickActionItem[] }) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
      {items.map((action) => {
        const Icon = action.icon
        return (
          <Link
            key={action.id}
            href={action.href}
            className="rounded-xl border border-gray-200 px-4 py-3 transition-colors hover:border-primary-300 hover:bg-primary-50/40"
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Icon className="h-4 w-4 text-primary-600" />
                <span className="text-sm font-medium text-gray-900">{action.label}</span>
              </div>
              <ArrowUpRight className="h-4 w-4 text-gray-400" />
            </div>
          </Link>
        )
      })}
    </div>
  )
}
