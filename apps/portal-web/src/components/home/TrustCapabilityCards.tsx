'use client'

import { Shield, Lock, FileCheck, Eye, Zap, Link as LinkIcon } from 'lucide-react'

const CAPABILITIES = [
  {
    id: 'compliance',
    name: '合规',
    icon: FileCheck,
    description: '符合国家数据安全法规要求',
    color: 'text-blue-600',
    bgColor: 'bg-blue-50',
  },
  {
    id: 'privacy',
    name: '脱敏',
    icon: Lock,
    description: '多种脱敏策略保护隐私',
    color: 'text-purple-600',
    bgColor: 'bg-purple-50',
  },
  {
    id: 'authorization',
    name: '授权',
    icon: Shield,
    description: '精细化权限控制与审批',
    color: 'text-green-600',
    bgColor: 'bg-green-50',
  },
  {
    id: 'audit',
    name: '审计',
    icon: Eye,
    description: '全链路可追溯审计日志',
    color: 'text-orange-600',
    bgColor: 'bg-orange-50',
  },
  {
    id: 'api-delivery',
    name: 'API 交付',
    icon: Zap,
    description: '高性能实时数据接口',
    color: 'text-yellow-600',
    bgColor: 'bg-yellow-50',
  },
  {
    id: 'chain-proof',
    name: '链上存证',
    icon: LinkIcon,
    description: '区块链不可篡改凭证',
    color: 'text-red-600',
    bgColor: 'bg-red-50',
  },
]

export default function TrustCapabilityCards() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      {CAPABILITIES.map((capability) => {
        const Icon = capability.icon
        return (
          <div
            key={capability.id}
            className="group bg-white rounded-xl border border-gray-200 p-6 hover:shadow-lg hover:border-primary-300 transition-all"
          >
            <div className={`w-12 h-12 ${capability.bgColor} rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform`}>
              <Icon className={`w-6 h-6 ${capability.color}`} />
            </div>
            <h3 className="text-lg font-bold text-gray-900 mb-2">
              {capability.name}
            </h3>
            <p className="text-sm text-gray-600">
              {capability.description}
            </p>
          </div>
        )
      })}
    </div>
  )
}
