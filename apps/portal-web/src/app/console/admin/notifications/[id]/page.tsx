import { notFound } from 'next/navigation'
import { NotificationThreadPage } from '@/components/notifications'
import { getNotificationByRoleAndId } from '@/lib/notification-utils'

export default function AdminNotificationDetailPage({ params }: { params: { id: string } }) {
  const notice = getNotificationByRoleAndId('admin', params.id)
  if (!notice) return notFound()
  return <NotificationThreadPage role="admin" notification={notice} backPath="/admin/console/notifications" />
}
