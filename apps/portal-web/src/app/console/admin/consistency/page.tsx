'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Search,
  Database,
  Link2,
  CheckCircle,
  XCircle,
  AlertTriangle,
  RefreshCw,
  Eye,
  Shield,
  Activity
} from 'lucide-react'

interface ConsistencyCheck {
  id: string
  requestId?: string
  txHash?: string
  businessId?: string
  businessType: 'LISTING' | 'ACCESS_REQUEST' | 'ORDER' | 'SUBSCRIPTION' | 'DELIVERY'
  chainStatus: 'NOT_SUBMITTED' | 'SUBMITTED' | 'CONFIRMED' | 'FAILED'
  projectionStatus: 'PENDING' | 'PROJECTED' | 'OUT_OF_SYNC' | 'FAILED'
  dbRecord: boolean
  chainRecord: boolean
  projectionRecord: boolean
  consistent: boolean
  inconsistencyType?: string
  createdAt: string
  lastCheckedAt: string
}

const MOCK_CHECKS: ConsistencyCheck[] = [
  {
    id: 'check_001',
    requestId: 'request_20260428_001',
    txHash: '0x1234567890abcdef1234567890abcdef12345678',
    businessId: 'listing_001',
    businessType: 'LISTING',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
    dbRecord: true,
    chainRecord: true,
    projectionRecord: true,
    consistent: true,
    createdAt: '2026-04-28 10:00:00',
    lastCheckedAt: '2026-04-28 15:30:00',
  },
  {
    id: 'check_002',
    requestId: 'request_20260428_002',
    txHash: '0xabcdef1234567890abcdef1234567890abcdef12',
    businessId: 'request_002',
    businessType: 'ACCESS_REQUEST',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'OUT_OF_SYNC',
    dbRecord: true,
    chainRecord: true,
    projectionRecord: false,
    consistent: false,
    inconsistencyType: 'PROJECTION_MISSING',
    createdAt: '2026-04-28 09:00:00',
    lastCheckedAt: '2026-04-28 15:25:00',
  },
  {
    id: 'check_003',
    requestId: 'request_20260427_003',
    businessId: 'order_003',
    businessType: 'ORDER',
    chainStatus: 'SUBMITTED',
    projectionStatus: 'PENDING',
    dbRecord: true,
    chainRecord: false,
    projectionRecord: false,
    consistent: false,
    inconsistencyType: 'CHAIN_NOT_CONFIRMED',
    createdAt: '2026-04-27 16:00:00',
    lastCheckedAt: '2026-04-28 15:20:00',
  },
  {
    id: 'check_004',
    requestId: 'request_20260427_004',
    txHash: '0x567890abcdef1234567890abcdef1234567890ab',
    businessId: 'subscription_004',
    businessType: 'SUBSCRIPTION',
    chainStatus: 'CONFIRMED',
    projectionStatus: 'PROJECTED',
    dbRecord: true,
    chainRecord: true,
    projectionRecord: true,
    consistent: true,
    createdAt: '2026-04-27 14:00:00',
    lastCheckedAt: '2026-04-28 15:15:00',
  },
  {
    id: 'check_005',
    requestId: 'request_20260427_005',
    txHash: '0x234567890abcdef1234567890abcdef1234567890',
    businessId: 'delivery_005',
    businessType: 'DELIVERY',
    chainStatus: 'FAILED',
    projectionStatus: 'FAILED',
    dbRecord: true,
    chainRecord: false,
    projectionRecord: false,
    consistent: false,
    inconsistencyType: 'CHAIN_SUBMISSION_FAILED',
    createdAt: '2026-04-27 11:00:00',
    lastCheckedAt: '2026-04-28 15:10:00',
  },
]

const BUSINESS_TYPE_CONFIG = {
  LISTING: { label: '商品', color: 'bg-blue-100 text-blue-800' },
  ACCESS_REQUEST: { label: '访问申请', color: 'bg-purple-100 text-purple-800' },
  ORDER: { label: '订单', color: 'bg-green-100 text-green-800' },
  SUBSCRIPTION: { label: '订阅', color: 'bg-yellow-100 text-yellow-800' },
  DELIVERY: { label: '交付', color: 'bg-pink-100 text-pink-800' },
}

const CHAIN_STATUS_CONFIG = {
  NOT_SUBMITTED: { label: '未提交', color: 'bg-gray-100 text-gray-800' },
  SUBMITTED: { label: '已提交', color: 'bg-yellow-100 text-yellow-800' },
  CONFIRMED: { label: '已确认', color: 'bg-green-100 text-green-800' },
  FAILED: { label: '失败', color: 'bg-red-100 text-red-800' },
}

const PROJECTION_STATUS_CONFIG = {
  PENDING: { label: '待投影', color: 'bg-gray-100 text-gray-800' },
  PROJECTED: { label: '已投影', color: 'bg-green-100 text-green-800' },
  OUT_OF_SYNC: { label: '不同步', color: 'bg-orange-100 text-orange-800' },
  FAILED: { label: '失败', color: 'bg-red-100 text-red-800' },
}

const INCONSISTENCY_TYPE_CONFIG: Record<string, { label: string; description: string }> = {
  PROJECTION_MISSING: {
    label: '投影缺失',
    description: '链上记录已确认，但投影数据库中缺少对应记录',
  },
  CHAIN_NOT_CONFIRMED: {
    label: '链未确认',
    description: '数据库有记录，但链上交易尚未确认',
  },
  CHAIN_SUBMISSION_FAILED: {
    label: '链提交失败',
    description: '向区块链提交交易失败',
  },
  DATA_MISMATCH: {
    label: '数据不一致',
    description: '数据库、链上、投影三者数据内容不一致',
  },
}

export default function AdminConsistencyPage() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const [selectedFilter, setSelectedFilter] = useState<string>('all')
  const [selectedCheck, setSelectedCheck] = useState<ConsistencyCheck | null>(null)
  const [isChecking, setIsChecking] = useState(false)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const filteredChecks = MOCK_CHECKS.filter((check) => {
    const matchesKeyword = 
      (check.requestId?.toLowerCase().includes(searchKeyword.toLowerCase())) ||
      (check.txHash?.toLowerCase().includes(searchKeyword.toLowerCase())) ||
      (check.businessId?.toLowerCase().includes(searchKeyword.toLowerCase()))
    
    const matchesFilter = 
      selectedFilter === 'all' ||
      (selectedFilter === 'inconsistent' && !check.consistent) ||
      (selectedFilter === 'consistent' && check.consistent)
    
    return matchesKeyword && matchesFilter
  })

  const handleRunCheck = () => {
    setIsChecking(true)
    setTimeout(() => {
      setIsChecking(false)
    }, 2000)
  }

  const stats = {
    total: MOCK_CHECKS.length,
    consistent: MOCK_CHECKS.filter(c => c.consistent).length,
    inconsistent: MOCK_CHECKS.filter(c => !c.consistent).length,
    consistencyRate: ((MOCK_CHECKS.filter(c => c.consistent).length / MOCK_CHECKS.length) * 100).toFixed(1),
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="数据交易平台"
        roleName="平台管理员"
        tenantId="tenant_platform_001"
        scope="admin:consistency:write"
        sessionExpiresAt={sessionExpiresAt}
        userName="管理员"
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">系统一致性检查</h1>
            <p className="text-gray-600">检查数据库、区块链、投影状态的一致性</p>
          </div>
          <button
            onClick={handleRunCheck}
            disabled={isChecking}
            className="flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <RefreshCw className={`w-5 h-5 ${isChecking ? 'animate-spin' : ''}`} />
            <span>{isChecking ? '检查中...' : '运行检查'}</span>
          </button>
        </div>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 mb-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-blue-50 rounded-lg flex items-center justify-center">
                <Database className="w-6 h-6 text-blue-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{stats.total}</div>
            <div className="text-sm text-gray-600">总检查记录</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-green-50 rounded-lg flex items-center justify-center">
                <CheckCircle className="w-6 h-6 text-green-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{stats.consistent}</div>
            <div className="text-sm text-gray-600">一致记录</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-red-50 rounded-lg flex items-center justify-center">
                <AlertTriangle className="w-6 h-6 text-red-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{stats.inconsistent}</div>
            <div className="text-sm text-gray-600">不一致记录</div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center justify-between mb-4">
              <div className="w-12 h-12 bg-purple-50 rounded-lg flex items-center justify-center">
                <Activity className="w-6 h-6 text-purple-600" />
              </div>
            </div>
            <div className="text-2xl font-bold text-gray-900 mb-1">{stats.consistencyRate}%</div>
            <div className="text-sm text-gray-600">一致性率</div>
          </div>
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
                placeholder="搜索 Request ID、TX Hash 或 Business ID..."
                className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
              />
            </div>

            <select
              value={selectedFilter}
              onChange={(e) => setSelectedFilter(e.target.value)}
              className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            >
              <option value="all">全部记录</option>
              <option value="consistent">仅一致</option>
              <option value="inconsistent">仅不一致</option>
            </select>
          </div>
        </div>

        {/* 检查结果列表 */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* 左侧列表 */}
          <div className="lg:col-span-2 space-y-4">
            {filteredChecks.map((check) => {
              const isSelected = selectedCheck?.id === check.id

              return (
                <div
                  key={check.id}
                  onClick={() => setSelectedCheck(check)}
                  className={`bg-white rounded-xl border-2 p-6 cursor-pointer transition-all ${
                    isSelected
                      ? 'border-primary-500 shadow-lg'
                      : 'border-gray-200 hover:border-primary-300 hover:shadow-md'
                  }`}
                >
                  {/* 头部 */}
                  <div className="flex items-start justify-between mb-4">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <span className={`status-tag text-xs ${BUSINESS_TYPE_CONFIG[check.businessType].color}`}>
                          {BUSINESS_TYPE_CONFIG[check.businessType].label}
                        </span>
                        {check.consistent ? (
                          <span className="status-tag text-xs bg-green-100 text-green-800">
                            <CheckCircle className="w-3 h-3" />
                            <span>一致</span>
                          </span>
                        ) : (
                          <span className="status-tag text-xs bg-red-100 text-red-800">
                            <AlertTriangle className="w-3 h-3" />
                            <span>不一致</span>
                          </span>
                        )}
                      </div>
                      {check.requestId && (
                        <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block mb-2">
                          {check.requestId}
                        </code>
                      )}
                    </div>
                  </div>

                  {/* 状态指示器 */}
                  <div className="grid grid-cols-3 gap-3 mb-4">
                    <div className={`p-3 rounded-lg border ${
                      check.dbRecord ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'
                    }`}>
                      <div className="flex items-center gap-2 mb-1">
                        <Database className={`w-4 h-4 ${check.dbRecord ? 'text-green-600' : 'text-red-600'}`} />
                        <span className="text-xs font-medium text-gray-900">数据库</span>
                      </div>
                      <div className={`text-xs ${check.dbRecord ? 'text-green-700' : 'text-red-700'}`}>
                        {check.dbRecord ? '有记录' : '无记录'}
                      </div>
                    </div>

                    <div className={`p-3 rounded-lg border ${
                      check.chainRecord ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'
                    }`}>
                      <div className="flex items-center gap-2 mb-1">
                        <Link2 className={`w-4 h-4 ${check.chainRecord ? 'text-green-600' : 'text-red-600'}`} />
                        <span className="text-xs font-medium text-gray-900">区块链</span>
                      </div>
                      <div className={`text-xs ${check.chainRecord ? 'text-green-700' : 'text-red-700'}`}>
                        {check.chainRecord ? '已确认' : '未确认'}
                      </div>
                    </div>

                    <div className={`p-3 rounded-lg border ${
                      check.projectionRecord ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'
                    }`}>
                      <div className="flex items-center gap-2 mb-1">
                        <Shield className={`w-4 h-4 ${check.projectionRecord ? 'text-green-600' : 'text-red-600'}`} />
                        <span className="text-xs font-medium text-gray-900">投影</span>
                      </div>
                      <div className={`text-xs ${check.projectionRecord ? 'text-green-700' : 'text-red-700'}`}>
                        {check.projectionRecord ? '已投影' : '未投影'}
                      </div>
                    </div>
                  </div>

                  {/* 不一致类型 */}
                  {!check.consistent && check.inconsistencyType && (
                    <div className="p-3 bg-red-50 border border-red-200 rounded-lg mb-4">
                      <div className="text-xs font-medium text-red-900 mb-1">
                        {INCONSISTENCY_TYPE_CONFIG[check.inconsistencyType]?.label || check.inconsistencyType}
                      </div>
                      <div className="text-xs text-red-700">
                        {INCONSISTENCY_TYPE_CONFIG[check.inconsistencyType]?.description || '数据不一致'}
                      </div>
                    </div>
                  )}

                  {/* 底部信息 */}
                  <div className="flex items-center justify-between text-xs text-gray-500">
                    <span>最后检查: {check.lastCheckedAt}</span>
                    {!check.consistent && (
                      <button className="text-primary-600 hover:text-primary-700 font-medium">
                        修复 →
                      </button>
                    )}
                  </div>
                </div>
              )
            })}
          </div>

          {/* 右侧详情 */}
          <div className="lg:col-span-1">
            {selectedCheck ? (
              <div className="bg-white rounded-xl border border-gray-200 p-6 sticky top-28">
                <h3 className="text-lg font-bold text-gray-900 mb-6">检查详情</h3>

                <div className="space-y-6">
                  {/* 业务信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">业务信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">业务类型:</span>
                        <span className={`status-tag text-xs ${BUSINESS_TYPE_CONFIG[selectedCheck.businessType].color}`}>
                          {BUSINESS_TYPE_CONFIG[selectedCheck.businessType].label}
                        </span>
                      </div>
                      {selectedCheck.businessId && (
                        <div>
                          <div className="text-xs text-gray-600 mb-1">Business ID</div>
                          <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                            {selectedCheck.businessId}
                          </code>
                        </div>
                      )}
                      {selectedCheck.requestId && (
                        <div>
                          <div className="text-xs text-gray-600 mb-1">Request ID</div>
                          <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                            {selectedCheck.requestId}
                          </code>
                        </div>
                      )}
                    </div>
                  </div>

                  {/* 链上信息 */}
                  {selectedCheck.txHash && (
                    <div>
                      <div className="text-xs text-gray-500 mb-2">链上信息</div>
                      <div className="space-y-2">
                        <div>
                          <div className="text-xs text-gray-600 mb-1">TX Hash</div>
                          <code className="text-xs font-mono text-gray-900 bg-gray-50 px-2 py-1 rounded block break-all">
                            {selectedCheck.txHash}
                          </code>
                        </div>
                        <div className="flex justify-between text-sm">
                          <span className="text-gray-600">链状态:</span>
                          <span className={`status-tag text-xs ${CHAIN_STATUS_CONFIG[selectedCheck.chainStatus].color}`}>
                            {CHAIN_STATUS_CONFIG[selectedCheck.chainStatus].label}
                          </span>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* 投影状态 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">投影状态</div>
                    <div className="flex justify-between text-sm">
                      <span className="text-gray-600">投影状态:</span>
                      <span className={`status-tag text-xs ${PROJECTION_STATUS_CONFIG[selectedCheck.projectionStatus].color}`}>
                        {PROJECTION_STATUS_CONFIG[selectedCheck.projectionStatus].label}
                      </span>
                    </div>
                  </div>

                  {/* 一致性状态 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">一致性状态</div>
                    <div className={`p-4 rounded-lg border ${
                      selectedCheck.consistent
                        ? 'bg-green-50 border-green-200'
                        : 'bg-red-50 border-red-200'
                    }`}>
                      <div className="flex items-center gap-2 mb-3">
                        {selectedCheck.consistent ? (
                          <>
                            <CheckCircle className="w-5 h-5 text-green-600" />
                            <span className="font-medium text-green-900">数据一致</span>
                          </>
                        ) : (
                          <>
                            <AlertTriangle className="w-5 h-5 text-red-600" />
                            <span className="font-medium text-red-900">数据不一致</span>
                          </>
                        )}
                      </div>
                      <div className="space-y-2 text-xs">
                        <div className="flex items-center justify-between">
                          <span className={selectedCheck.consistent ? 'text-green-700' : 'text-red-700'}>数据库记录:</span>
                          <span className="font-medium">{selectedCheck.dbRecord ? '✓' : '✗'}</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className={selectedCheck.consistent ? 'text-green-700' : 'text-red-700'}>链上记录:</span>
                          <span className="font-medium">{selectedCheck.chainRecord ? '✓' : '✗'}</span>
                        </div>
                        <div className="flex items-center justify-between">
                          <span className={selectedCheck.consistent ? 'text-green-700' : 'text-red-700'}>投影记录:</span>
                          <span className="font-medium">{selectedCheck.projectionRecord ? '✓' : '✗'}</span>
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* 不一致详情 */}
                  {!selectedCheck.consistent && selectedCheck.inconsistencyType && (
                    <div>
                      <div className="text-xs text-gray-500 mb-2">不一致详情</div>
                      <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
                        <div className="text-sm font-medium text-red-900 mb-2">
                          {INCONSISTENCY_TYPE_CONFIG[selectedCheck.inconsistencyType]?.label || selectedCheck.inconsistencyType}
                        </div>
                        <div className="text-xs text-red-700">
                          {INCONSISTENCY_TYPE_CONFIG[selectedCheck.inconsistencyType]?.description || '数据不一致'}
                        </div>
                      </div>
                    </div>
                  )}

                  {/* 时间信息 */}
                  <div>
                    <div className="text-xs text-gray-500 mb-2">时间信息</div>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-gray-600">创建时间:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedCheck.createdAt}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-gray-600">最后检查:</span>
                        <span className="font-medium text-gray-900 text-xs">{selectedCheck.lastCheckedAt}</span>
                      </div>
                    </div>
                  </div>

                  {/* 操作按钮 */}
                  <div className="space-y-2 pt-4 border-t border-gray-200">
                    <button className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                      <RefreshCw className="w-4 h-4" />
                      <span>重新检查</span>
                    </button>
                    {!selectedCheck.consistent && (
                      <button className="w-full flex items-center justify-center gap-2 px-4 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                        <Shield className="w-4 h-4" />
                        <span>尝试修复</span>
                      </button>
                    )}
                  </div>
                </div>
              </div>
            ) : (
              <div className="bg-white rounded-xl border border-gray-200 p-12 text-center sticky top-28">
                <Eye className="w-12 h-12 mx-auto text-gray-300 mb-4" />
                <p className="text-gray-600">选择一条记录查看详情</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </>
  )
}
