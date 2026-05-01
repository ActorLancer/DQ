'use client'

import { ReactNode } from 'react'

interface ConsoleListPageShellProps {
  title: string
  subtitle: string
  headerAction?: ReactNode
  summaryCards?: ReactNode
  toolbar: ReactNode
  content: ReactNode
  pagination: ReactNode
}

export default function ConsoleListPageShell({
  title,
  subtitle,
  headerAction,
  summaryCards,
  toolbar,
  content,
  pagination,
}: ConsoleListPageShellProps) {
  return (
    <div className="p-8">
      <div className="mb-8 flex items-center justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 mb-2">{title}</h1>
          <p className="text-gray-600">{subtitle}</p>
        </div>
        {headerAction ? <div className="shrink-0">{headerAction}</div> : null}
      </div>

      {summaryCards ? <div className="mb-6">{summaryCards}</div> : null}

      {toolbar}

      <div className="space-y-5">{content}</div>

      {pagination}
    </div>
  )
}
