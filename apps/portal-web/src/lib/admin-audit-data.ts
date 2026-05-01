export type AuditSeverity = 'LOW' | 'MEDIUM' | 'HIGH'
export type AuditResult = 'SUCCESS' | 'WARN' | 'FAILED'

export interface AdminAuditItem {
  id: string
  requestId: string
  action: string
  actor: string
  resourceType: string
  resourceId: string
  severity: AuditSeverity
  result: AuditResult
  ip: string
  createdAt: string
  detail: string
}

export const ADMIN_AUDIT_ITEMS: AdminAuditItem[] = [
  {
    id: 'audit_001',
    requestId: 'req_20260430_000101',
    action: 'listing.approve',
    actor: 'admin_zhang',
    resourceType: 'listing',
    resourceId: 'listing_001',
    severity: 'LOW',
    result: 'SUCCESS',
    ip: '10.10.1.23',
    createdAt: '2026-04-30 10:12:02',
    detail: '平台审核通过商品 listing_001，并触发链上登记事件。',
  },
  {
    id: 'audit_002',
    requestId: 'req_20260430_000102',
    action: 'notification.replay',
    actor: 'admin_wang',
    resourceType: 'notification',
    resourceId: 'notif_2871',
    severity: 'MEDIUM',
    result: 'WARN',
    ip: '10.10.1.58',
    createdAt: '2026-04-30 10:25:41',
    detail: '通知重放成功，但渠道回执延迟，等待二次确认。',
  },
  {
    id: 'audit_003',
    requestId: 'req_20260430_000103',
    action: 'subject.reject',
    actor: 'admin_liu',
    resourceType: 'subject',
    resourceId: 'subject_20260427_004',
    severity: 'HIGH',
    result: 'FAILED',
    ip: '10.10.1.77',
    createdAt: '2026-04-30 11:03:19',
    detail: '主体审核拒绝时附件校验失败，需人工复核并补录证据。',
  },
]
