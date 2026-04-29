'use client'

import Link from 'next/link'
import { Upload, ShoppingCart, Shield } from 'lucide-react'

const FLOWS = [
  {
    id: 'supplier',
    title: '供应商发布链路',
    description: '创建数据商品 → 平台审核 → 挂牌市场 → 处理申请 → 交付数据',
    icon: Upload,
    color: 'from-blue-500 to-blue-700',
    href: '/standard-flow/supplier',
  },
  {
    id: 'buyer',
    title: '买方采购链路',
    description: '搜索商品 → 申请访问 → 等待审批 → 获取授权 → 使用数据',
    icon: ShoppingCart,
    color: 'from-green-500 to-green-700',
    href: '/standard-flow/buyer',
  },
  {
    id: 'platform',
    title: '平台审核链路',
    description: '主体审核 → 商品审核 → 风险审计 → 争议处理 → 系统监控',
    icon: Shield,
    color: 'from-purple-500 to-purple-700',
    href: '/standard-flow/platform',
  },
]

export default function StandardFlowEntrance() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
      {FLOWS.map((flow) => {
        const Icon = flow.icon
        return (
          <Link
            key={flow.id}
            href={flow.href}
            className="group relative overflow-hidden bg-white rounded-xl border-2 border-gray-200 p-8 hover:border-primary-500 hover:shadow-xl transition-all"
          >
            {/* 背景装饰 */}
            <div className={`absolute top-0 right-0 w-32 h-32 bg-gradient-to-br ${flow.color} opacity-10 rounded-full -mr-16 -mt-16 group-hover:scale-150 transition-transform`} />
            
            {/* 图标 */}
            <div className={`relative w-14 h-14 bg-gradient-to-br ${flow.color} rounded-xl flex items-center justify-center mb-4 group-hover:scale-110 transition-transform`}>
              <Icon className="w-7 h-7 text-white" />
            </div>

            {/* 内容 */}
            <h3 className="relative text-xl font-bold text-gray-900 mb-3">
              {flow.title}
            </h3>
            <p className="relative text-sm text-gray-600 leading-relaxed">
              {flow.description}
            </p>

            {/* 箭头 */}
            <div className="relative mt-4 text-primary-600 font-medium group-hover:translate-x-2 transition-transform">
              了解详情 →
            </div>
          </Link>
        )
      })}
    </div>
  )
}
