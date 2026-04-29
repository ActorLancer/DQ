'use client'

import Link from 'next/link'
import { 
  Building2, 
  Landmark, 
  Heart, 
  Factory, 
  Truck, 
  ShoppingBag, 
  Briefcase, 
  Zap,
  MoreHorizontal 
} from 'lucide-react'

const INDUSTRIES = [
  { id: 'finance', name: '金融', icon: Landmark, color: 'bg-blue-500' },
  { id: 'government', name: '政务', icon: Building2, color: 'bg-red-500' },
  { id: 'healthcare', name: '医疗', icon: Heart, color: 'bg-green-500' },
  { id: 'manufacturing', name: '工业', icon: Factory, color: 'bg-gray-500' },
  { id: 'logistics', name: '交通', icon: Truck, color: 'bg-yellow-500' },
  { id: 'retail', name: '消费', icon: ShoppingBag, color: 'bg-pink-500' },
  { id: 'enterprise', name: '企业服务', icon: Briefcase, color: 'bg-purple-500' },
  { id: 'energy', name: '能源', icon: Zap, color: 'bg-orange-500' },
]

export default function IndustryCategoryGrid() {
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4">
      {INDUSTRIES.map((industry) => {
        const Icon = industry.icon
        return (
          <Link
            key={industry.id}
            href={`/marketplace?industry=${industry.id}`}
            className="group flex flex-col items-center p-6 bg-white rounded-xl border-2 border-gray-200 hover:border-primary-500 hover:shadow-lg transition-all"
          >
            <div className={`w-16 h-16 ${industry.color} rounded-xl flex items-center justify-center mb-3 group-hover:scale-110 transition-transform`}>
              <Icon className="w-8 h-8 text-white" />
            </div>
            <span className="text-sm font-medium text-gray-900 group-hover:text-primary-600">
              {industry.name}
            </span>
          </Link>
        )
      })}
      
      <Link
        href="/marketplace"
        className="group flex flex-col items-center p-6 bg-white rounded-xl border-2 border-gray-200 hover:border-primary-500 hover:shadow-lg transition-all"
      >
        <div className="w-16 h-16 bg-gray-200 rounded-xl flex items-center justify-center mb-3 group-hover:scale-110 transition-transform">
          <MoreHorizontal className="w-8 h-8 text-gray-600" />
        </div>
        <span className="text-sm font-medium text-gray-900 group-hover:text-primary-600">
          更多
        </span>
      </Link>
    </div>
  )
}
