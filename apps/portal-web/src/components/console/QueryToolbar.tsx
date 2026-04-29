'use client'

import { ReactNode } from 'react'
import { Search, RotateCcw } from 'lucide-react'

export function QueryToolbar({
  searchValue,
  onSearchChange,
  searchPlaceholder,
  onReset,
  controls,
  stats,
}: {
  searchValue: string
  onSearchChange: (value: string) => void
  searchPlaceholder: string
  onReset: () => void
  controls: ReactNode
  stats?: ReactNode
}) {
  return (
    <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6 space-y-4">
      <div className="flex items-center gap-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            value={searchValue}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={searchPlaceholder}
            className="w-full pl-10 pr-4 h-10 border border-gray-300 rounded-lg"
          />
        </div>
        <button
          onClick={onReset}
          className="h-10 px-3 inline-flex items-center gap-2 border border-gray-300 rounded-lg hover:bg-gray-50 text-sm"
        >
          <RotateCcw className="w-4 h-4" />重置
        </button>
      </div>
      {controls}
      {stats ? <div className="flex items-center gap-4 text-sm text-gray-600">{stats}</div> : null}
    </div>
  )
}

export function PaginationBar({
  page,
  pageSize,
  total,
  onPageChange,
  onPageSizeChange,
}: {
  page: number
  pageSize: number
  total: number
  onPageChange: (page: number) => void
  onPageSizeChange: (size: number) => void
}) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize))

  return (
    <div className="flex items-center justify-between rounded-xl border border-gray-200 bg-white px-4 py-3 mt-4">
      <div className="text-sm text-gray-600">共 {total} 条，第 {page}/{totalPages} 页</div>
      <div className="flex items-center gap-2">
        <select
          value={pageSize}
          onChange={(e) => onPageSizeChange(Number(e.target.value))}
          className="h-9 px-3 border border-gray-300 rounded-lg text-sm"
        >
          <option value={5}>每页 5 条</option>
          <option value={10}>每页 10 条</option>
          <option value={20}>每页 20 条</option>
        </select>
        <button
          disabled={page <= 1}
          onClick={() => onPageChange(page - 1)}
          className="h-9 px-3 border border-gray-300 rounded-lg text-sm disabled:opacity-40"
        >
          上一页
        </button>
        <button
          disabled={page >= totalPages}
          onClick={() => onPageChange(page + 1)}
          className="h-9 px-3 border border-gray-300 rounded-lg text-sm disabled:opacity-40"
        >
          下一页
        </button>
      </div>
    </div>
  )
}
