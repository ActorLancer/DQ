'use client'

import Link from 'next/link'
import { useAuthStore } from '@/store/useAuthStore'

export default function RoleConsoleEntrances() {
  const { isAuthenticated, user } = useAuthStore()

  if (!isAuthenticated || !user) return null

  return (
    <div className="mt-6 flex flex-wrap items-center justify-center gap-3">
      {user.roles.includes('buyer') && (
        <Link href="/console/buyer" className="px-4 py-2 rounded-lg bg-white text-primary-700 font-medium hover:bg-primary-50">
          进入买家控制台
        </Link>
      )}
      {user.roles.includes('seller') && (
        <Link href="/console/seller" className="px-4 py-2 rounded-lg bg-white text-green-700 font-medium hover:bg-green-50">
          进入供应商后台
        </Link>
      )}
    </div>
  )
}
