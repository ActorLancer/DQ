'use client'

import { useState } from 'react'
import { Search, X } from 'lucide-react'

interface TopSearchBarProps {
  initialKeyword?: string
  onSearch: (keyword: string) => void
}

export default function TopSearchBar({ initialKeyword = '', onSearch }: TopSearchBarProps) {
  const [keyword, setKeyword] = useState(initialKeyword)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSearch(keyword.trim())
  }

  const handleClear = () => {
    setKeyword('')
    onSearch('')
  }

  return (
    <form onSubmit={handleSubmit} className="relative mb-6">
      <div className="relative">
        <input
          type="text"
          value={keyword}
          onChange={(e) => setKeyword(e.target.value)}
          placeholder="搜索数据商品、供应商、行业..."
          className="w-full h-12 pl-12 pr-24 border-2 border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
        />
        <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
        
        {keyword && (
          <button
            type="button"
            onClick={handleClear}
            className="absolute right-14 top-1/2 -translate-y-1/2 p-1 text-gray-400 hover:text-gray-600"
          >
            <X className="w-4 h-4" />
          </button>
        )}
        
        <button
          type="submit"
          className="absolute right-2 top-1/2 -translate-y-1/2 px-4 py-1.5 bg-primary-600 hover:bg-primary-700 active:bg-primary-800 text-white rounded-lg transition-colors"
        >
          搜索
        </button>
      </div>
    </form>
  )
}
