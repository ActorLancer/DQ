'use client'

import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { NotificationCenter } from '@/components/notifications'
import { ADMIN_NOTIFICATIONS } from '@/lib/notifications-data'

export default function AdminNotificationsPage() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  return (
    <>
      <SessionIdentityBar
        subjectName="数据交易平台运营中心"
        roleName="平台管理员"
        tenantId="tenant_platform_001"
        scope="admin:notifications:read"
        sessionExpiresAt={sessionExpiresAt}
      />
      <NotificationCenter
        role="admin"
        title="平台事件通知中心"
        subtitle="处理平台级风险、一致性异常、链上事件与跨系统通知回执。"
        initialItems={ADMIN_NOTIFICATIONS}
        rulesPath="/console/admin/notifications/rules"
      />
    </>
  )
}
