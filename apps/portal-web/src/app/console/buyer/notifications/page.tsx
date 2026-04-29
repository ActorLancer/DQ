'use client'

import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { NotificationCenter } from '@/components/notifications'
import { BUYER_NOTIFICATIONS } from '@/lib/notifications-data'

export default function BuyerNotificationsPage() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:notifications:read"
        sessionExpiresAt={sessionExpiresAt}
      />
      <NotificationCenter
        role="buyer"
        title="事件通知中心"
        subtitle="汇总平台、系统、外部服务与交易对手事件，支持筛选、分页、置顶、回复与业务跳转。"
        initialItems={BUYER_NOTIFICATIONS}
        rulesPath="/console/buyer/notifications/rules"
      />
    </>
  )
}
