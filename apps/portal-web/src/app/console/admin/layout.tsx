import { redirect } from 'next/navigation'

export default function LegacyAdminLayout({
  children,
}: {
  children: React.ReactNode
}) {
  void children
  redirect('/admin/console')
}
