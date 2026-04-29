'use client'

import { useEffect, useMemo, useState } from 'react'
import { useSearchParams } from 'next/navigation'
import Link from 'next/link'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import PermissionGate from '@/components/auth/PermissionGate'
import {
  Plus,
  Copy,
  RotateCw,
  Trash2,
  CheckCircle,
  XCircle,
  AlertCircle,
  Shield,
  FileText,
  ExternalLink,
} from 'lucide-react'
import { ApiKey, MOCK_API_KEYS, PAID_ORDERS } from '@/lib/buyer-api-keys-data'

const STATUS_CONFIG = {
  ACTIVE: { label: '活跃', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  DISABLED: { label: '已禁用', color: 'bg-gray-100 text-gray-800', icon: XCircle },
  EXPIRED: { label: '已过期', color: 'bg-red-100 text-red-800', icon: AlertCircle },
}

export default function BuyerApiKeysPage() {
  const searchParams = useSearchParams()
  const [apiKeys, setApiKeys] = useState(MOCK_API_KEYS)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showKeyModal, setShowKeyModal] = useState(false)
  const [newKeyData, setNewKeyData] = useState<ApiKey | null>(null)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [form, setForm] = useState({
    name: '',
    orderId: PAID_ORDERS[0]?.orderId ?? '',
    expireDays: '',
    whitelist: '',
  })
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const paidOrders = useMemo(() => PAID_ORDERS, [])

  useEffect(() => {
    const orderId = searchParams.get('orderId')
    if (!orderId) return
    const matched = paidOrders.find((order) => order.orderId === orderId)
    if (!matched) return
    setForm((prev) => ({ ...prev, orderId: matched.orderId }))
  }, [searchParams, paidOrders])

  const handleCopyKey = (key: string) => {
    navigator.clipboard.writeText(key)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  const handleCreateKey = () => {
    const order = paidOrders.find((item) => item.orderId === form.orderId)
    if (!order || !form.name.trim()) return

    const expireDays = form.expireDays ? Number(form.expireDays) : null
    const expiresAt = expireDays
      ? new Date(Date.now() + expireDays * 24 * 60 * 60 * 1000).toISOString().slice(0, 19).replace('T', ' ')
      : null

    const key: ApiKey = {
      id: `key_${Date.now()}`,
      name: form.name.trim(),
      keyPrefix: 'sk_live_',
      fullKey: `sk_live_${Math.random().toString(36).slice(2)}${Math.random().toString(36).slice(2)}`,
      orderId: order.orderId,
      listingTitle: order.listingTitle,
      permissions: ['read'],
      status: 'ACTIVE',
      createdAt: new Date().toISOString().slice(0, 19).replace('T', ' '),
      expiresAt,
      lastUsedAt: null,
      totalCalls: 0,
      ipWhitelist: form.whitelist
        .split('\n')
        .map((v) => v.trim())
        .filter(Boolean),
    }

    setNewKeyData(key)
    setShowCreateModal(false)
    setShowKeyModal(true)
  }

  const handleDisableKey = (keyId: string) => {
    setApiKeys((prev) => prev.map((key) => (key.id === keyId ? { ...key, status: 'DISABLED' } : key)))
  }

  const handleRotateKey = (keyId: string) => {
    setApiKeys((prev) =>
      prev.map((key) =>
        key.id === keyId
          ? {
              ...key,
              keyPrefix: key.keyPrefix.startsWith('sk_live_') ? 'sk_live_rot_' : 'sk_test_rot_',
              createdAt: new Date().toISOString().slice(0, 19).replace('T', ' '),
              lastUsedAt: null,
            }
          : key
      )
    )
  }

  const handleDeleteKey = (keyId: string) => {
    if (confirm('确定要删除这个 API Key 吗？此操作不可撤销。')) {
      setApiKeys((prev) => prev.filter((key) => key.id !== keyId))
    }
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

      <div className="p-8">
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">API 密钥管理</h1>
            <p className="text-gray-600">仅可为已支付订单创建 API Key，并与订单状态追溯关联</p>
          </div>
          <PermissionGate requiredPermission="buyer:api-keys:write">
            <button
              onClick={() => setShowCreateModal(true)}
              disabled={paidOrders.length === 0}
              className="flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium disabled:opacity-50"
            >
              <Plus className="w-5 h-5" />
              <span>创建 API Key</span>
            </button>
          </PermissionGate>
        </div>

        <div className="bg-yellow-50 border border-yellow-200 rounded-xl p-6 mb-6">
          <div className="flex gap-4">
            <Shield className="w-6 h-6 text-yellow-600 flex-shrink-0" />
            <div>
              <h3 className="font-bold text-yellow-900 mb-2">安全提示</h3>
              <ul className="text-sm text-yellow-800 space-y-1">
                <li>• API Key 必须绑定已支付订单，自动继承订单可用范围</li>
                <li>• 创建后仅显示一次，请妥善保存</li>
                <li>• 建议定期轮换并配置 IP 白名单</li>
                <li>• 轮换、复制、禁用、删除均记录审计日志</li>
              </ul>
            </div>
          </div>
        </div>

        <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">名称</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">Key</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">最近使用</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {apiKeys.map((apiKey) => {
                const statusConfig = STATUS_CONFIG[apiKey.status]
                const StatusIcon = statusConfig.icon
                const maskedKey = `${apiKey.keyPrefix}••••••••${apiKey.id.slice(-4)}`

                return (
                  <tr key={apiKey.id} className="hover:bg-gray-50">
                    <td className="py-4 px-6">
                      <div className="font-medium text-gray-900">{apiKey.name}</div>
                    </td>
                    <td className="py-4 px-6">
                      <code className="font-mono text-sm text-gray-900 bg-gray-50 px-2 py-1 rounded">{maskedKey}</code>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className={`status-tag ${statusConfig.color}`}>
                        <StatusIcon className="w-3.5 h-3.5" />
                        <span>{statusConfig.label}</span>
                      </span>
                    </td>
                    <td className="py-4 px-6 text-sm text-gray-900">{apiKey.lastUsedAt ? apiKey.lastUsedAt.split(' ')[0] : '从未使用'}</td>
                    <td className="py-4 px-6">
                      <div className="flex items-center justify-end gap-2">
                        <Link href={`/console/buyer/api-keys/${apiKey.id}`} className="p-2 text-gray-600 hover:text-primary-600 hover:bg-primary-50 rounded-lg" title="详情">
                          详情
                        </Link>
                        <PermissionGate requiredPermission="buyer:api-keys:write">
                          {apiKey.status === 'ACTIVE' && (
                            <>
                              <button
                                onClick={() => handleRotateKey(apiKey.id)}
                                className="p-2 text-gray-600 hover:text-primary-600 hover:bg-primary-50 rounded-lg"
                                title="轮换"
                              >
                                <RotateCw className="w-4 h-4" />
                              </button>
                              <button
                                onClick={() => handleDisableKey(apiKey.id)}
                                className="p-2 text-gray-600 hover:text-orange-600 hover:bg-orange-50 rounded-lg"
                                title="禁用"
                              >
                                <XCircle className="w-4 h-4" />
                              </button>
                            </>
                          )}
                          <button
                            onClick={() => handleDeleteKey(apiKey.id)}
                            className="p-2 text-gray-600 hover:text-red-600 hover:bg-red-50 rounded-lg"
                            title="删除"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </PermissionGate>
                      </div>
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      </div>

      <PermissionGate requiredPermission="buyer:api-keys:write">
        {showCreateModal && (
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div className="bg-white rounded-xl p-8 max-w-2xl w-full mx-4 animate-fade-in">
              <h2 className="text-2xl font-bold text-gray-900 mb-6">创建 API Key</h2>

              <div className="space-y-4 mb-6">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">Key 名称 <span className="text-red-500">*</span></label>
                  <input
                    type="text"
                    value={form.name}
                    onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))}
                    placeholder="例如：生产环境 - 企业风险数据"
                    className="input"
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">关联已支付订单 <span className="text-red-500">*</span></label>
                  <select
                    value={form.orderId}
                    onChange={(e) => setForm((prev) => ({ ...prev, orderId: e.target.value }))}
                    className="input"
                  >
                    {paidOrders.map((order) => (
                      <option key={order.orderId} value={order.orderId}>
                        {order.orderId} | {order.listingTitle} | {order.plan} | ¥{order.amount.toLocaleString()}
                      </option>
                    ))}
                  </select>
                  <p className="text-xs text-gray-500 mt-1">仅展示订单账单中已完成支付的订单</p>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">过期时间</label>
                  <select value={form.expireDays} onChange={(e) => setForm((prev) => ({ ...prev, expireDays: e.target.value }))} className="input">
                    <option value="">无限期</option>
                    <option value="30">30 天</option>
                    <option value="90">90 天</option>
                    <option value="180">180 天</option>
                    <option value="365">365 天</option>
                  </select>
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">IP 白名单（可选）</label>
                  <textarea
                    value={form.whitelist}
                    onChange={(e) => setForm((prev) => ({ ...prev, whitelist: e.target.value }))}
                    placeholder="每行一个 IP 地址，例如：\n192.168.1.100\n10.0.0.50"
                    className="input min-h-[100px]"
                  />
                </div>
              </div>

              <div className="flex gap-3">
                <button onClick={() => setShowCreateModal(false)} className="flex-1 px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">取消</button>
                <button
                  onClick={handleCreateKey}
                  disabled={!form.name.trim() || !form.orderId}
                  className="flex-1 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium disabled:opacity-50"
                >
                  创建
                </button>
              </div>
            </div>
          </div>
        )}

        {showKeyModal && newKeyData && (
          <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
            <div className="bg-white rounded-xl p-8 max-w-2xl w-full mx-4 animate-fade-in">
              <div className="text-center mb-6">
                <div className="w-16 h-16 bg-success-100 rounded-full flex items-center justify-center mx-auto mb-4">
                  <CheckCircle className="w-8 h-8 text-success-600" />
                </div>
                <h2 className="text-2xl font-bold text-gray-900 mb-2">API Key 创建成功</h2>
                <p className="text-gray-600">请立即复制并妥善保存，关闭后将无法再次查看</p>
              </div>

              <div className="bg-red-50 border-2 border-red-200 rounded-lg p-6 mb-6">
                <div className="flex items-start gap-3 mb-4">
                  <AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" />
                  <div className="text-sm text-red-800">
                    <p className="font-bold mb-1">重要提示</p>
                    <p>此 API Key 仅显示一次，且已绑定订单 {newKeyData.orderId}。</p>
                  </div>
                </div>

                <div className="bg-white rounded-lg p-4">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-sm font-medium text-gray-700">API Key</span>
                    <button
                      onClick={() => handleCopyKey(newKeyData.fullKey || '')}
                      className="flex items-center gap-2 px-3 py-1.5 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium"
                    >
                      {copiedKey === newKeyData.fullKey ? (
                        <>
                          <CheckCircle className="w-4 h-4" />
                          <span>已复制</span>
                        </>
                      ) : (
                        <>
                          <Copy className="w-4 h-4" />
                          <span>复制</span>
                        </>
                      )}
                    </button>
                  </div>
                  <code className="block font-mono text-sm text-gray-900 bg-gray-50 px-3 py-2 rounded break-all">{newKeyData.fullKey}</code>
                </div>
              </div>

              <button
                onClick={() => {
                  setShowKeyModal(false)
                  setApiKeys((prev) => [newKeyData, ...prev])
                  setNewKeyData(null)
                  setForm({ name: '', orderId: PAID_ORDERS[0]?.orderId ?? '', expireDays: '', whitelist: '' })
                }}
                className="w-full px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
              >
                我已保存，关闭
              </button>
            </div>
          </div>
        )}
      </PermissionGate>

      <div className="px-8 pb-8">
        <div className="bg-white rounded-xl border border-gray-200 p-5 flex items-center justify-between">
          <div className="flex items-center gap-3 text-sm text-gray-700">
            <FileText className="w-4 h-4 text-gray-500" />
            <span>订单账单中的已支付订单是创建 API Key 的唯一来源</span>
          </div>
          <a href="/console/buyer/orders" className="text-sm text-primary-600 hover:text-primary-700 font-medium inline-flex items-center gap-1">
            <span>前往订单账单</span>
            <ExternalLink className="w-3.5 h-3.5" />
          </a>
        </div>
      </div>
    </>
  )
}
