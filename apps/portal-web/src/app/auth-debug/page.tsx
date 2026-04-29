'use client'

import { useEffect, useMemo, useState } from 'react'
import Link from 'next/link'
import { ArrowLeft } from 'lucide-react'
import { useAuthStore } from '@/store/useAuthStore'
import { useAdminAuthStore } from '@/store/useAdminAuthStore'

export default function AuthDebugPage() {
  const trade = useAuthStore()
  const admin = useAdminAuthStore()

  const [isHydrated, setIsHydrated] = useState(false)
  const [tradeStorage, setTradeStorage] = useState('loading...')
  const [adminStorage, setAdminStorage] = useState('loading...')

  useEffect(() => {
    setIsHydrated(true)
    setTradeStorage(localStorage.getItem('trade-auth-storage') || 'null')
    setAdminStorage(localStorage.getItem('admin-auth-storage') || 'null')
  }, [trade.isAuthenticated, admin.isAuthenticated])

  const roleChecks = useMemo(() => {
    const buyer = trade.hasRole('buyer')
    const seller = trade.hasRole('seller')
    const adminRole = admin.isAuthenticated
    return { buyer, seller, adminRole }
  }, [trade, admin.isAuthenticated])

  if (!isHydrated) {
    return <div className="min-h-screen bg-gray-50 p-8">Hydrating...</div>
  }

  return (
    <div className="min-h-screen bg-gray-50 p-8">
      <div className="max-w-5xl mx-auto">
        <Link href="/" className="inline-flex items-center gap-2 text-primary-600 hover:text-primary-700 mb-8">
          <ArrowLeft className="w-5 h-5" />
          <span>返回首页</span>
        </Link>

        <h1 className="text-3xl font-bold text-gray-900 mb-8">认证状态调试</h1>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-4">交易端会话</h2>
            <p className="text-sm text-gray-600 mb-2">isAuthenticated: {trade.isAuthenticated ? '✓ 已认证' : '✗ 未认证'}</p>
            <p className="text-sm text-gray-600 mb-2">当前角色: {trade.user?.currentRole || '-'}</p>
            <p className="text-sm text-gray-600 mb-2">角色列表: {(trade.user?.roles || []).join(', ') || '-'}</p>
            <p className="text-sm text-gray-600">Token: {trade.token ? `${trade.token.slice(0, 40)}...` : 'null'}</p>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <h2 className="text-xl font-bold text-gray-900 mb-4">管理端会话</h2>
            <p className="text-sm text-gray-600 mb-2">isAuthenticated: {admin.isAuthenticated ? '✓ 已认证' : '✗ 未认证'}</p>
            <p className="text-sm text-gray-600 mb-2">管理员: {admin.user?.name || '-'}</p>
            <p className="text-sm text-gray-600">Token: {admin.token ? `${admin.token.slice(0, 40)}...` : 'null'}</p>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6 lg:col-span-2">
            <h2 className="text-xl font-bold text-gray-900 mb-4">角色检查（汇总）</h2>
            <div className="space-y-2">
              <p className={roleChecks.buyer ? 'text-green-600 font-medium' : 'text-red-600 font-medium'}>hasRole(&apos;buyer&apos;): {roleChecks.buyer ? '✓ 有权限' : '✗ 无权限'}</p>
              <p className={roleChecks.seller ? 'text-green-600 font-medium' : 'text-red-600 font-medium'}>hasRole(&apos;seller&apos;): {roleChecks.seller ? '✓ 有权限' : '✗ 无权限'}</p>
              <p className={roleChecks.adminRole ? 'text-green-600 font-medium' : 'text-red-600 font-medium'}>admin session: {roleChecks.adminRole ? '✓ 已登录' : '✗ 未登录'}</p>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6 lg:col-span-2">
            <h2 className="text-xl font-bold text-gray-900 mb-4">LocalStorage 内容</h2>
            <p className="text-xs text-gray-500 mb-2">trade-auth-storage</p>
            <pre className="text-xs bg-gray-100 p-4 rounded overflow-auto mb-4">{tradeStorage}</pre>
            <p className="text-xs text-gray-500 mb-2">admin-auth-storage</p>
            <pre className="text-xs bg-gray-100 p-4 rounded overflow-auto">{adminStorage}</pre>
          </div>
        </div>
      </div>
    </div>
  )
}
