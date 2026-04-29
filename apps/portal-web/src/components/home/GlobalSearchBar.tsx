'use client'

import { useState } from 'react'
import { Search } from 'lucide-react'
import { useRouter } from 'next/navigation'

const HOT_KEYWORDS = ['企业风险', '金融数据', '消费行为', '物流轨迹', '医疗健康']

export default function GlobalSearchBar() {
  const [keyword, setKeyword] = useState('')
  const router = useRouter()

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    if (keyword.trim()) {
      router.push(`/marketplace?keyword=${encodeURIComponent(keyword.trim())}`)
    }
  }

  const handleHotKeyword = (word: string) => {
    router.push(`/marketplace?keyword=${encodeURIComponent(word)}`)
  }

  return (
    <div className="w-full max-w-3xl mx-auto">
      <form onSubmit={handleSearch} className="relative">
        <div className="relative">
          <input
            type="text"
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            placeholder="搜索数据商品、供应商、行业..."
            className="w-full h-14 pl-6 pr-14 text-lg border-2 border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent shadow-lg"
          />
          <button
            type="submit"
            className="absolute right-2 top-1/2 -translate-y-1/2 w-10 h-10 bg-primary-600 hover:bg-primary-700 active:bg-primary-800 rounded-lg flex items-center justify-center transition-colors"
          >
            <Search className="w-5 h-5 text-white" />
          </button>
        </div>
      </form>

      {/* 热门搜索 */}
      <div className="mt-4 flex items-center gap-3 flex-wrap">
        <span className="text-sm text-gray-600">热门搜索:</span>
        {HOT_KEYWORDS.map((word) => (
          <button
            key={word}
            onClick={() => handleHotKeyword(word)}
            className="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 active:bg-gray-300 text-gray-700 rounded-full transition-colors"
          >
            {word}
          </button>
        ))}
      </div>
    </div>
  )
}
