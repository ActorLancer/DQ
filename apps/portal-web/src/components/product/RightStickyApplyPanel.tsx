'use client'

import { useState } from 'react'
import { Heart, GitCompare, MessageCircle, Shield } from 'lucide-react'
import type { Listing } from '@/types'

interface RightStickyApplyPanelProps {
  listing: Listing
  onApply: () => void
}

export default function RightStickyApplyPanel({ listing, onApply }: RightStickyApplyPanelProps) {
  const [isFavorited, setIsFavorited] = useState(false)
  const [selectedPlan, setSelectedPlan] = useState(listing.pricingPlans[0]?.id || '')

  const currentPlan = listing.pricingPlans.find((p) => p.id === selectedPlan)

  return (
    <div className="sticky top-24 bg-white rounded-xl border-2 border-gray-200 p-6 shadow-lg">
      {/* 价格摘要 */}
      <div className="mb-6">
        <div className="text-sm text-gray-500 mb-1">起步价</div>
        <div className="text-3xl font-bold text-primary-600">
          {currentPlan?.price ? `¥${currentPlan.price}` : '面议'}
        </div>
        {currentPlan?.pricingModel && (
          <div className="text-sm text-gray-600 mt-1">
            {currentPlan.pricingModel === 'MONTHLY' && '/ 月'}
            {currentPlan.pricingModel === 'YEARLY' && '/ 年'}
            {currentPlan.pricingModel === 'USAGE_BASED' && '/ 次'}
          </div>
        )}
      </div>

      {/* 套餐选择 */}
      {listing.pricingPlans.length > 1 && (
        <div className="mb-6">
          <label className="text-sm font-medium text-gray-700 mb-2 block">选择套餐</label>
          <select
            value={selectedPlan}
            onChange={(e) => setSelectedPlan(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          >
            {listing.pricingPlans.map((plan) => (
              <option key={plan.id} value={plan.id}>
                {plan.name} - {plan.price ? `¥${plan.price}` : '面议'}
              </option>
            ))}
          </select>
        </div>
      )}

      {/* 交付方式 */}
      <div className="mb-6">
        <div className="text-sm font-medium text-gray-700 mb-2">交付方式</div>
        <div className="flex flex-wrap gap-2">
          {currentPlan?.deliveryMethods.map((method) => (
            <span key={method} className="tag-primary">
              {method}
            </span>
          ))}
        </div>
      </div>

      {/* 授权期限 */}
      {currentPlan?.durationDays && (
        <div className="mb-6">
          <div className="text-sm font-medium text-gray-700 mb-2">授权期限</div>
          <div className="text-sm text-gray-900">{currentPlan.durationDays} 天</div>
        </div>
      )}

      {/* 调用额度 */}
      {currentPlan?.quota && (
        <div className="mb-6">
          <div className="text-sm font-medium text-gray-700 mb-2">调用额度</div>
          <div className="text-sm text-gray-900">{currentPlan.quota.toLocaleString()} 次</div>
        </div>
      )}

      {/* 试用支持 */}
      {listing.trialSupported && (
        <div className="mb-6 p-3 bg-green-50 border border-green-200 rounded-lg">
          <div className="flex items-center gap-2 text-success-600 text-sm font-medium">
            <Shield className="w-4 h-4" />
            <span>支持免费试用</span>
          </div>
        </div>
      )}

      {/* 主要操作按钮 */}
      <div className="space-y-3 mb-6">
        <button onClick={onApply} className="w-full btn-primary">
          申请访问
        </button>
        <button className="w-full btn-secondary">
          <MessageCircle className="w-4 h-4 mr-2" />
          联系供应商
        </button>
      </div>

      {/* 次要操作 */}
      <div className="flex gap-2">
        <button
          onClick={() => setIsFavorited(!isFavorited)}
          className={`flex-1 flex items-center justify-center gap-2 px-4 py-2 border-2 rounded-lg transition-colors ${
            isFavorited
              ? 'border-red-500 text-red-500 bg-red-50'
              : 'border-gray-300 text-gray-700 hover:border-gray-400'
          }`}
        >
          <Heart className={`w-4 h-4 ${isFavorited ? 'fill-red-500' : ''}`} />
          <span className="text-sm font-medium">{isFavorited ? '已收藏' : '收藏'}</span>
        </button>
        <button className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border-2 border-gray-300 text-gray-700 rounded-lg hover:border-gray-400 transition-colors">
          <GitCompare className="w-4 h-4" />
          <span className="text-sm font-medium">对比</span>
        </button>
      </div>

      {/* 供应商响应时间 */}
      <div className="mt-6 pt-6 border-t border-gray-200">
        <div className="flex justify-between text-sm">
          <span className="text-gray-600">供应商响应时间</span>
          <span className="font-medium text-gray-900">2 小时内</span>
        </div>
      </div>
    </div>
  )
}
