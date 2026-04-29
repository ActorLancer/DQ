'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import PermissionGate from '@/components/auth/PermissionGate'
import { 
  Plus,
  Key,
  Copy,
  Eye,
  EyeOff,
  RotateCw,
  Trash2,
  CheckCircle,
  XCircle,
  AlertCircle,
  Calendar,
  Activity,
  Shield
} from 'lucide-react'

interface ApiKey {
  id: string
  name: string
  keyPrefix: string
  fullKey?: string
  subscriptionId: string
  listingTitle: string
  permissions: string[]
  status: 'ACTIVE' | 'DISABLED' | 'EXPIRED'
  createdAt: string
  expiresAt: string | null
  lastUsedAt: string | null
  totalCalls: number
  ipWhitelist: string[]
}

const MOCK_API_KEYS: ApiKey[] = [
  {
    id: 'key_001',
    name: '生产环境 - 企业风险数据',
    keyPrefix: 'sk_live_',
    subscriptionId: 'sub_001',
    listingTitle: '企业工商风险数据',
    permissions: ['read'],
    status: 'ACTIVE',
    createdAt: '2026-03-28 10:00:00',
    expiresAt: '2026-05-28 23:59:59',
    lastUsedAt: '2026-04-28 15:30:00',
    totalCalls: 6580,
    ipWhitelist: ['192.168.1.100', '10.0.0.50'],
  },
  {
    id: 'key_002',
    name: '测试环境 - 企业风险数据',
    keyPrefix: 'sk_test_',
    subscriptionId: 'sub_001',
    listingTitle: '企业工商风险数据',
    permissions: ['read'],
    status: 'ACTIVE',
    createdAt: '2026-03-28 10:05:00',
    expiresAt: '2026-05-28 23:59:59',
    lastUsedAt: '2026-04-27 18:20:00',
    totalCalls: 1250,
    ipWhitelist: [],
  },
  {
    id: 'key_003',
    name: '生产环境 - 消费行为数据',
    keyPrefix: 'sk_live_',
    subscriptionId: 'sub_002',
    listingTitle: '消费者行为分析数据',
    permissions: ['read'],
    status: 'ACTIVE',
    createdAt: '2026-01-01 09:00:00',
    expiresAt: null,
    lastUsedAt: '2026-04-28 16:15:00',
    totalCalls: 12350,
    ipWhitelist: ['192.168.1.100'],
  },
  {
    id: 'key_004',
    name: '旧版 Key - 已禁用',
    keyPrefix: 'sk_live_',
    subscriptionId: 'sub_001',
    listingTitle: '企业工商风险数据',
    permissions: ['read'],
    status: 'DISABLED',
    createdAt: '2026-02-15 14:30:00',
    expiresAt: '2026-05-28 23:59:59',
    lastUsedAt: '2026-03-20 10:00:00',
    totalCalls: 3200,
    ipWhitelist: [],
  },
]

const STATUS_CONFIG = {
  ACTIVE: { label: '活跃', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  DISABLED: { label: '已禁用', color: 'bg-gray-100 text-gray-800', icon: XCircle },
  EXPIRED: { label: '已过期', color: 'bg-red-100 text-red-800', icon: AlertCircle },
}

export default function BuyerApiKeysPage() {
  const [apiKeys, setApiKeys] = useState(MOCK_API_KEYS)
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showKeyModal, setShowKeyModal] = useState(false)
  const [newKeyData, setNewKeyData] = useState<any>(null)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const handleCopyKey = (key: string) => {
    navigator.clipboard.writeText(key)
    setCopiedKey(key)
    setTimeout(() => setCopiedKey(null), 2000)
  }

  const handleCreateKey = () => {
    const newKey = {
      id: `key_${Date.now()}`,
      name: '新 API Key',
      keyPrefix: 'sk_live_',
      fullKey: `sk_live_${Math.random().toString(36).substring(2, 15)}${Math.random().toString(36).substring(2, 15)}`,
      subscriptionId: 'sub_001',
      listingTitle: '企业工商风险数据',
      permissions: ['read'],
      status: 'ACTIVE' as const,
      createdAt: new Date().toISOString(),
      expiresAt: null,
      lastUsedAt: null,
      totalCalls: 0,
      ipWhitelist: [],
    }
    setNewKeyData(newKey)
    setShowKeyModal(true)
    setShowCreateModal(false)
  }

  const handleDisableKey = (keyId: string) => {
    setApiKeys(apiKeys.map(key => 
      key.id === keyId ? { ...key, status: 'DISABLED' as const } : key
    ))
  }

  const handleDeleteKey = (keyId: string) => {
    if (confirm('确定要删除这个 API Key 吗？此操作不可撤销。')) {
      setApiKeys(apiKeys.filter(key => key.id !== keyId))
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
        {/* 页面标题 */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">API 密钥管理</h1>
            <p className="text-gray-600">管理您的 API 访问密钥，确保数据安全</p>
          </div>
          {/* 权限控制：只有具有写权限的用户才能创建 API Key */}
          <PermissionGate requiredPermission="buyer:api-keys:write">
            <button
              onClick={() => setShowCreateModal(true)}
              className="flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
            >
              <Plus className="w-5 h-5" />
              <span>创建 API Key</span>
            </button>
          </PermissionGate>
        </div>

        {/* 安全提示 */}
        <div className="bg-yellow-50 border border-yellow-200 rounded-xl p-6 mb-6">
          <div className="flex gap-4">
            <Shield className="w-6 h-6 text-yellow-600 flex-shrink-0" />
            <div>
              <h3 className="font-bold text-yellow-900 mb-2">安全提示</h3>
              <ul className="text-sm text-yellow-800 space-y-1">
                <li>• API Key 创建后仅显示一次，请妥善保存</li>
                <li>• 不要在公开代码库中提交 API Key</li>
                <li>• 建议定期轮换 API Key</li>
                <li>• 使用 IP 白名单限制访问来源</li>
                <li>• 所有 API Key 操作都会记录审计日志</li>
              </ul>
            </div>
          </div>
        </div>

        {/* API Key 列表 */}
        <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">名称</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">Key</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">订阅商品</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th>
                <th className="text-center py-4 px-6 text-sm font-medium text-gray-700">调用次数</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">最近使用</th>
                <th className="text-left py-4 px-6 text-sm font-medium text-gray-700">过期时间</th>
                <th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-200">
              {apiKeys.map((apiKey) => {
                const statusConfig = STATUS_CONFIG[apiKey.status]
                const StatusIcon = statusConfig.icon
                const maskedKey = `${apiKey.keyPrefix}••••••••${apiKey.keyPrefix.slice(-4)}`

                return (
                  <tr key={apiKey.id} className="hover:bg-gray-50">
                    <td className="py-4 px-6">
                      <div className="font-medium text-gray-900">{apiKey.name}</div>
                      <div className="text-xs text-gray-500 mt-1">
                        创建于 {apiKey.createdAt.split(' ')[0]}
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      <code className="font-mono text-sm text-gray-900 bg-gray-50 px-2 py-1 rounded">
                        {maskedKey}
                      </code>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">{apiKey.listingTitle}</div>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className={`status-tag ${statusConfig.color}`}>
                        <StatusIcon className="w-3.5 h-3.5" />
                        <span>{statusConfig.label}</span>
                      </span>
                    </td>
                    <td className="py-4 px-6 text-center">
                      <span className="font-medium text-gray-900">{apiKey.totalCalls.toLocaleString()}</span>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">
                        {apiKey.lastUsedAt ? (
                          <span>{apiKey.lastUsedAt.split(' ')[0]}</span>
                        ) : (
                          <span className="text-gray-400">从未使用</span>
                        )}
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="text-sm text-gray-900">
                        {apiKey.expiresAt ? (
                          <span>{apiKey.expiresAt.split(' ')[0]}</span>
                        ) : (
                          <span className="text-gray-400">无限期</span>
                        )}
                      </div>
                    </td>
                    <td className="py-4 px-6">
                      <div className="flex items-center justify-end gap-2">
                        {/* 权限控制：只有具有写权限的用户才能操作 API Key */}
                        <PermissionGate requiredPermission="buyer:api-keys:write">
                          {apiKey.status === 'ACTIVE' && (
                            <>
                              <button
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

      {/* 权限控制：只有具有写权限的用户才能看到创建 Modal */}
      <PermissionGate requiredPermission="buyer:api-keys:write">
        {/* 创建 API Key Modal */}
      {showCreateModal && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white rounded-xl p-8 max-w-2xl w-full mx-4 animate-fade-in">
            <h2 className="text-2xl font-bold text-gray-900 mb-6">创建 API Key</h2>

            <div className="space-y-4 mb-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Key 名称 <span className="text-red-500">*</span>
                </label>
                <input
                  type="text"
                  placeholder="例如：生产环境 - 企业风险数据"
                  className="input"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  关联订阅 <span className="text-red-500">*</span>
                </label>
                <select className="input">
                  <option>企业工商风险数据 - 标准版</option>
                  <option>消费者行为分析数据 - 企业版</option>
                  <option>物流轨迹实时数据 - 按量计费</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  过期时间
                </label>
                <select className="input">
                  <option value="">无限期</option>
                  <option value="30">30 天</option>
                  <option value="90">90 天</option>
                  <option value="180">180 天</option>
                  <option value="365">365 天</option>
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  IP 白名单（可选）
                </label>
                <textarea
                  placeholder="每行一个 IP 地址，例如：&#10;192.168.1.100&#10;10.0.0.50"
                  className="input min-h-[100px]"
                />
                <p className="text-xs text-gray-500 mt-1">留空表示不限制 IP</p>
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => setShowCreateModal(false)}
                className="flex-1 px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium"
              >
                取消
              </button>
              <button
                onClick={handleCreateKey}
                className="flex-1 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 显示新创建的 Key Modal */}
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
                  <p>此 API Key 仅显示一次，关闭后将无法再次查看。请立即复制并保存到安全的地方。</p>
                </div>
              </div>

              <div className="bg-white rounded-lg p-4">
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium text-gray-700">API Key</span>
                  <button
                    onClick={() => handleCopyKey(newKeyData.fullKey)}
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
                <code className="block font-mono text-sm text-gray-900 bg-gray-50 px-3 py-2 rounded break-all">
                  {newKeyData.fullKey}
                </code>
              </div>
            </div>

            <button
              onClick={() => {
                setShowKeyModal(false)
                setNewKeyData(null)
                setApiKeys([newKeyData, ...apiKeys])
              }}
              className="w-full px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
            >
              我已保存，关闭
            </button>
          </div>
        </div>
      )}
      </PermissionGate>
    </>
  )
}
