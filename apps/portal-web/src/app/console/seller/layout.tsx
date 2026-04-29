'use client'

import ProtectedRoute from '@/components/auth/ProtectedRoute'
import ConsoleLayout from '@/components/console/ConsoleLayout'

export default function SellerConsoleLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <ProtectedRoute requiredRole="seller" fallbackUrl="/login">
      <ConsoleLayout role="seller">
        {children}
      </ConsoleLayout>
    </ProtectedRoute>
  )
}
