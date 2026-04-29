import { notFound } from 'next/navigation'
import { NotificationThreadPage } from '@/components/notifications'
import { getNotificationByRoleAndId } from '@/lib/notification-utils'

export default function SellerNotificationDetailPage({ params }: { params: { id: string } }) {
  const notice = getNotificationByRoleAndId('seller', params.id)
  if (!notice) return notFound()
  return <NotificationThreadPage role="seller" notification={notice} backPath="/console/seller/notifications" />
}
