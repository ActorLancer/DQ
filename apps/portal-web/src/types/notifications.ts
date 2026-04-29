export type NotificationLevel = 'info' | 'warning' | 'error' | 'success'
export type NotificationStatus = 'unread' | 'read' | 'archived'
export type NotificationSource = 'platform_internal' | 'system_internal' | 'service_external' | 'buyer_message' | 'seller_message'

export interface NotificationReply {
  id: string
  author: string
  role: string
  content: string
  createdAt: string
}

export interface NotificationItem {
  id: string
  title: string
  desc: string
  detail: string
  level: NotificationLevel
  status: NotificationStatus
  time: string
  timestamp: string
  category: 'subscription' | 'request' | 'billing' | 'system' | 'risk' | 'chain' | 'contract' | 'security'
  source: NotificationSource
  priority: 1 | 2 | 3 | 4 | 5
  pinned: boolean
  targetPath: string
  requestId?: string
  txHash?: string
  relatedEntityType?: 'order' | 'request' | 'subscription' | 'listing' | 'subject' | 'ticket'
  relatedEntityId?: string
  replies: NotificationReply[]
}
