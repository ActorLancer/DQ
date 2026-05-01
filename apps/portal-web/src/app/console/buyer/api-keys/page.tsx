'use client'

import { useEffect, useMemo, useState } from 'react'
import { useSearchParams } from 'next/navigation'
import Link from 'next/link'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import PermissionGate from '@/components/auth/PermissionGate'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import { Plus, Copy, RotateCw, Trash2, CheckCircle, XCircle, AlertCircle, Shield, Filter, Layers, ArrowUpDown } from 'lucide-react'
import { ApiKey, PAID_ORDERS } from '@/lib/buyer-api-keys-data'
import { bootstrapBuyerApiKeys, deleteBuyerApiKey, disableBuyerApiKey, enableBuyerApiKey, getBuyerApiKeys, onBuyerApiKeysUpdated, rotateBuyerApiKey, saveBuyerApiKeys } from '@/lib/buyer-api-keys-storage'

const STATUS_CONFIG = {
  ACTIVE: { label: '活跃', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  DISABLED: { label: '已禁用', color: 'bg-gray-100 text-gray-800', icon: XCircle },
  EXPIRED: { label: '已过期', color: 'bg-red-100 text-red-800', icon: AlertCircle },
}

type GroupBy = 'none' | 'status' | 'order'
type SortBy = 'created_desc' | 'last_used_desc' | 'calls_desc'

export default function BuyerApiKeysPage() {
  const searchParams = useSearchParams()
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([])
  const [showCreateModal, setShowCreateModal] = useState(false)
  const [showKeyModal, setShowKeyModal] = useState(false)
  const [newKeyData, setNewKeyData] = useState<ApiKey | null>(null)
  const [copiedKey, setCopiedKey] = useState<string | null>(null)
  const [listToast, setListToast] = useState<string | null>(null)
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<'all' | ApiKey['status']>('all')
  const [selectedOrderId, setSelectedOrderId] = useState<'all' | string>('all')
  const [groupBy, setGroupBy] = useState<GroupBy>('none')
  const [sortBy, setSortBy] = useState<SortBy>('created_desc')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [form, setForm] = useState({ name: '', orderId: PAID_ORDERS[0]?.orderId ?? '', expireDays: '', whitelist: '' })

  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()
  const paidOrders = useMemo(() => PAID_ORDERS, [])

  useEffect(() => {
    bootstrapBuyerApiKeys()
    setApiKeys(getBuyerApiKeys())
    const off = onBuyerApiKeysUpdated(() => setApiKeys(getBuyerApiKeys()))
    return off
  }, [])

  useEffect(() => {
    const orderId = searchParams.get('orderId')
    if (!orderId) return
    const matched = paidOrders.find((order) => order.orderId === orderId)
    if (!matched) return
    setForm((prev) => ({ ...prev, orderId: matched.orderId }))
  }, [searchParams, paidOrders])

  useEffect(() => {
    const flash = searchParams.get('flash')
    if (!flash) return
    const messageMap: Record<string, string> = { rotated: 'API 密钥已轮换', disabled: 'API 密钥已禁用', enabled: 'API 密钥已启用', deleted: 'API 密钥已删除' }
    if (!messageMap[flash]) return
    setListToast(messageMap[flash])
    const timer = window.setTimeout(() => setListToast(null), 2200)
    return () => window.clearTimeout(timer)
  }, [searchParams])

  useEffect(() => { setPage(1) }, [searchKeyword, selectedStatus, selectedOrderId, groupBy, sortBy, pageSize])

  const filteredKeys = useMemo(() => {
    const list = apiKeys.filter((key) => {
      const kw = searchKeyword.toLowerCase()
      const matchesKeyword = key.name.toLowerCase().includes(kw) || key.listingTitle.toLowerCase().includes(kw) || key.orderId.toLowerCase().includes(kw)
      const matchesStatus = selectedStatus === 'all' || key.status === selectedStatus
      const matchesOrder = selectedOrderId === 'all' || key.orderId === selectedOrderId
      return matchesKeyword && matchesStatus && matchesOrder
    })
    return [...list].sort((a, b) => {
      if (sortBy === 'calls_desc') return b.totalCalls - a.totalCalls
      if (sortBy === 'last_used_desc') return new Date(b.lastUsedAt || 0).getTime() - new Date(a.lastUsedAt || 0).getTime()
      return new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    })
  }, [apiKeys, searchKeyword, selectedStatus, selectedOrderId, sortBy])

  const pagedKeys = useMemo(() => {
    const start = (page - 1) * pageSize
    return filteredKeys.slice(start, start + pageSize)
  }, [filteredKeys, page, pageSize])

  const groupedKeys = useMemo(() => {
    if (groupBy === 'none') return [{ label: '全部结果', items: pagedKeys }]
    const map = new Map<string, ApiKey[]>()
    for (const key of pagedKeys) {
      const groupKey = groupBy === 'status' ? STATUS_CONFIG[key.status].label : key.orderId
      map.set(groupKey, [...(map.get(groupKey) || []), key])
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }))
  }, [pagedKeys, groupBy])

  const handleCopyKey = (key: string) => { navigator.clipboard.writeText(key); setCopiedKey(key); setTimeout(() => setCopiedKey(null), 2000) }
  const handleDisableKey = (keyId: string) => disableBuyerApiKey(keyId)
  const handleRotateKey = (keyId: string) => rotateBuyerApiKey(keyId)
  const handleEnableKey = (keyId: string) => enableBuyerApiKey(keyId)
  const handleDeleteKey = (keyId: string) => { if (confirm('确定要删除这个 API Key 吗？此操作不可撤销。')) deleteBuyerApiKey(keyId) }

  const handleCreateKey = () => {
    const order = paidOrders.find((item) => item.orderId === form.orderId)
    if (!order || !form.name.trim()) return
    const expireDays = form.expireDays ? Number(form.expireDays) : null
    const expiresAt = expireDays ? new Date(Date.now() + expireDays * 24 * 60 * 60 * 1000).toISOString().slice(0, 19).replace('T', ' ') : null
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
      ipWhitelist: form.whitelist.split('\n').map((v) => v.trim()).filter(Boolean),
    }
    setNewKeyData(key)
    setShowCreateModal(false)
    setShowKeyModal(true)
  }

  return (
    <>
      <SessionIdentityBar subjectName="某某科技有限公司" roleName="买家管理员" tenantId="tenant_buyer_001" scope="buyer:api-keys:write" sessionExpiresAt={sessionExpiresAt} />
      <ConsoleListPageShell
        title="API 密钥管理"
        subtitle="支持按订单、状态、调用活跃度筛选与分组管理"
        headerAction={<PermissionGate requiredPermission="buyer:api-keys:write"><button onClick={() => setShowCreateModal(true)} disabled={paidOrders.length === 0} className="flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium disabled:opacity-50"><Plus className="w-5 h-5" /><span>创建 API Key</span></button></PermissionGate>}
        summaryCards={<div className="bg-yellow-50 border border-yellow-200 rounded-xl p-6"><div className="flex gap-4"><Shield className="w-6 h-6 text-yellow-600 flex-shrink-0" /><div><h3 className="font-bold text-yellow-900 mb-2">安全提示</h3><ul className="text-sm text-yellow-800 space-y-1"><li>• API Key 必须绑定已支付订单，自动继承订单可用范围</li><li>• 创建后仅显示一次，请妥善保存</li><li>• 建议定期轮换并配置 IP 白名单</li><li>• 轮换、复制、禁用、删除均记录审计日志</li></ul></div></div></div>}
        toolbar={<QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索名称、订单号、数据商品..."
          onReset={() => { setSearchKeyword(''); setSelectedStatus('all'); setSelectedOrderId('all'); setGroupBy('none'); setSortBy('created_desc'); setPage(1); setPageSize(10) }}
          controls={<div className="grid grid-cols-1 md:grid-cols-5 gap-3"><select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value as any)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="ACTIVE">活跃</option><option value="DISABLED">已禁用</option><option value="EXPIRED">已过期</option></select><select value={selectedOrderId} onChange={(e) => setSelectedOrderId(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部订单</option>{paidOrders.map((o) => <option key={o.orderId} value={o.orderId}>{o.orderId}</option>)}</select><select value={groupBy} onChange={(e) => setGroupBy(e.target.value as GroupBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="none">不分组</option><option value="status">按状态分组</option><option value="order">按订单分组</option></select><select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select><select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="created_desc">创建时间优先</option><option value="last_used_desc">最近使用优先</option><option value="calls_desc">调用次数优先</option></select></div>}
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filteredKeys.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分组 {groupedKeys.length}</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'created_desc' ? '创建时间' : sortBy === 'last_used_desc' ? '最近使用' : '调用次数'}</span></>}
        />}
        content={<>
          {groupedKeys.map((group) => (
            <section key={group.label} className="bg-white rounded-xl border border-gray-200 overflow-hidden">
              {groupBy !== 'none' && <div className="px-6 py-3 bg-gray-50 border-b border-gray-200 text-sm font-semibold text-gray-700">{group.label} <span className="text-gray-400 font-normal">({group.items.length})</span></div>}
              <table className="w-full">
                <thead className="bg-gray-50 border-b border-gray-200"><tr><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">名称</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">Key</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">最近使用</th><th className="text-right py-4 px-6 text-sm font-medium text-gray-700">操作</th></tr></thead>
                <tbody className="divide-y divide-gray-200">
                  {group.items.map((apiKey) => {
                    const statusConfig = STATUS_CONFIG[apiKey.status]
                    const StatusIcon = statusConfig.icon
                    const maskedKey = `${apiKey.keyPrefix}••••••••${apiKey.id.slice(-4)}`
                    return (
                      <tr key={apiKey.id} className="hover:bg-gray-50">
                        <td className="py-4 px-6"><div className="font-medium text-gray-900">{apiKey.name}</div><div className="text-xs text-gray-500 mt-0.5">{apiKey.orderId}</div></td>
                        <td className="py-4 px-6"><code className="font-mono text-sm text-gray-900 bg-gray-50 px-2 py-1 rounded">{maskedKey}</code></td>
                        <td className="py-4 px-6 text-center"><span className={`status-tag ${statusConfig.color}`}><StatusIcon className="w-3.5 h-3.5" /><span>{statusConfig.label}</span></span></td>
                        <td className="py-4 px-6 text-sm text-gray-900">{apiKey.lastUsedAt ? apiKey.lastUsedAt.split(' ')[0] : '从未使用'}</td>
                        <td className="py-4 px-6"><div className="flex items-center justify-end gap-2"><Link href={`/console/buyer/api-keys/${apiKey.id}`} className="px-3 py-1.5 text-xs font-medium rounded-md border border-gray-300 text-gray-700 hover:bg-gray-50">详情</Link><PermissionGate requiredPermission="buyer:api-keys:write">{apiKey.status === 'ACTIVE' && <button onClick={() => handleRotateKey(apiKey.id)} className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-md border border-primary-300 text-primary-700 hover:bg-primary-50"><RotateCw className="w-3.5 h-3.5" /><span>轮换</span></button>}{apiKey.status === 'ACTIVE' && <button onClick={() => handleDisableKey(apiKey.id)} className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-md border border-orange-300 text-orange-700 hover:bg-orange-50"><XCircle className="w-3.5 h-3.5" /><span>禁用</span></button>}{apiKey.status === 'DISABLED' && <button onClick={() => handleEnableKey(apiKey.id)} className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-md border border-green-300 text-green-700 hover:bg-green-50"><CheckCircle className="w-3.5 h-3.5" /><span>解除禁用</span></button>}<button onClick={() => handleDeleteKey(apiKey.id)} className="inline-flex items-center gap-1 px-3 py-1.5 text-xs font-medium rounded-md border border-red-300 text-red-700 hover:bg-red-50"><Trash2 className="w-3.5 h-3.5" /><span>删除</span></button></PermissionGate></div></td>
                      </tr>
                    )
                  })}
                  {group.items.length === 0 && (
                    <tr>
                      <td colSpan={5} className="py-14 text-center text-sm text-gray-500">
                        暂无符合筛选条件的 API 密钥
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </section>
          ))}
        </>}
        pagination={<PaginationBar page={page} pageSize={pageSize} total={filteredKeys.length} onPageChange={setPage} onPageSizeChange={setPageSize} />}
      />

      <PermissionGate requiredPermission="buyer:api-keys:write">
        {showCreateModal && <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"><div className="bg-white rounded-xl p-8 max-w-2xl w-full mx-4 animate-fade-in"><h2 className="text-2xl font-bold text-gray-900 mb-6">创建 API Key</h2><div className="space-y-4 mb-6"><div><label className="block text-sm font-medium text-gray-700 mb-2">Key 名称 <span className="text-red-500">*</span></label><input type="text" value={form.name} onChange={(e) => setForm((prev) => ({ ...prev, name: e.target.value }))} placeholder="例如：生产环境 - 企业风险数据" className="input" /></div><div><label className="block text-sm font-medium text-gray-700 mb-2">关联已支付订单 <span className="text-red-500">*</span></label><select value={form.orderId} onChange={(e) => setForm((prev) => ({ ...prev, orderId: e.target.value }))} className="input">{paidOrders.map((order) => <option key={order.orderId} value={order.orderId}>{order.orderId} | {order.listingTitle} | {order.plan} | ¥{order.amount.toLocaleString()}</option>)}</select></div><div><label className="block text-sm font-medium text-gray-700 mb-2">过期时间</label><select value={form.expireDays} onChange={(e) => setForm((prev) => ({ ...prev, expireDays: e.target.value }))} className="input"><option value="">无限期</option><option value="30">30 天</option><option value="90">90 天</option><option value="180">180 天</option><option value="365">365 天</option></select></div><div><label className="block text-sm font-medium text-gray-700 mb-2">IP 白名单（可选）</label><textarea value={form.whitelist} onChange={(e) => setForm((prev) => ({ ...prev, whitelist: e.target.value }))} placeholder="每行一个 IP 地址" className="input min-h-[100px]" /></div></div><div className="flex gap-3"><button onClick={() => setShowCreateModal(false)} className="flex-1 px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">取消</button><button onClick={handleCreateKey} disabled={!form.name.trim() || !form.orderId} className="flex-1 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium disabled:opacity-50">创建</button></div></div></div>}
        {showKeyModal && newKeyData && <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"><div className="bg-white rounded-xl p-8 max-w-2xl w-full mx-4 animate-fade-in"><div className="text-center mb-6"><div className="w-16 h-16 bg-success-100 rounded-full flex items-center justify-center mx-auto mb-4"><CheckCircle className="w-8 h-8 text-success-600" /></div><h2 className="text-2xl font-bold text-gray-900 mb-2">API Key 创建成功</h2><p className="text-gray-600">请立即复制并妥善保存</p></div><div className="bg-red-50 border-2 border-red-200 rounded-lg p-6 mb-6"><div className="flex items-start gap-3 mb-4"><AlertCircle className="w-5 h-5 text-red-600 flex-shrink-0 mt-0.5" /><div className="text-sm text-red-800"><p className="font-bold mb-1">重要提示</p><p>此 API Key 仅显示一次，且已绑定订单 {newKeyData.orderId}。</p></div></div><div className="bg-white rounded-lg p-4"><div className="flex items-center justify-between mb-2"><span className="text-sm font-medium text-gray-700">API Key</span><button onClick={() => handleCopyKey(newKeyData.fullKey || '')} className="flex items-center gap-2 px-3 py-1.5 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium">{copiedKey === newKeyData.fullKey ? <><CheckCircle className="w-4 h-4" /><span>已复制</span></> : <><Copy className="w-4 h-4" /><span>复制</span></>}</button></div><code className="block font-mono text-sm text-gray-900 bg-gray-50 px-3 py-2 rounded break-all">{newKeyData.fullKey}</code></div></div><button onClick={() => { setShowKeyModal(false); saveBuyerApiKeys([newKeyData, ...getBuyerApiKeys()]); setNewKeyData(null); setForm({ name: '', orderId: PAID_ORDERS[0]?.orderId ?? '', expireDays: '', whitelist: '' }) }} className="w-full px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">我已保存，关闭</button></div></div>}
      </PermissionGate>

      {listToast && <div className="fixed right-6 top-24 z-[70] animate-fade-in"><div className="rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm font-medium text-green-800 shadow-lg">{listToast}</div></div>}
    </>
  )
}
