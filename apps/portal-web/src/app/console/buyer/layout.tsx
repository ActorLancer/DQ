'use client'

import ProtectedRoute from '@/components/auth/ProtectedRoute'
import ConsoleLayout from '@/components/console/ConsoleLayout'

export default function BuyerConsoleLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <ProtectedRoute requiredRole="buyer" fallbackUrl="/login">
      <ConsoleLayout role="buyer">
        {children}
      </ConsoleLayout>
    </ProtectedRoute>
  )
}
