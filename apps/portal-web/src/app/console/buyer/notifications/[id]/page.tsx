import { notFound } from 'next/navigation'
import { NotificationThreadPage } from '@/components/notifications'
import { getNotificationByRoleAndId } from '@/lib/notification-utils'

export default function BuyerNotificationDetailPage({ params }: { params: { id: string } }) {
  const notice = getNotificationByRoleAndId('buyer', params.id)
  if (!notice) return notFound()
  return <NotificationThreadPage role="buyer" notification={notice} backPath="/console/buyer/notifications" />
}
