'use client'

import Link from 'next/link'
import { useMemo, useState } from 'react'
import { useParams, useRouter } from 'next/navigation'
import { motion } from 'framer-motion'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { ApiKey, MOCK_API_KEYS } from '@/lib/buyer-api-keys-data'
import { ArrowLeft, CheckCircle, Copy, ExternalLink, RotateCw, Shield, Trash2, XCircle } from 'lucide-react'

const STATUS_LABEL: Record<ApiKey['status'], { label: string; color: string }> = {
  ACTIVE: { label: '活跃', color: 'bg-green-100 text-green-800' },
  DISABLED: { label: '已禁用', color: 'bg-gray-100 text-gray-800' },
  EXPIRED: { label: '已过期', color: 'bg-red-100 text-red-800' },
}

export default function ApiKeyDetailPage() {
  const router = useRouter()
  const params = useParams<{ id: string }>()
  const [copied, setCopied] = useState(false)

  const apiKey = useMemo(() => MOCK_API_KEYS.find((item) => item.id === params.id), [params.id])
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  if (!apiKey) {
    return (
      <div className="p-8">
        <div className="bg-white rounded-xl border border-gray-200 p-8 text-center">
          <h1 className="text-2xl font-bold text-gray-900 mb-2">未找到 API 密钥</h1>
          <Link href="/console/buyer/api-keys" className="text-primary-600 hover:text-primary-700">返回 API 密钥列表</Link>
        </div>
      </div>
    )
  }

  const masked = `${apiKey.keyPrefix}••••••••${apiKey.id.slice(-4)}`

  const handleCopyMasked = async () => {
    await navigator.clipboard.writeText(masked)
    setCopied(true)
    setTimeout(() => setCopied(false), 1500)
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:api-keys:write"
        sessionExpiresAt={sessionExpiresAt}
      />

      <motion.div
        initial={{ opacity: 0, x: 32 }}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.28, ease: 'easeOut' }}
        className="p-8"
      >
        <div className="flex items-center justify-between mb-6">
          <button onClick={() => router.back()} className="inline-flex items-center gap-2 text-gray-700 hover:text-gray-900">
            <ArrowLeft className="w-4 h-4" />
            <span>返回</span>
          </button>
          <Link href="/console/buyer/api-keys" className="text-sm text-primary-600 hover:text-primary-700">关闭详情</Link>
        </div>

        <div className="bg-white rounded-2xl border border-gray-200 shadow-sm overflow-hidden">
          <div className="px-8 py-7 border-b border-gray-100 bg-gradient-to-r from-slate-50 to-white">
            <div className="flex items-start justify-between gap-6">
              <div>
                <h1 className="text-2xl font-bold text-gray-900 mb-1">{apiKey.name}</h1>
                <p className="text-sm text-gray-600">订单绑定型 API 密钥详情页，可查看并执行常用操作</p>
              </div>
              <span className={`status-tag ${STATUS_LABEL[apiKey.status].color}`}>{STATUS_LABEL[apiKey.status].label}</span>
            </div>
          </div>

          <div className="p-8 grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="xl:col-span-2 space-y-6">
              <div className="bg-gray-50 rounded-xl border border-gray-200 p-5">
                <div className="text-xs text-gray-500 mb-2">API Key</div>
                <div className="flex items-center justify-between gap-4">
                  <code className="font-mono text-sm text-gray-900 break-all">{masked}</code>
                  <button onClick={handleCopyMasked} className="inline-flex items-center gap-2 px-3 py-2 text-sm rounded-lg bg-primary-600 text-white hover:bg-primary-700">
                    <Copy className="w-4 h-4" />
                    <span>{copied ? '已复制' : '复制'}</span>
                  </button>
                </div>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="bg-white rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">关联订单</div>
                  <div className="font-medium text-gray-900">{apiKey.orderId}</div>
                  <Link href={`/console/buyer/orders`} className="inline-flex items-center gap-1 text-xs text-primary-600 hover:text-primary-700 mt-2">
                    <span>查看订单账单</span><ExternalLink className="w-3 h-3" />
                  </Link>
                </div>
                <div className="bg-white rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">订阅商品</div>
                  <div className="font-medium text-gray-900">{apiKey.listingTitle}</div>
                </div>
                <div className="bg-white rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">创建时间</div>
                  <div className="font-medium text-gray-900">{apiKey.createdAt}</div>
                </div>
                <div className="bg-white rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">最近使用</div>
                  <div className="font-medium text-gray-900">{apiKey.lastUsedAt || '从未使用'}</div>
                </div>
                <div className="bg-white rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">过期时间</div>
                  <div className="font-medium text-gray-900">{apiKey.expiresAt || '无限期'}</div>
                </div>
                <div className="bg-white rounded-xl border border-gray-200 p-4">
                  <div className="text-xs text-gray-500 mb-1">累计调用</div>
                  <div className="font-medium text-gray-900">{apiKey.totalCalls.toLocaleString()}</div>
                </div>
              </div>

              <div className="bg-white rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-3">权限范围</div>
                <div className="flex flex-wrap gap-2">
                  {apiKey.permissions.map((p) => (
                    <code key={p} className="text-xs bg-gray-100 px-2 py-1 rounded">{p}</code>
                  ))}
                </div>
              </div>

              <div className="bg-white rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-3">IP 白名单</div>
                {apiKey.ipWhitelist.length > 0 ? (
                  <div className="space-y-2">
                    {apiKey.ipWhitelist.map((ip) => (
                      <code key={ip} className="block text-xs bg-gray-100 px-2 py-1 rounded">{ip}</code>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-gray-500">未设置，默认按账户权限控制</p>
                )}
              </div>
            </section>

            <aside className="space-y-4">
              <div className="bg-white rounded-xl border border-gray-200 p-5">
                <div className="text-sm font-semibold text-gray-900 mb-4">快捷操作</div>
                <div className="space-y-2">
                  <button className="w-full inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg bg-primary-600 text-white hover:bg-primary-700">
                    <RotateCw className="w-4 h-4" />
                    <span>轮换密钥</span>
                  </button>
                  <button className="w-full inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg border border-gray-300 text-gray-700 hover:bg-gray-50">
                    <XCircle className="w-4 h-4" />
                    <span>禁用密钥</span>
                  </button>
                  <button className="w-full inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-lg border border-red-300 text-red-600 hover:bg-red-50">
                    <Trash2 className="w-4 h-4" />
                    <span>删除密钥</span>
                  </button>
                </div>
              </div>

              <div className="bg-amber-50 rounded-xl border border-amber-200 p-5">
                <div className="flex items-start gap-2">
                  <Shield className="w-4 h-4 mt-0.5 text-amber-700" />
                  <div>
                    <div className="text-sm font-semibold text-amber-900 mb-1">安全建议</div>
                    <ul className="text-xs text-amber-800 space-y-1">
                      <li>• 高风险环境建议 30 天轮换</li>
                      <li>• 白名单建议最小可用范围</li>
                      <li>• 关键操作会写入审计日志</li>
                    </ul>
                  </div>
                </div>
              </div>

              <div className="bg-emerald-50 rounded-xl border border-emerald-200 p-5">
                <div className="flex items-center gap-2 text-emerald-800 text-sm font-medium">
                  <CheckCircle className="w-4 h-4" />
                  <span>当前状态正常</span>
                </div>
              </div>
            </aside>
          </div>
        </div>
      </motion.div>
    </>
  )
}
