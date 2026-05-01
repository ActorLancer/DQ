'use client'

import { useMemo, useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { QueryToolbar, PaginationBar } from '@/components/console/QueryToolbar'
import ConsoleListPageShell from '@/components/console/ConsoleListPageShell'
import { CheckCircle, XCircle, Clock, Calendar, Filter, Layers, ArrowUpDown } from 'lucide-react'

interface Subject {
  id: string
  subjectId: string
  name: string
  type: 'SUPPLIER' | 'BUYER' | 'PLATFORM'
  status: 'PENDING' | 'APPROVED' | 'REJECTED'
  creditCode: string
  legalPerson: string
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
  submittedAt: string
  reviewedAt?: string
}

const MOCK_SUBJECTS: Subject[] = [
  { id: 'subject_001', subjectId: 'subject_20260428_001', name: '某某数据科技有限公司', type: 'SUPPLIER', status: 'PENDING', creditCode: '91110000XXXXXXXXXX', legalPerson: '张三', riskLevel: 'LOW', submittedAt: '2026-04-28 10:30:00' },
  { id: 'subject_002', subjectId: 'subject_20260428_002', name: '某某金融服务公司', type: 'BUYER', status: 'PENDING', creditCode: '91110000YYYYYYYYYY', legalPerson: '王五', riskLevel: 'MEDIUM', submittedAt: '2026-04-28 09:15:00' },
  { id: 'subject_003', subjectId: 'subject_20260427_003', name: '某某物流数据中心', type: 'SUPPLIER', status: 'APPROVED', creditCode: '91110000ZZZZZZZZZZ', legalPerson: '孙七', riskLevel: 'LOW', submittedAt: '2026-04-27 16:20:00', reviewedAt: '2026-04-28 10:00:00' },
  { id: 'subject_004', subjectId: 'subject_20260427_004', name: '某某咨询服务公司', type: 'BUYER', status: 'REJECTED', creditCode: '91110000AAAAAAAAAA', legalPerson: '吴九', riskLevel: 'HIGH', submittedAt: '2026-04-27 14:00:00', reviewedAt: '2026-04-27 18:30:00' },
]

const STATUS_CONFIG = {
  PENDING: { label: '待审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: XCircle },
}

type SortBy = 'submitted_desc' | 'name_asc'

export default function AdminSubjectsPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedType, setSelectedType] = useState<string>('all')
  const [sortBy, setSortBy] = useState<SortBy>('submitted_desc')
  const [page, setPage] = useState(1)
  const [pageSize, setPageSize] = useState(10)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filtered = useMemo(() => {
    const list = MOCK_SUBJECTS.filter((subject) => {
      const kw = searchKeyword.toLowerCase()
      const matchesKeyword = subject.name.toLowerCase().includes(kw) || subject.creditCode.toLowerCase().includes(kw)
      const matchesStatus = selectedStatus === 'all' || subject.status === selectedStatus
      const matchesType = selectedType === 'all' || subject.type === selectedType
      return matchesKeyword && matchesStatus && matchesType
    })
    return [...list].sort((a, b) => sortBy === 'name_asc' ? a.name.localeCompare(b.name) : new Date(b.submittedAt).getTime() - new Date(a.submittedAt).getTime())
  }, [searchKeyword, selectedStatus, selectedType, sortBy])

  const paged = useMemo(() => filtered.slice((page - 1) * pageSize, (page - 1) * pageSize + pageSize), [filtered, page, pageSize])

  return (
    <>
      <SessionIdentityBar subjectName="数据交易平台" roleName="平台管理员" tenantId="tenant_platform_001" scope="admin:subjects:write" sessionExpiresAt={sessionExpiresAt} userName="管理员" />
      <ConsoleListPageShell
        title="主体审核"
        subtitle="审核和管理平台主体资质"
        toolbar={<QueryToolbar
          searchValue={searchKeyword}
          onSearchChange={setSearchKeyword}
          searchPlaceholder="搜索主体名称或统一社会信用代码..."
          onReset={() => { setSearchKeyword(''); setSelectedStatus('all'); setSelectedType('all'); setSortBy('submitted_desc'); setPage(1); setPageSize(10) }}
          controls={<div className="grid grid-cols-1 md:grid-cols-5 gap-3"><select value={selectedStatus} onChange={(e) => setSelectedStatus(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部状态</option><option value="PENDING">待审核</option><option value="APPROVED">已通过</option><option value="REJECTED">已拒绝</option></select><select value={selectedType} onChange={(e) => setSelectedType(e.target.value)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="all">全部类型</option><option value="SUPPLIER">供应商</option><option value="BUYER">买家</option></select><select value={pageSize} onChange={(e) => setPageSize(Number(e.target.value))} className="h-10 px-4 border border-gray-300 rounded-lg"><option value={5}>分页 5 条</option><option value={10}>分页 10 条</option><option value={20}>分页 20 条</option></select><select value={sortBy} onChange={(e) => setSortBy(e.target.value as SortBy)} className="h-10 px-4 border border-gray-300 rounded-lg"><option value="submitted_desc">提交时间优先</option><option value="name_asc">名称 A-Z</option></select><div className="h-10 px-3 rounded-lg border border-gray-200 bg-gray-50 text-xs text-gray-600 flex items-center">待审核 {filtered.filter(f=>f.status==='PENDING').length} 条</div></div>}
          stats={<><span className="inline-flex items-center gap-1"><Filter className="w-4 h-4" />结果 {filtered.length}</span><span className="inline-flex items-center gap-1"><Layers className="w-4 h-4" />分页 {pageSize}/页</span><span className="inline-flex items-center gap-1"><ArrowUpDown className="w-4 h-4" />排序 {sortBy === 'submitted_desc' ? '提交时间' : '名称 A-Z'}</span></>}
        />}
        content={<div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200"><tr><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">主体</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">类型</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">信用代码</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">法人</th><th className="text-center py-4 px-6 text-sm font-medium text-gray-700">状态</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">提交时间</th><th className="text-left py-4 px-6 text-sm font-medium text-gray-700">审核时间</th></tr></thead>
            <tbody className="divide-y divide-gray-200">
              {paged.map((subject) => {
                const cfg = STATUS_CONFIG[subject.status]
                const Icon = cfg.icon
                return (
                  <tr key={subject.id} className="hover:bg-gray-50"><td className="py-4 px-6"><div className="font-medium text-gray-900">{subject.name}</div><div className="text-xs text-gray-500">{subject.subjectId}</div></td><td className="py-4 px-6 text-sm">{subject.type}</td><td className="py-4 px-6"><code className="text-xs bg-gray-100 px-2 py-1 rounded">{subject.creditCode}</code></td><td className="py-4 px-6 text-sm">{subject.legalPerson}</td><td className="py-4 px-6 text-center"><span className={`status-tag ${cfg.color}`}><Icon className="w-3.5 h-3.5" /><span>{cfg.label}</span></span></td><td className="py-4 px-6 text-sm"><div className="inline-flex items-center gap-1"><Calendar className="w-3 h-3 text-gray-400" />{subject.submittedAt.split(' ')[0]}</div></td><td className="py-4 px-6 text-sm">{subject.reviewedAt ? subject.reviewedAt.split(' ')[0] : '-'}</td></tr>
                )
              })}
              {paged.length === 0 && (
                <tr>
                  <td colSpan={7} className="py-14 text-center text-sm text-gray-500">
                    暂无符合筛选条件的主体
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>}
        pagination={<PaginationBar page={page} pageSize={pageSize} total={filtered.length} onPageChange={setPage} onPageSizeChange={setPageSize} />}
      />
    </>
  )
}
