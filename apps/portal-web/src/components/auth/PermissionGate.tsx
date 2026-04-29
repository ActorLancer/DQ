'use client'

import { useAuthStore, UserRole } from '@/store/useAuthStore'
import { AlertCircle } from 'lucide-react'

interface PermissionGateProps {
  children: React.ReactNode
  requiredRole?: UserRole
  requiredPermission?: string
  fallback?: React.ReactNode
  showFallback?: boolean
}

/**
 * 权限门组件 - 根据权限显示/隐藏内容
 * 
 * @example
 * // 检查角色
 * <PermissionGate requiredRole="admin">
 *   <button>管理员功能</button>
 * </PermissionGate>
 * 
 * @example
 * // 检查具体权限
 * <PermissionGate requiredPermission="buyer:orders:write">
 *   <button>创建订单</button>
 * </PermissionGate>
 * 
 * @example
 * // 显示自定义 fallback
 * <PermissionGate 
 *   requiredRole="admin" 
 *   fallback={<p>需要管理员权限</p>}
 *   showFallback
 * >
 *   <button>管理员功能</button>
 * </PermissionGate>
 */
export default function PermissionGate({
  children,
  requiredRole,
  requiredPermission,
  fallback,
  showFallback = false,
}: PermissionGateProps) {
  const { hasRole, hasPermission } = useAuthStore()

  // 检查角色权限
  if (requiredRole && !hasRole(requiredRole)) {
    if (showFallback && fallback) {
      return <>{fallback}</>
    }
    return null
  }

  // 检查具体权限
  if (requiredPermission && !hasPermission(requiredPermission)) {
    if (showFallback && fallback) {
      return <>{fallback}</>
    }
    return null
  }

  // 有权限，渲染子组件
  return <>{children}</>
}

/**
 * 权限提示组件 - 显示权限不足提示
 */
export function PermissionDenied({ message }: { message?: string }) {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <div className="text-center max-w-md">
        <div className="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-4">
          <AlertCircle className="w-8 h-8 text-red-600" />
        </div>
        <h2 className="text-2xl font-bold text-gray-900 mb-2">权限不足</h2>
        <p className="text-gray-600 mb-6">
          {message || '您没有权限访问此功能，请联系管理员'}
        </p>
        <button
          onClick={() => window.history.back()}
          className="px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
        >
          返回上一页
        </button>
      </div>
    </div>
  )
}
