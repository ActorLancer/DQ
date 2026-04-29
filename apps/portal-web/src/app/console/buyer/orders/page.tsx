'use client'

import { useEffect, useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import InlineExpandableList from '@/components/console/InlineExpandableList'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import { Filter, DollarSign, Download, CheckCircle, Clock, XCircle, Calendar, CreditCard, Receipt, FileText, Layers, ArrowUpDown } from 'lucide-react'
import { INVOICE_STATUS_CONFIG, MOCK_ORDERS, ORDER_STATUS_CONFIG, ORDER_TYPE_CONFIG, Order } from '@/lib/buyer-orders-data'
import { getBuyerApiKeys } from '@/lib/buyer-api-keys-storage'

const ORDER_STATUS_ICON = { PENDING_PAYMENT: Clock, PAID: CheckCircle, CANCELLED: XCircle, REFUNDED: XCircle }
type GroupBy = 'none' | 'status' | 'invoice'
type SortBy = 'created_desc' | 'amount_desc' | 'paid_desc'

export default function BuyerOrdersPage() {
  const router = useRouter()
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedInvoiceStatus, setSelectedInvoiceStatus] = useState<string>('all')
  const [selectedType, setSelectedType] = useState<string>('all')
  const [groupBy, setGroupBy] = useState<GroupBy>('none')
  const [sortBy, setSortBy] = useState<SortBy>('created_desc')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [selectedOrderId, setSelectedOrderId] = useState<string | null>(null)
  const [highlightedOrderId, setHighlightedOrderId] = useState<string | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredOrders = useMemo(() => {
    const list = MOCK_ORDERS.filter((order) => {
      const kw = searchKeyword.toLowerCase()
      const matchesKeyword = order.listingTitle.toLowerCase().includes(kw) || order.supplierName.toLowerCase().includes(kw) || order.orderId.toLowerCase().includes(kw)
      const matchesStatus = selectedStatus === 'all' || order.status === selectedStatus
      const matchesInvoice = selectedInvoiceStatus === 'all' || order.invoiceStatus === selectedInvoiceStatus
      const matchesType = selectedType === 'all' || order.orderType === selectedType
      return matchesKeyword && matchesStatus && matchesInvoice && matchesType
    })
    return [...list].sort((a, b) => sortBy === 'amount_desc' ? b.amount - a.amount : sortBy === 'paid_desc' ? new Date(b.paidAt || 0).getTime() - new Date(a.paidAt || 0).getTime() : new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
  }, [searchKeyword, selectedStatus, selectedInvoiceStatus, selectedType, sortBy])

  const pagedOrders = useMemo(() => {
    const start = (page - 1) * pageSize
    return filteredOrders.slice(start, start + pageSize)
  }, [filteredOrders, page, pageSize])

  const groupedOrders = useMemo(() => {
    if (groupBy === 'none') return [{ label: '全部结果', items: pagedOrders }]
    const map = new Map<string, Order[]>()
    for (const order of pagedOrders) {
      const key = groupBy === 'status' ? ORDER_STATUS_CONFIG[order.status].label : INVOICE_STATUS_CONFIG[order.invoiceStatus].label
      map.set(key, [...(map.get(key) || []), order])
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }))
  }, [pagedOrders, groupBy])

  const totalSpent = MOCK_ORDERS.filter((o) => o.status === 'PAID').reduce((sum, o) => sum + o.amount, 0)
  const pendingPayment = MOCK_ORDERS.filter((o) => o.status === 'PENDING_PAYMENT').reduce((sum, o) => sum + o.amount, 0)

  useEffect(() => { setPage(1) }, [searchKeyword, selectedStatus, selectedInvoiceStatus, selectedType, groupBy, sortBy, pageSize])

  useEffect(() => {
    const params = new URLSearchParams(window.location.search)
    const focusOrderId = params.get('focus')
    if (!focusOrderId) return
    const matched = MOCK_ORDERS.find((o) => o.orderId === focusOrderId)
    if (!matched) return
    setSelectedOrderId(matched.id)
    setHighlightedOrderId(matched.id)
    requestAnimationFrame(() => document.getElementById(`order-card-${matched.id}`)?.scrollIntoView({ behavior: 'smooth', block: 'center' }))
    const timer = window.setTimeout(() => setHighlightedOrderId(null), 2400)
    return () => window.clearTimeout(timer)
  }, [])

  return (
    <>
      <SessionIdentityBar subjectName="某某科技有限公司" roleName="买家管理员" tenantId="tenant_buyer_001" scope="buyer:orders:read" sessionExpiresAt={sessionExpiresAt} />
      <div className="p-8">
        <div className="mb-8"><h1 className="text-3xl font-bold text-gray-900 mb-2">订单账单</h1><p className="text-gray-600">管理订单记录、账单与发票信息</p></div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center mb-4"><DollarSign className="w-6 h-6 text-green-600" /></div><div className="text-2xl font-semibold text-gray-900 mb-1">¥{totalSpent.toLocaleString()}</div><div className="text-sm text-gray-600">累计支出</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="w-12 h-12 bg-yellow-50 rounded-lg flex items-center justify-center mb-4"><Clock className="w-6 h-6 text-yellow-600" /></div><div className="text-2xl font-semibold text-gray-900 mb-1">¥{pendingPayment.toLocaleString()}</div><div className="text-sm text-gray-600">待支付</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center mb-4"><Receipt className="w-6 h-6 text-blue-600" /></div><div className="text-2xl font-semibold text-gray-900 mb-1">{MOCK_ORDERS.filter((o) => o.invoiceStatus === 'ISSUED').length}</div><div className="text-sm text-gray-600">已开发票</div></div>
        </div>

        <QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索订单号、商品、供应商..."
          onReset={() => { setSearchKeyword(''); setSelectedStatus('all'); setSelectedInvoiceStatus('all'); setSelectedType('all'); setGroupBy('none'); setSortBy('created_desc'); setPage(1); setPageSize(10) }}
          controls={<div className="grid grid-cols-1 md:grid-cols-6 gap-3"><select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部订单状态</option><option value="PENDING_PAYMENT">待支付</option><option value="PAID">已支付</option><option value="CANCELLED">已取消</option><option value="REFUNDED">已退款</option></select><select value={selectedInvoiceStatus} onChange={(e) => setSelectedInvoiceStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部发票状态</option><option value="NOT_REQUESTED">未申请</option><option value="REQUESTED">已申请</option><option value="ISSUED">已开具</option></select><select value={selectedType} onChange={(e) => setSelectedType(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部订单类型</option><option value="SUBSCRIPTION">订阅</option><option value="ONE_TIME">一次性</option><option value="RENEWAL">续订</option></select><select value={groupBy} onChange={(e) => setGroupBy(e.target.value as GroupBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="none">不分组</option><option value="status">按订单状态分组</option><option value="invoice">按发票状态分组</option></select><select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select><select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="created_desc">创建时间优先</option><option value="amount_desc">金额优先</option><option value="paid_desc">支付时间优先</option></select></div>}
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filteredOrders.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分组 {groupedOrders.length}</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'created_desc' ? '创建时间' : sortBy === 'amount_desc' ? '金额' : '支付时间'}</span></>}
        />

        <div className="space-y-5">
          {groupedOrders.map((group) => (
            <section key={group.label}>
              {groupBy !== 'none' && <div className="mb-2 text-sm font-semibold text-gray-700">{group.label} <span className="text-gray-400 font-normal">({group.items.length})</span></div>}
              <InlineExpandableList items={group.items} getKey={(order) => order.id} selectedKey={selectedOrderId} onSelect={setSelectedOrderId} highlightedKey={highlightedOrderId} getItemId={(order) => `order-card-${order.id}`} onOpenDetail={(order) => router.push(`/console/buyer/orders/${order.orderId}`)}
                renderSummary={(order) => { const statusConfig = ORDER_STATUS_CONFIG[order.status]; const StatusIcon = ORDER_STATUS_ICON[order.status]; return (<><div className="flex items-start justify-between mb-4"><div className="flex-1"><div className="flex items-center gap-2 mb-2"><h3 className="text-lg font-bold text-gray-900">{order.listingTitle}</h3><span className={`status-tag text-xs ${ORDER_TYPE_CONFIG[order.orderType].color}`}>{ORDER_TYPE_CONFIG[order.orderType].label}</span></div><div className="text-sm text-gray-600">{order.supplierName} · <span className="font-medium text-gray-900">{order.plan}</span></div></div><span className={`status-tag ${statusConfig.color}`}><StatusIcon className="w-3.5 h-3.5" /><span>{statusConfig.label}</span></span></div><div className="grid grid-cols-2 gap-4 mb-4 pb-4 border-b border-gray-100"><div><div className="text-xs text-gray-500 mb-1">订单金额</div><div className="text-lg font-semibold text-gray-900">¥{order.amount.toLocaleString()}</div></div><div><div className="text-xs text-gray-500 mb-1">订单号</div><code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block truncate">{order.orderId}</code></div></div><div className="flex items-center justify-between text-xs"><div className="flex items-center gap-4 text-gray-500"><div className="flex items-center gap-1"><Calendar className="w-3 h-3" /><span>创建: {order.createdAt.split(' ')[0]}</span></div>{order.paidAt && <div className="flex items-center gap-1"><CheckCircle className="w-3 h-3" /><span>支付: {order.paidAt.split(' ')[0]}</span></div>}</div><span className={`status-tag text-xs ${INVOICE_STATUS_CONFIG[order.invoiceStatus].color}`}>发票: {INVOICE_STATUS_CONFIG[order.invoiceStatus].label}</span></div></>) }}
                renderExpanded={(order) => { const linkedCount = getBuyerApiKeys().filter((item) => item.orderId === order.orderId).length; return (<div className="grid grid-cols-1 lg:grid-cols-2 gap-5"><div className="space-y-4"><div><div className="text-xs text-gray-500 mb-1">Order ID</div><code className="font-mono text-xs text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">{order.orderId}</code></div><div className="text-sm space-y-2"><div className="flex justify-between"><span className="text-gray-600">订单状态</span><span className={`status-tag text-xs ${ORDER_STATUS_CONFIG[order.status].color}`}>{ORDER_STATUS_CONFIG[order.status].label}</span></div><div className="flex justify-between"><span className="text-gray-600">发票状态</span><span className={`status-tag text-xs ${INVOICE_STATUS_CONFIG[order.invoiceStatus].color}`}>{INVOICE_STATUS_CONFIG[order.invoiceStatus].label}</span></div><div className="flex justify-between"><span className="text-gray-600">支付方式</span><span className="font-medium text-gray-900">{order.paymentMethod || '待支付'}</span></div><div className="flex justify-between"><span className="text-gray-600">关联密钥</span><span className="font-medium text-gray-900">{linkedCount} 个</span></div></div></div><div className="space-y-2"><button onClick={(e) => { e.stopPropagation(); router.push(`/console/buyer/orders/${order.orderId}`) }} className="w-full h-10 px-4 border border-primary-300 text-primary-700 rounded-lg hover:bg-primary-50 text-sm font-medium">查看订单详情页</button><button onClick={(e) => { e.stopPropagation(); router.push(`/console/buyer/billing/${order.orderId}`) }} className="w-full h-10 px-4 border border-blue-300 text-blue-700 rounded-lg hover:bg-blue-50 text-sm font-medium">查看账单详情页</button>{order.status === 'PENDING_PAYMENT' && <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium inline-flex items-center justify-center gap-2"><CreditCard className="w-4 h-4" /><span>立即支付</span></button>}{order.status === 'PENDING_PAYMENT' && <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium">取消订单</button>}{order.invoiceStatus === 'ISSUED' && <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium inline-flex items-center justify-center gap-2"><Download className="w-4 h-4" /><span>下载发票</span></button>}{order.status === 'PAID' && order.invoiceStatus === 'NOT_REQUESTED' && <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium inline-flex items-center justify-center gap-2"><FileText className="w-4 h-4" /><span>申请发票</span></button>}</div></div>) }} />
            </section>
          ))}
        </div>

        <PaginationBar page={page} pageSize={pageSize} total={filteredOrders.length} onPageChange={setPage} onPageSizeChange={setPageSize} />
      </div>
    </>
  )
}
