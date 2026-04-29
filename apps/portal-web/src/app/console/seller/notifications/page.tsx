'use client'

import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { NotificationCenter } from '@/components/notifications'
import { SELLER_NOTIFICATIONS } from '@/lib/notifications-data'

export default function SellerNotificationsPage() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:notifications:read"
        sessionExpiresAt={sessionExpiresAt}
      />
      <NotificationCenter
        role="seller"
        title="供应商事件通知中心"
        subtitle="统一管理商品运营、调用告警、链路异常与买家沟通消息。"
        initialItems={SELLER_NOTIFICATIONS}
        rulesPath="/console/seller/notifications/rules"
      />
    </>
  )
}
