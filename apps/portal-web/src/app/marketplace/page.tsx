'use client'

import { useState, useEffect, Suspense } from 'react'
import { useSearchParams, useRouter } from 'next/navigation'
import Header from '@/components/layout/Header'
import Footer from '@/components/layout/Footer'
import TopSearchBar from '@/components/marketplace/TopSearchBar'
import LeftFilterPanel from '@/components/marketplace/LeftFilterPanel'
import SortToolbar from '@/components/marketplace/SortToolbar'
import ProductCard from '@/components/home/ProductCard'
import type { Listing } from '@/types'

// Mock 数据
const MOCK_PRODUCTS: Listing[] = [
  {
    id: 'listing_001',
    title: '企业工商风险数据',
    summary: '覆盖全国 5000 万+企业的工商信息、司法风险、经营异常等多维度数据',
    supplierId: 'supplier_001',
    supplierName: '天眼数据科技',
    industry: '金融',
    dataType: '企业征信',
    deliveryMethods: ['API', 'FILE'],
    licenseTypes: ['COMMERCIAL', 'SUBSCRIPTION'],
    pricingPlans: [
      {
        id: 'plan_001',
        listingId: 'listing_001',
        name: '标准版',
        pricingModel: 'MONTHLY',
        price: 9999,
        currency: 'CNY',
        quota: 10000,
        durationDays: 30,
        deliveryMethods: ['API'],
      },
    ],
    qualityScore: 9.2,
    complianceTags: ['数据安全认证', '隐私保护', '合规审查'],
    chainRegistered: true,
    status: 'LISTED',
    trialSupported: true,
    createdAt: '2026-04-01T00:00:00Z',
    updatedAt: '2026-04-20T00:00:00Z',
  },
  // ... 添加更多 mock 数据
]

function MarketplaceContent() {
  const searchParams = useSearchParams()
  const router = useRouter()
  
  const [keyword, setKeyword] = useState(searchParams.get('keyword') || '')
  const [selectedFilters, setSelectedFilters] = useState<Record<string, string[]>>({})
  const [currentSort, setCurrentSort] = useState(searchParams.get('sort') || 'comprehensive')
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid')
  const [isLoading, setIsLoading] = useState(false)

  // 更新 URL
  const updateURL = (params: Record<string, any>) => {
    const newParams = new URLSearchParams()
    
    if (params.keyword) newParams.set('keyword', params.keyword)
    if (params.sort) newParams.set('sort', params.sort)
    
    Object.entries(params.filters || {}).forEach(([key, values]) => {
      if (Array.isArray(values) && values.length > 0) {
        newParams.set(key, values.join(','))
      }
    })

    router.push(`/marketplace?${newParams.toString()}`, { scroll: false })
  }

  const handleSearch = (newKeyword: string) => {
    setKeyword(newKeyword)
    updateURL({ keyword: newKeyword, sort: currentSort, filters: selectedFilters })
  }

  const handleFilterChange = (filterId: string, values: string[]) => {
    if (filterId === 'reset') {
      setSelectedFilters({})
      updateURL({ keyword, sort: currentSort, filters: {} })
    } else {
      const newFilters = { ...selectedFilters, [filterId]: values }
      setSelectedFilters(newFilters)
      updateURL({ keyword, sort: currentSort, filters: newFilters })
    }
  }

  const handleSortChange = (sort: string) => {
    setCurrentSort(sort)
    updateURL({ keyword, sort, filters: selectedFilters })
  }

  return (
    <div className="min-h-screen bg-gray-50">
      <Header />

      <div className="container-custom py-8">
        {/* 顶部搜索 */}
        <TopSearchBar initialKeyword={keyword} onSearch={handleSearch} />

        {/* 主体布局 */}
        <div className="flex gap-6">
          {/* 左侧筛选 */}
          <aside className="w-64 flex-shrink-0">
            <div className="sticky top-20">
              <LeftFilterPanel
                selectedFilters={selectedFilters}
                onFilterChange={handleFilterChange}
              />
            </div>
          </aside>

          {/* 右侧结果 */}
          <main className="flex-1">
            <SortToolbar
              total={MOCK_PRODUCTS.length}
              currentSort={currentSort}
              viewMode={viewMode}
              onSortChange={handleSortChange}
              onViewModeChange={setViewMode}
            />

            {/* 加载状态 */}
            {isLoading && (
              <div className="relative">
                <div className="absolute inset-0 bg-white/50 z-10 flex items-center justify-center">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
                </div>
              </div>
            )}

            {/* 商品列表 */}
            {viewMode === 'grid' ? (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                {MOCK_PRODUCTS.map((product) => (
                  <ProductCard key={product.id} product={product} />
                ))}
              </div>
            ) : (
              <div className="space-y-4">
                {MOCK_PRODUCTS.map((product) => (
                  <ProductCard key={product.id} product={product} />
                ))}
              </div>
            )}

            {/* 空状态 */}
            {MOCK_PRODUCTS.length === 0 && (
              <div className="text-center py-16">
                <div className="text-gray-400 mb-4">
                  <svg
                    className="w-16 h-16 mx-auto"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                </div>
                <h3 className="text-lg font-medium text-gray-900 mb-2">未找到相关商品</h3>
                <p className="text-gray-600 mb-6">尝试调整筛选条件或清空筛选</p>
                <button
                  onClick={() => handleFilterChange('reset', [])}
                  className="btn-primary"
                >
                  清空筛选
                </button>
              </div>
            )}

            {/* 分页 */}
            {MOCK_PRODUCTS.length > 0 && (
              <div className="flex items-center justify-center gap-2 mt-8">
                <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed">
                  上一页
                </button>
                <button className="px-4 py-2 bg-primary-600 text-white rounded-lg">1</button>
                <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
                  2
                </button>
                <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
                  3
                </button>
                <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
                  下一页
                </button>
              </div>
            )}
          </main>
        </div>
      </div>

      <Footer />
    </div>
  )
}

export default function MarketplacePage() {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <MarketplaceContent />
    </Suspense>
  )
}
