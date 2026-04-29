'use client'

import { useEffect, useState } from 'react'
import { usePathname, useRouter } from 'next/navigation'
import { Loader2 } from 'lucide-react'
import { useAdminAuthStore } from '@/store/useAdminAuthStore'

interface AdminProtectedRouteProps {
  children: React.ReactNode
  requiredPermission?: string
  fallbackUrl?: string
}

export default function AdminProtectedRoute({
  children,
  requiredPermission,
  fallbackUrl = '/admin/login',
}: AdminProtectedRouteProps) {
  const router = useRouter()
  const pathname = usePathname()
  const { isAuthenticated, hasPermission } = useAdminAuthStore()
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

    if (requiredPermission && !hasPermission(requiredPermission)) {
      router.push('/unauthorized')
    }
  }, [isHydrated, isAuthenticated, hasPermission, requiredPermission, fallbackUrl, pathname, router])

  if (!isHydrated || !isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <Loader2 className="w-8 h-8 animate-spin text-primary-600 mx-auto mb-4" />
          <p className="text-gray-600">验证管理端登录状态...</p>
        </div>
      </div>
    )
  }

  if (requiredPermission && !hasPermission(requiredPermission)) {
    return null
  }

  return <>{children}</>
}
