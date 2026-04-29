'use client'

import { Grid, List } from 'lucide-react'

const SORT_OPTIONS = [
  { value: 'comprehensive', label: '综合排序' },
  { value: 'latest', label: '最新发布' },
  { value: 'quality_desc', label: '质量评分最高' },
  { value: 'calls_desc', label: '调用量最高' },
  { value: 'transactions_desc', label: '成交量最高' },
  { value: 'price_asc', label: '价格从低到高' },
  { value: 'price_desc', label: '价格从高到低' },
]

interface SortToolbarProps {
  total: number
  currentSort: string
  viewMode: 'grid' | 'list'
  onSortChange: (sort: string) => void
  onViewModeChange: (mode: 'grid' | 'list') => void
}

export default function SortToolbar({
  total,
  currentSort,
  viewMode,
  onSortChange,
  onViewModeChange,
}: SortToolbarProps) {
  return (
    <div className="flex items-center justify-between bg-white rounded-lg border border-gray-200 p-4 mb-6">
      {/* 左侧：结果数量 */}
      <div className="text-sm text-gray-600">
        共找到 <span className="font-bold text-gray-900">{total}</span> 个数据商品
      </div>

      {/* 中间：排序 */}
      <div className="flex items-center gap-2">
        <span className="text-sm text-gray-600">排序:</span>
        <select
          value={currentSort}
          onChange={(e) => onSortChange(e.target.value)}
          className="px-3 py-1.5 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
        >
          {SORT_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
      </div>

      {/* 右侧：视图切换 */}
      <div className="flex items-center gap-1 bg-gray-100 rounded-lg p-1">
        <button
          onClick={() => onViewModeChange('grid')}
          className={`p-2 rounded ${
            viewMode === 'grid'
              ? 'bg-white text-primary-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900'
          }`}
        >
          <Grid className="w-4 h-4" />
        </button>
        <button
          onClick={() => onViewModeChange('list')}
          className={`p-2 rounded ${
            viewMode === 'list'
              ? 'bg-white text-primary-600 shadow-sm'
              : 'text-gray-600 hover:text-gray-900'
          }`}
        >
          <List className="w-4 h-4" />
        </button>
      </div>
    </div>
  )
}
