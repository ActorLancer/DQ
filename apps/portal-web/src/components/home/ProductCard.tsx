'use client'

import Link from 'next/link'
import { Shield, Star, TrendingUp, Clock } from 'lucide-react'
import type { Listing } from '@/types'

interface ProductCardProps {
  product: Listing
}

export default function ProductCard({ product }: ProductCardProps) {
  return (
    <Link
      href={`/products/${product.id}`}
      className="product-card block bg-white rounded-xl border border-gray-200 p-6 hover:shadow-lg transition-shadow"
    >
      {/* 头部 */}
      <div className="flex items-start justify-between mb-3">
        <h3 className="text-lg font-bold text-gray-900 line-clamp-2 flex-1">
          {product.title}
        </h3>
        {product.chainRegistered && (
          <div className="chain-badge ml-2 flex-shrink-0">
            <Shield className="w-3 h-3" />
            <span>链上存证</span>
          </div>
        )}
      </div>

      {/* 简介 */}
      <p className="text-sm text-gray-600 line-clamp-2 mb-4">
        {product.summary}
      </p>

      {/* 供应商 */}
      <div className="text-sm text-gray-700 mb-4">
        <span className="font-medium">{product.supplierName}</span>
      </div>

      {/* 标签 */}
      <div className="flex flex-wrap gap-2 mb-4">
        <span className="tag">{product.industry}</span>
        <span className="tag">{product.dataType}</span>
        {product.deliveryMethods[0] && (
          <span className="tag-primary">{product.deliveryMethods[0]}</span>
        )}
      </div>

      {/* 底部信息 */}
      <div className="flex items-center justify-between pt-4 border-t border-gray-100">
        <div className="flex items-center gap-4 text-sm text-gray-600">
          <div className="flex items-center gap-1">
            <Star className="w-4 h-4 text-yellow-500 fill-yellow-500" />
            <span className="font-medium">{product.qualityScore.toFixed(1)}</span>
          </div>
          {product.trialSupported && (
            <span className="text-success-600 font-medium">支持试用</span>
          )}
        </div>
        
        {product.pricingPlans[0] && (
          <div className="text-right">
            <div className="text-sm text-gray-500">起步价</div>
            <div className="text-lg font-bold text-primary-600">
              {product.pricingPlans[0].price 
                ? `¥${product.pricingPlans[0].price}` 
                : '面议'}
            </div>
          </div>
        )}
      </div>

      {/* 合规标签 */}
      {product.complianceTags.length > 0 && (
        <div className="mt-3 pt-3 border-t border-gray-100">
          <div className="flex flex-wrap gap-1">
            {product.complianceTags.slice(0, 3).map((tag) => (
              <span key={tag} className="text-xs px-2 py-0.5 bg-green-50 text-green-700 rounded">
                {tag}
              </span>
            ))}
          </div>
        </div>
      )}
    </Link>
  )
}
