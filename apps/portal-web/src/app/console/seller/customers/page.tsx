'use client'

import { useMemo, useState } from 'react'
import { useRouter } from 'next/navigation'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import InlineExpandableList from '@/components/console/InlineExpandableList'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import {
  Users,
  TrendingUp,
  DollarSign,
  Activity,
  Mail,
  Phone,
  Calendar,
  Package,
  AlertCircle,
  CheckCircle,
  Clock,
  Filter,
  Layers,
  ArrowUpDown,
  ExternalLink,
  type LucideIcon,
} from 'lucide-react'
import { SELLER_CUSTOMERS, type SellerCustomer } from '@/lib/seller-customers-data'

type GroupBy = 'none' | 'status' | 'industry'
type SortBy = 'recent' | 'revenue_desc'
const STATUS_CONFIG: Record<SellerCustomer['status'], { label: string; color: string; icon: LucideIcon }> = {
  ACTIVE: { label: '活跃', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  INACTIVE: { label: '不活跃', color: 'bg-gray-100 text-gray-800', icon: Clock },
  SUSPENDED: { label: '已暂停', color: 'bg-red-100 text-red-800', icon: AlertCircle },
}

const RISK_CONFIG = {
  LOW: { label: '低风险', color: 'text-green-700 bg-green-50 border border-green-200' },
  MEDIUM: { label: '中风险', color: 'text-yellow-700 bg-yellow-50 border border-yellow-200' },
  HIGH: { label: '高风险', color: 'text-red-700 bg-red-50 border border-red-200' },
}

export default function SellerCustomersPage() {
  const router = useRouter()
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedIndustry, setSelectedIndustry] = useState<string>('all')
  const [groupBy, setGroupBy] = useState<GroupBy>('none')
  const [sortBy, setSortBy] = useState<SortBy>('recent')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredCustomers = useMemo(() => {
    const key = searchKeyword.trim().toLowerCase()
    const list = SELLER_CUSTOMERS.filter((customer) => {
      const matchesKeyword =
        !key ||
        customer.name.toLowerCase().includes(key) ||
        customer.contactPerson.toLowerCase().includes(key) ||
        customer.subjectId.toLowerCase().includes(key)
      const matchesStatus = selectedStatus === 'all' || customer.status === selectedStatus
      const matchesIndustry = selectedIndustry === 'all' || customer.industry === selectedIndustry
      return matchesKeyword && matchesStatus && matchesIndustry
    })

    list.sort((a, b) => {
      if (sortBy === 'revenue_desc') return b.totalRevenue - a.totalRevenue
      return new Date(b.lastActiveDate).getTime() - new Date(a.lastActiveDate).getTime()
    })

    return list
  }, [searchKeyword, selectedStatus, selectedIndustry, sortBy])

  const pagedCustomers = useMemo(() => {
    const start = (page - 1) * pageSize
    return filteredCustomers.slice(start, start + pageSize)
  }, [filteredCustomers, page, pageSize])

  const groupedCustomers = useMemo(() => {
    if (groupBy === 'none') return [{ label: '全部结果', items: pagedCustomers }]
    const map = new Map<string, SellerCustomer[]>()
    for (const c of pagedCustomers) {
      const key = groupBy === 'status' ? STATUS_CONFIG[c.status].label : c.industry
      map.set(key, [...(map.get(key) || []), c])
    }
    return Array.from(map.entries()).map(([label, items]) => ({ label, items }))
  }, [pagedCustomers, groupBy])

  const stats = {
    total: SELLER_CUSTOMERS.length,
    active: SELLER_CUSTOMERS.filter((c) => c.status === 'ACTIVE').length,
    totalRevenue: SELLER_CUSTOMERS.reduce((sum, c) => sum + c.totalRevenue, 0),
    totalCalls: SELLER_CUSTOMERS.reduce((sum, c) => sum + c.totalCalls, 0),
  }

  return (
    <>
      <SessionIdentityBar subjectName="天眼数据科技有限公司" roleName="供应商管理员" tenantId="tenant_supplier_001" scope="seller:customers:read" sessionExpiresAt={sessionExpiresAt} userName="管理员" />

      <ConsoleListPageShell
        title="订阅客户"
        subtitle="统一查看客户状态、订阅规模、收入贡献与调用质量"
        summaryCards={<div className="grid grid-cols-1 md:grid-cols-4 gap-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="flex items-center justify-between mb-4"><div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center"><Users className="w-6 h-6 text-blue-600" /></div></div><div className="text-2xl font-bold text-gray-900 mb-1">{stats.total}</div><div className="text-sm text-gray-600">总客户数</div><div className="mt-2 text-xs text-green-600">活跃: {stats.active}</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="flex items-center justify-between mb-4"><div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center"><DollarSign className="w-6 h-6 text-green-600" /></div></div><div className="text-2xl font-bold text-gray-900 mb-1">¥{stats.totalRevenue.toLocaleString()}</div><div className="text-sm text-gray-600">累计收入</div><div className="mt-2 text-xs text-green-600 inline-flex items-center gap-1"><TrendingUp className="w-3 h-3" />+18% 本月</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="flex items-center justify-between mb-4"><div className="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center"><Activity className="w-6 h-6 text-purple-600" /></div></div><div className="text-2xl font-bold text-gray-900 mb-1">{(stats.totalCalls / 1000).toFixed(0)}K</div><div className="text-sm text-gray-600">总调用次数</div><div className="mt-2 text-xs text-green-600 inline-flex items-center gap-1"><TrendingUp className="w-3 h-3" />+25% 本月</div></div>
          <div className="bg-white rounded-xl border border-gray-200 p-6"><div className="flex items-center justify-between mb-4"><div className="w-12 h-12 bg-yellow-50 rounded-lg flex items-center justify-center"><Package className="w-6 h-6 text-yellow-600" /></div></div><div className="text-2xl font-bold text-gray-900 mb-1">{SELLER_CUSTOMERS.reduce((sum, c) => sum + c.activeSubscriptions, 0)}</div><div className="text-sm text-gray-600">活跃订阅数</div><div className="mt-2 text-xs text-gray-500">总订阅: {SELLER_CUSTOMERS.reduce((sum, c) => sum + c.subscriptionCount, 0)}</div></div>
        </div>}
        toolbar={<QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索客户名称、联系人、Subject ID..."
          onReset={() => {
            setSearchKeyword('')
            setSelectedStatus('all')
            setSelectedIndustry('all')
            setGroupBy('none')
            setSortBy('recent')
            setPage(1)
            setPageSize(10)
          }}
          controls={
            <div className="grid grid-cols-1 md:grid-cols-5 gap-3">
              <select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="ACTIVE">活跃</option><option value="INACTIVE">不活跃</option><option value="SUSPENDED">已暂停</option></select>
              <select value={selectedIndustry} onChange={(e) => setSelectedIndustry(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部行业</option><option value="金融">金融</option><option value="物流">物流</option><option value="企业服务">企业服务</option><option value="数据服务">数据服务</option></select>
              <select value={groupBy} onChange={(e) => setGroupBy(e.target.value as GroupBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="none">不分组</option><option value="status">按状态分组</option><option value="industry">按行业分组</option></select>
              <select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select>
              <select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="recent">最近活跃优先</option><option value="revenue_desc">收入贡献优先</option></select>
            </div>
          }
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filteredCustomers.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分组 {groupedCustomers.length}</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'recent' ? '最近活跃' : '收入贡献'}</span></>}
        />}
        content={<>
          {groupedCustomers.map((group) => (
            <section key={group.label}>
              {groupBy !== 'none' && <div className="mb-2 text-sm font-semibold text-gray-700">{group.label} <span className="text-gray-400 font-normal">({group.items.length})</span></div>}
              <InlineExpandableList
                items={group.items}
                getKey={(customer) => customer.id}
                selectedKey={selectedId}
                onSelect={setSelectedId}
                getItemId={(customer) => `seller-customer-card-${customer.id}`}
                onOpenDetail={(customer) => router.push(`/console/seller/customers/${customer.subjectId}`)}
                renderSummary={(customer) => {
                  const StatusIcon = STATUS_CONFIG[customer.status].icon
                  return (
                    <>
                      <div className="flex items-start justify-between mb-3">
                        <div>
                          <h3 className="text-lg font-bold text-gray-900 mb-1">{customer.name}</h3>
                          <div className="text-sm text-gray-600">{customer.contactPerson} · {customer.industry}</div>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className={`status-tag ${STATUS_CONFIG[customer.status].color}`}><StatusIcon className="w-3.5 h-3.5" /><span>{STATUS_CONFIG[customer.status].label}</span></span>
                          <span className={`text-xs px-2 py-1 rounded-full font-medium ${RISK_CONFIG[customer.riskLevel].color}`}>{RISK_CONFIG[customer.riskLevel].label}</span>
                        </div>
                      </div>
                      <div className="grid grid-cols-4 gap-4 mb-4 pb-4 border-b border-gray-100">
                        <div><div className="text-xs text-gray-500 mb-1">订阅数</div><div className="text-lg font-semibold text-gray-900">{customer.activeSubscriptions}/{customer.subscriptionCount}</div></div>
                        <div><div className="text-xs text-gray-500 mb-1">累计收入</div><div className="text-lg font-semibold text-gray-900">¥{(customer.totalRevenue / 1000).toFixed(1)}K</div></div>
                        <div><div className="text-xs text-gray-500 mb-1">总调用</div><div className="text-lg font-semibold text-gray-900">{(customer.totalCalls / 1000).toFixed(0)}K</div></div>
                        <div><div className="text-xs text-gray-500 mb-1">成功率</div><div className="text-lg font-semibold text-green-600">{customer.successRate}%</div></div>
                      </div>
                      <div className="flex items-center justify-between text-xs text-gray-500"><div className="inline-flex items-center gap-1"><Calendar className="w-3 h-3" /><span>首次订阅：{customer.firstSubscribeDate}</span></div><div className="inline-flex items-center gap-1"><Activity className="w-3 h-3" /><span>最近活跃：{customer.lastActiveDate.split(' ')[0]}</span></div></div>
                    </>
                  )
                }}
                renderExpanded={(customer) => (
                  <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
                    <div className="space-y-4">
                      <div><div className="text-xs text-gray-500 mb-1">Subject ID</div><code className="block bg-gray-50 text-xs px-2 py-1 rounded text-gray-900 break-all">{customer.subjectId}</code></div>
                      <div className="text-sm space-y-2">
                        <div className="flex justify-between"><span className="text-gray-600">客户类型</span><span className="font-medium text-gray-900">{customer.type === 'ENTERPRISE' ? '企业客户' : '个人客户'}</span></div>
                        <div className="flex justify-between"><span className="text-gray-600">平均响应</span><span className="font-medium text-gray-900">{customer.avgResponseTime}ms</span></div>
                        <div className="flex justify-between"><span className="text-gray-600">成功率</span><span className="font-medium text-green-600">{customer.successRate}%</span></div>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <button onClick={(e) => { e.stopPropagation(); router.push(`/console/seller/customers/${customer.subjectId}`) }} className="w-full h-10 px-4 border border-primary-300 text-primary-700 rounded-lg hover:bg-primary-50 text-sm font-medium inline-flex items-center justify-center gap-2"><ExternalLink className="w-4 h-4" /><span>进入客户详情页</span></button>
                      <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 bg-primary-600 text-white rounded-lg hover:bg-primary-700 text-sm font-medium inline-flex items-center justify-center gap-2"><Mail className="w-4 h-4" /><span>发送消息</span></button>
                      <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center justify-center gap-2"><Phone className="w-4 h-4" /><span>联系客户</span></button>
                      <button onClick={(e) => e.stopPropagation()} className="w-full h-10 px-4 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 text-sm font-medium">查看订阅详情</button>
                    </div>
                  </div>
                )}
              />
            </section>
          ))}
        </>}
        pagination={<PaginationBar page={page} pageSize={pageSize} total={filteredCustomers.length} onPageChange={setPage} onPageSizeChange={setPageSize} />}
      />
    </>
  )
}
