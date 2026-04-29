'use client'

import AdminProtectedRoute from '@/components/auth/AdminProtectedRoute'
import ConsoleLayout from '@/components/console/ConsoleLayout'

export default function AdminConsoleLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <AdminProtectedRoute>
      <ConsoleLayout role="admin">
        {children}
      </ConsoleLayout>
    </AdminProtectedRoute>
  )
}
