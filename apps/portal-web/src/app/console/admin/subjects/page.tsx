'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Search,
  Filter,
  Users,
  Building2,
  CheckCircle,
  XCircle,
  Clock,
  AlertTriangle,
  Eye,
  FileText,
  Shield,
  Calendar,
  MapPin,
  Phone,
  Mail
} from 'lucide-react'

interface Subject {
  id: string
  subjectId: string
  name: string
  type: 'SUPPLIER' | 'BUYER' | 'PLATFORM'
  status: 'PENDING' | 'APPROVED' | 'REJECTED'
  creditCode: string
  legalPerson: string
  registeredCapital: string
  registeredAddress: string
  contactPerson: string
  contactPhone: string
  contactEmail: string
  businessLicense: string
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH'
  submittedAt: string
  reviewedAt?: string
  reviewer?: string
  reviewNotes?: string
  chainStatus: 'NOT_SUBMITTED' | 'SUBMITTED' | 'CONFIRMED'
}

const MOCK_SUBJECTS: Subject[] = [
  {
    id: 'subject_001',
    subjectId: 'subject_20260428_001',
    name: '某某数据科技有限公司',
    type: 'SUPPLIER',
    status: 'PENDING',
    creditCode: '91110000XXXXXXXXXX',
    legalPerson: '张三',
    registeredCapital: '1000万元',
    registeredAddress: '北京市朝阳区某某街道某某大厦 1001 室',
    contactPerson: '李四',
    contactPhone: '13800138000',
    contactEmail: 'contact@example.com',
    businessLicense: 'license_001.pdf',
    riskLevel: 'LOW',
    submittedAt: '2026-04-28 10:30:00',
    chainStatus: 'SUBMITTED',
  },
  {
    id: 'subject_002',
    subjectId: 'subject_20260428_002',
    name: '某某金融服务公司',
    type: 'BUYER',
    status: 'PENDING',
    creditCode: '91110000YYYYYYYYYY',
    legalPerson: '王五',
    registeredCapital: '5000万元',
    registeredAddress: '上海市浦东新区某某路某某号',
    contactPerson: '赵六',
    contactPhone: '13900139000',
    contactEmail: 'info@example.com',
    businessLicense: 'license_002.pdf',
    riskLevel: 'MEDIUM',
    submittedAt: '2026-04-28 09:15:00',
    chainStatus: 'SUBMITTED',
  },
  {
    id: 'subject_003',
    subjectId: 'subject_20260427_003',
    name: '某某物流数据中心',
    type: 'SUPPLIER',
    status: 'APPROVED',
    creditCode: '91110000ZZZZZZZZZZ',
    legalPerson: '孙七',
    registeredCapital: '2000万元',
    registeredAddress: '深圳市南山区某某科技园',
    contactPerson: '周八',
    contactPhone: '13700137000',
    contactEmail: 'admin@example.com',
    businessLicense: 'license_003.pdf',
    riskLevel: 'LOW',
    submittedAt: '2026-04-27 16:20:00',
    reviewedAt: '2026-04-28 10:00:00',
    reviewer: '管理员张三',
    reviewNotes: '资质齐全，审核通过',
    chainStatus: 'CONFIRMED',
  },
  {
    id: 'subject_004',
    subjectId: 'subject_20260427_004',
    name: '某某咨询服务公司',
    type: 'BUYER',
    status: 'REJECTED',
    creditCode: '91110000AAAAAAAAAA',
    legalPerson: '吴九',
    registeredCapital: '500万元',
    registeredAddress: '广州市天河区某某大道',
    contactPerson: '郑十',
    contactPhone: '13600136000',
    contactEmail: 'service@example.com',
    businessLicense: 'license_004.pdf',
    riskLevel: 'HIGH',
    submittedAt: '2026-04-27 14:00:00',
    reviewedAt: '2026-04-27 18:30:00',
    reviewer: '管理员李四',
    reviewNotes: '企业资质不符合要求，营业执照信息与申请信息不一致',
    chainStatus: 'CONFIRMED',
  },
]

const STATUS_CONFIG = {
  PENDING: { label: '待审核', color: 'bg-yellow-100 text-yellow-800', icon: Clock },
  APPROVED: { label: '已通过', color: 'bg-green-100 text-green-800', icon: CheckCircle },
  REJECTED: { label: '已拒绝', color: 'bg-red-100 text-red-800', icon: XCircle },
}

const SUBJECT_TYPE_CONFIG = {
  SUPPLIER: { label: '供应商', color: 'bg-blue-100 text-blue-800' },
  BUYER: { label: '买家', color: 'bg-green-100 text-green-800' },
  PLATFORM: { label: '平台', color: 'bg-purple-100 text-purple-800' },
}

const RISK_LEVEL_CONFIG = {
  LOW: { label: '低风险', color: 'bg-green-100 text-green-800' },
  MEDIUM: { label: '中风险', color: 'bg-yellow-100 text-yellow-800' },
  HIGH: { label: '高风险', color: 'bg-red-100 text-red-800' },
}

export default function AdminSubjectsPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedStatus, setSelectedStatus] = useState<string>('all')
  const [selectedType, setSelectedType] = useState<string>('all')
  const [selectedSubject, setSelectedSubject] = useState<Subject | null>(null)
  const [showReviewModal, setShowReviewModal] = useState(false)
  const [reviewAction, setReviewAction] = useState<'approve' | 'reject'>('approve')
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredSubjects = MOCK_SUBJECTS.filter((subject) => {
    const matchesKeyword = 
      subject.name.toLowerCase().includes(searchKeyword.toLowerCase()) ||
      subject.creditCode.toLowerCase().includes(searchKeyword.toLowerCase())
    const matchesStatus = selectedStatus === 'all' || subject.status === selectedStatus
    const matchesType = selectedType === 'all' || subject.type === selectedType
    return matchesKeyword && matchesStatus && matchesType
  })

  const handleReview = (action: 'approve' | 'reject') => {
    setReviewAction(action)
    setShowReviewModal(true)
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="数据交易平台"
        roleName="平台管理员"
        tenantId="tenant_platform_001"
        scope="admin:subjects:write"
        sessionExpiresAt={sessionExpiresAt}
        userName="管理员"
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">主体审核</h1>
          <p className="text-gray-600">审核和管理平台主体资质</p>
        </div>

        {/* 筛选和搜索 */}
        <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
          <div className="flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
              <input
                type="text"
                value={searchKeyword}
                onChange={(e) => setSearchKeyword(e.target.value)}
                placeholder="搜索主体名称或统一社会信用代码..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>

            <select
              value={selectedStatus}
              onChange={(e) => setSelectedStatus(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部状态</option>
              <option value="PENDING">待审核</option>
              <option value="APPROVED">已通过</option>
              <option value="REJECTED">已拒绝</option>
            </select>

            <select
              value={selectedType}
              onChange={(e) => setSelectedType(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部类型</option>
              <option value="SUPPLIER">供应商</option>
              <option value="BUYER">买家</option>
            </select>

            <button className="flex items-center gap-2 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
              <Filter className="w-4 h-4" />
              <span>更多筛选</span>
            </button>
          </div>
        </div>

        {/* 主体列表 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 左侧列表 */}
          <div className="lg:col-span-2 space-y-4">
            {filteredSubjects.map((subject) => {
              const statusConfig = STATUS_CONFIG[subject.status]
              const StatusIcon = statusConfig.icon
              const isSelected = selectedSubject?.id === subject.id

              return (
                <div
                  key={subject.id}
                  onClick={() => setSelectedSubject(subject)}
                  className={`bg-white rounded-xl border-2 p-6 cursor-pointer transition-all ${
                    isSelected
                      ? 'border-primary-500 shadow-lg'
                      : 'border-gray-200 hover:border-primary-300 hover:shadow-md'
                  }`}
                >
                  {/* 头部 */}
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex-1">
                      <h3 className="text-lg font-bold text-gray-900 mb-2">{subject.name}</h3>
                      <div className="flex items-center gap-2 mb-2">
                        <span className={`status-tag text-xs ${SUBJECT_TYPE_CONFIG[subject.type].color}`}>
                          {SUBJECT_TYPE_CONFIG[subject.type].label}
                        </span>
                        <span className={`status-tag text-xs ${RISK_LEVEL_CONFIG[subject.riskLevel].color}`}>
                          {RISK_LEVEL_CONFIG[subject.riskLevel].label}
                        </span>
                      </div>
                    </div>
                    <span className={`status-tag ${statusConfig.color}`}>
                      <StatusIcon className="w-3.5 h-3.5" />
                      <span>{statusConfig.label}</span>
                    </span>
                  </div>

                  {/* 基本信息 */}
                  <div className="grid grid-cols-2 gap-4 mb-4 pb-4 border-b border-gray-100">
                    <div>
                      <div className="text-xs text-gray-500 mb-1">统一社会信用代码</div>
                      <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block truncate">
                        {subject.creditCode}
                      </code>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">法定代表人</div>
                      <div className="text-sm font-medium text-gray-900">{subject.legalPerson}</div>
                    </div>
                  </div>

                  {/* 底部信息 */}
                  <div className="flex items-center justify-between text-xs">
                    <div className="flex items-center gap-4 text-gray-500">
                      <div className="flex items-center gap-1">
                        <Calendar className="w-3 h-3" />
                        <span>提交: {subject.submittedAt.split(' ')[0]}</span>
                      </div>
                      {subject.reviewedAt && (
                        <div className="flex items-center gap-1">
                          <CheckCircle className="w-3 h-3" />
                          <span>审核: {subject.reviewedAt.split(' ')[0]}</span>
                        </div>
                      )}
                    </div>
                    {subject.status === 'PENDING' && (
                      <div className="flex gap-2">
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            handleReview('approve')
                          }}
                          className="px-3 py-1 bg-green-600 text-white rounded text-xs font-medium hover:bg-green-700"
                        >
                          通过
                        </button>
                        <button
                          onClick={(e) => {
                            e.stopPropagation()
                            handleReview('reject')
                          }}
                          className="px-3 py-1 bg-red-600 text-white rounded text-xs font-medium hover:bg-red-700"
                        >
                          拒绝
                        </button>
                      </div>
                    )}
                  </div>
                </div>
              )
            })}
          </div>

          {/* 右侧详情 */}
          <div className="lg:col-span-1">
            {selectedSubject ? (
              <div className="bg-white rounded-xl border border-gray-200 p-6 sticky top-28">
                <h3 className="text-lg font-bold text-gray-900 mb-6">主体详情</h3>

                <div className="space-y-6">
                  {/* Subject ID */}
                  <div>
                    <div className="text-xs text-gray-500 mb-1">Subject ID</div>
                    <code className="font-mono text-xs text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                      {selectedSubject.subjectId}
                    </code>
                  </div>

                  {/* 基本信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">基本信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">主体名称:</span>
                        <span className="font-medium text-gray-900">{selectedSubject.name}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">主体类型:</span>
                        <span className={`status-tag text-xs ${SUBJECT_TYPE_CONFIG[selectedSubject.type].color}`}>
                          {SUBJECT_TYPE_CONFIG[selectedSubject.type].label}
                        </span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">风险等级:</span>
                        <span className={`status-tag text-xs ${RISK_LEVEL_CONFIG[selectedSubject.riskLevel].color}`}>
                          {RISK_LEVEL_CONFIG[selectedSubject.riskLevel].label}
                        </span>
                      </div>
                    </div>
                  </div>

                  {/* 企业信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">企业信息</div>
                    <div className="space-y-3">
                      <div>
                        <div className="text-xs text-gray-600 mb-1">统一社会信用代码</div>
                        <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                          {selectedSubject.creditCode}
                        </code>
                      </div>
                      <div>
                        <div className="text-xs text-gray-600 mb-1">法定代表人</div>
                        <p className="text-sm text-gray-900">{selectedSubject.legalPerson}</p>
                      </div>
                      <div>
                        <div className="text-xs text-gray-600 mb-1">注册资本</div>
                        <p className="text-sm text-gray-900">{selectedSubject.registeredCapital}</p>
                      </div>
                      <div>
                        <div className="text-xs text-gray-600 mb-1 flex items-center gap-1">
                          <MapPin className="w-3 h-3" />
                          <span>注册地址</span>
                        </div>
                        <p className="text-sm text-gray-900">{selectedSubject.registeredAddress}</p>
                      </div>
                    </div>
                  </div>

                  {/* 联系信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">联系信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex items-center gap-2">
                        <Users className="w-4 h-4 text-gray-400" />
                        <span className="text-gray-600">联系人:</span>
                        <span className="font-medium text-gray-900">{selectedSubject.contactPerson}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Phone className="w-4 h-4 text-gray-400" />
                        <span className="text-gray-600">电话:</span>
                        <span className="font-medium text-gray-900">{selectedSubject.contactPhone}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <Mail className="w-4 h-4 text-gray-400" />
                        <span className="text-gray-600">邮箱:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedSubject.contactEmail}</span>
                      </div>
                    </div>
                  </div>

                  {/* 资质文件 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">资质文件</div>
                    <div className="p-3 bg-gray-50 border border-gray-200 rounded-lg">
                      <div className="flex items-center gap-2">
                        <FileText className="w-4 h-4 text-gray-600" />
                        <span className="text-sm text-gray-900">{selectedSubject.businessLicense}</span>
                      </div>
                      <button className="mt-2 text-xs text-primary-600 hover:text-primary-700 font-medium">
                        查看文件 →
                      </button>
                    </div>
                  </div>

                  {/* 审核信息 */}
                  {selectedSubject.reviewedAt && (
                    <div>
                      <div className="text-xs text-gray-500 mb-2">审核信息</div>
                      <div className="space-y-2 text-sm">
                        <div className="flex justify-between">
                          <span className="text-gray-600">审核人:</span>
                          <span className="font-medium text-gray-900">{selectedSubject.reviewer}</span>
                        </div>
                        <div className="flex justify-between">
                          <span className="text-gray-600">审核时间:</span>
                          <span className="font-medium text-gray-900 text-xs">{selectedSubject.reviewedAt}</span>
                        </div>
                        {selectedSubject.reviewNotes && (
                          <div>
                            <div className="text-xs text-gray-600 mb-1">审核备注</div>
                            <p className={`text-sm p-2 rounded ${
                              selectedSubject.status === 'APPROVED'
                                ? 'bg-green-50 text-green-800'
                                : 'bg-red-50 text-red-800'
                            }`}>
                              {selectedSubject.reviewNotes}
                            </p>
                          </div>
                        )}
                      </div>
                    </div>
                  )}

                  {/* 链上凭证 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">链上凭证</div>
                    <div className={`flex items-center gap-2 p-3 rounded-lg border ${
                      selectedSubject.chainStatus === 'CONFIRMED'
                        ? 'bg-green-50 border-green-200'
                        : 'bg-yellow-50 border-yellow-200'
                    }`}>
                      <Shield className={`w-4 h-4 ${
                        selectedSubject.chainStatus === 'CONFIRMED'
                          ? 'text-green-600'
                          : 'text-yellow-600'
                      }`} />
                      <span className={`text-xs font-medium ${
                        selectedSubject.chainStatus === 'CONFIRMED'
                          ? 'text-green-800'
                          : 'text-yellow-800'
                      }`}>
                        {selectedSubject.chainStatus === 'CONFIRMED' ? '已链上存证' : '已提交待确认'}
                      </span>
                    </div>
                  </div>

                  {/* 操作按钮 */}
                  {selectedSubject.status === 'PENDING' && (
                    <div className="space-y-2 pt-4 border-t border-gray-200">
                      <button
                        onClick={() => handleReview('approve')}
                        className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-green-600 text-white rounded-lg hover:bg-green-700 font-medium"
                      >
                        <CheckCircle className="w-4 h-4" />
                        <span>审核通过</span>
                      </button>
                      <button
                        onClick={() => handleReview('reject')}
                        className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-red-600 text-white rounded-lg hover:bg-red-700 font-medium"
                      >
                        <XCircle className="w-4 h-4" />
                        <span>审核拒绝</span>
                      </button>
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <div className="bg-white rounded-xl border border-gray-200 p-12 text-center sticky top-28">
                <Eye className="w-12 h-12 mx-auto text-gray-300 mb-4" />
                <p className="text-gray-600">选择一个主体查看详情</p>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* 审核 Modal */}
      {showReviewModal && selectedSubject && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-white rounded-xl p-8 max-w-2xl w-full mx-4 animate-fade-in">
            <h2 className="text-2xl font-bold text-gray-900 mb-6">
              {reviewAction === 'approve' ? '审核通过' : '审核拒绝'}
            </h2>

            <div className="mb-6">
              <div className="p-4 bg-gray-50 rounded-lg mb-4">
                <div className="text-sm text-gray-600 mb-1">主体名称</div>
                <div className="font-medium text-gray-900">{selectedSubject.name}</div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  审核备注 <span className="text-red-500">*</span>
                </label>
                <textarea
                  placeholder={reviewAction === 'approve' ? '请填写审核通过的备注...' : '请填写拒绝原因...'}
                  className="input min-h-[120px]"
                />
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={() => setShowReviewModal(false)}
                className="flex-1 px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium"
              >
                取消
              </button>
              <button
                onClick={() => {
                  setShowReviewModal(false)
                  // TODO: 提交审核
                }}
                className={`flex-1 px-6 py-3 text-white rounded-lg font-medium ${
                  reviewAction === 'approve'
                    ? 'bg-green-600 hover:bg-green-700'
                    : 'bg-red-600 hover:bg-red-700'
                }`}
              >
                确认{reviewAction === 'approve' ? '通过' : '拒绝'}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  )
}
