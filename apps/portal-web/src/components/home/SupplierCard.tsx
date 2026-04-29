'use client'

import Link from 'next/link'
import { Building2, CheckCircle, TrendingUp, Clock } from 'lucide-react'
import type { Supplier } from '@/types'

interface SupplierCardProps {
  supplier: Supplier
}

export default function SupplierCard({ supplier }: SupplierCardProps) {
  return (
    <Link
      href={`/suppliers/${supplier.id}`}
      className="block bg-white rounded-xl border border-gray-200 p-6 hover:shadow-lg transition-shadow"
    >
      {/* 头部 */}
      <div className="flex items-start gap-4 mb-4">
        <div className="w-16 h-16 bg-gradient-to-br from-primary-500 to-primary-700 rounded-xl flex items-center justify-center flex-shrink-0">
          <Building2 className="w-8 h-8 text-white" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <h3 className="text-lg font-bold text-gray-900 truncate">
              {supplier.name}
            </h3>
            {supplier.certificationStatus === 'APPROVED' && (
              <CheckCircle className="w-5 h-5 text-success-600 flex-shrink-0" />
            )}
          </div>
          {supplier.tier && (
            <span className="inline-block px-2 py-0.5 bg-yellow-100 text-yellow-800 text-xs font-medium rounded">
              {supplier.tier}
            </span>
          )}
        </div>
      </div>

      {/* 统计信息 */}
      <div className="grid grid-cols-2 gap-4 mb-4">
        {supplier.totalTransactions !== undefined && (
          <div>
            <div className="text-xs text-gray-500 mb-1">历史成交</div>
            <div className="flex items-center gap-1">
              <TrendingUp className="w-4 h-4 text-primary-600" />
              <span className="text-sm font-bold text-gray-900">
                {supplier.totalTransactions}
              </span>
            </div>
          </div>
        )}
        {supplier.responseTime && (
          <div>
            <div className="text-xs text-gray-500 mb-1">响应时效</div>
            <div className="flex items-center gap-1">
              <Clock className="w-4 h-4 text-success-600" />
              <span className="text-sm font-bold text-gray-900">
                {supplier.responseTime}
              </span>
            </div>
          </div>
        )}
      </div>

      {/* 合规认证 */}
      {supplier.complianceCertifications && supplier.complianceCertifications.length > 0 && (
        <div className="pt-4 border-t border-gray-100">
          <div className="text-xs text-gray-500 mb-2">合规认证</div>
          <div className="flex flex-wrap gap-1">
            {supplier.complianceCertifications.slice(0, 3).map((cert) => (
              <span key={cert} className="text-xs px-2 py-0.5 bg-green-50 text-green-700 rounded">
                {cert}
              </span>
            ))}
          </div>
        </div>
      )}
    </Link>
  )
}
