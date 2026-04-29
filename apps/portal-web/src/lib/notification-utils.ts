import { ADMIN_NOTIFICATIONS, BUYER_NOTIFICATIONS, SELLER_NOTIFICATIONS } from '@/lib/notifications-data'
import type { NotificationItem } from '@/types/notifications'

export type ConsoleRole = 'buyer' | 'seller' | 'admin'

export function getNotificationsByRole(role: ConsoleRole): NotificationItem[] {
  if (role === 'buyer') return BUYER_NOTIFICATIONS
  if (role === 'seller') return SELLER_NOTIFICATIONS
  return ADMIN_NOTIFICATIONS
}

export function getNotificationByRoleAndId(role: ConsoleRole, id: string): NotificationItem | undefined {
  return getNotificationsByRole(role).find((item) => item.id === id)
}
