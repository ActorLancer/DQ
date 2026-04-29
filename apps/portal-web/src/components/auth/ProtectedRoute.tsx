'use client'

import { useEffect, useState } from 'react'
import { useRouter, usePathname } from 'next/navigation'
import { useAuthStore, UserRole } from '@/store/useAuthStore'
import { Loader2 } from 'lucide-react'

interface ProtectedRouteProps {
  children: React.ReactNode
  requiredRole?: UserRole
  requiredPermission?: string
  fallbackUrl?: string
}

export default function ProtectedRoute({
  children,
  requiredRole,
  requiredPermission,
  fallbackUrl = '/login',
}: ProtectedRouteProps) {
  const router = useRouter()
  const pathname = usePathname()
  const { isAuthenticated, user, hasRole, hasPermission, setCurrentRole } = useAuthStore()
  const [isHydrated, setIsHydrated] = useState(false)

  useEffect(() => {
    setIsHydrated(true)
  }, [])

  useEffect(() => {
    if (!isHydrated) return

    if (!isAuthenticated) {
      const returnUrl = encodeURIComponent(pathname)
      router.push(`${fallbackUrl}?returnUrl=${returnUrl}`)
      return
    }

    if (requiredRole && !hasRole(requiredRole)) {
      const returnUrl = encodeURIComponent(pathname)
      router.push(
        `/role-access?targetRole=${requiredRole}&fallbackUrl=${encodeURIComponent(fallbackUrl)}&returnUrl=${returnUrl}`
      )
      return
    }

    if (requiredRole && user?.currentRole !== requiredRole) {
      setCurrentRole(requiredRole)
      return
    }

    if (requiredPermission && !hasPermission(requiredPermission)) {
      router.push('/unauthorized')
    }
  }, [
    isHydrated,
    isAuthenticated,
    user,
    requiredRole,
    requiredPermission,
    pathname,
    router,
    fallbackUrl,
    hasRole,
    hasPermission,
    setCurrentRole,
  ])

  if (!isHydrated || !isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <Loader2 className="w-8 h-8 animate-spin text-primary-600 mx-auto mb-4" />
          <p className="text-gray-600">验证登录状态...</p>
        </div>
      </div>
    )
  }

  if ((requiredRole && !hasRole(requiredRole)) || (requiredRole && user?.currentRole !== requiredRole)) {
    return null
  }

  if (requiredPermission && !hasPermission(requiredPermission)) {
    return null
  }

  return <>{children}</>
}
