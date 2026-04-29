'use client'

import { useState } from 'react'
import { useParams } from 'next/navigation'
import Header from '@/components/layout/Header'
import Footer from '@/components/layout/Footer'
import StickyTabs from '@/components/product/StickyTabs'
import RightStickyApplyPanel from '@/components/product/RightStickyApplyPanel'
import ChainProofCard from '@/components/product/ChainProofCard'
import AccessRequestDrawer from '@/components/product/AccessRequestDrawer'
import { Star, Shield, Clock, TrendingUp, Building2, CheckCircle } from 'lucide-react'
import type { Listing, ChainProof } from '@/types'

// Mock 数据
const MOCK_LISTING: Listing = {
  id: 'listing_001',
  title: '企业工商风险数据',
  summary: '覆盖全国 5000 万+企业的工商信息、司法风险、经营异常等多维度数据，实时更新，权威可信',
  supplierId: 'supplier_001',
  supplierName: '天眼数据科技',
  industry: '金融',
  dataType: '企业征信',
  deliveryMethods: ['API', 'FILE'],
  licenseTypes: ['COMMERCIAL', 'SUBSCRIPTION'],
  pricingPlans: [
    {
      id: 'plan_001',
      listingId: 'listing_001',
      name: '标准版',
      pricingModel: 'MONTHLY',
      price: 9999,
      currency: 'CNY',
      quota: 10000,
      durationDays: 30,
      deliveryMethods: ['API'],
      description: '适合中小企业日常风险查询',
    },
    {
      id: 'plan_002',
      listingId: 'listing_001',
      name: '企业版',
      pricingModel: 'YEARLY',
      price: 99999,
      currency: 'CNY',
      quota: 150000,
      durationDays: 365,
      deliveryMethods: ['API', 'FILE'],
      description: '适合大型企业批量查询',
    },
  ],
  qualityScore: 9.2,
  complianceTags: ['数据安全认证', '隐私保护', '合规审查'],
  chainRegistered: true,
  status: 'LISTED',
  coverageScope: '全国',
  updateFrequency: '实时更新',
  trialSupported: true,
  createdAt: '2026-04-01T00:00:00Z',
  updatedAt: '2026-04-20T00:00:00Z',
}

const MOCK_CHAIN_PROOF: ChainProof = {
  id: 'proof_001',
  requestId: 'req_20260420_listing_001',
  businessId: 'listing_001',
  businessType: 'LISTING',
  txHash: '0xabc123def456789012345678901234567890abcdef1234567890abcdef123456',
  chainStatus: 'CONFIRMED',
  projectionStatus: 'PROJECTED',
  blockHeight: 102938,
  contractName: 'DataListingRegistry',
  contractMethod: 'registerListing',
  submittedAt: '2026-04-20T10:30:00Z',
  confirmedAt: '2026-04-20T10:31:20Z',
  lastCheckedAt: '2026-04-28T15:32:10Z',
  createdAt: '2026-04-20T10:30:00Z',
  updatedAt: '2026-04-28T15:32:10Z',
}

const MOCK_SCHEMA_FIELDS = [
  {
    fieldName: 'company_name',
    fieldType: 'string',
    description: '企业名称',
    required: true,
    sensitive: false,
    sampleValue: '某某科技有限公司',
  },
  {
    fieldName: 'credit_code',
    fieldType: 'string',
    description: '统一社会信用代码',
    required: true,
    sensitive: true,
    maskingStrategy: '中间 8 位掩码',
    sampleValue: '91110000****0000XX',
  },
  {
    fieldName: 'legal_person',
    fieldType: 'string',
    description: '法定代表人',
    required: true,
    sensitive: true,
    maskingStrategy: '姓名脱敏',
    sampleValue: '张*',
  },
  {
    fieldName: 'risk_score',
    fieldType: 'number',
    description: '风险评分 (0-100)',
    required: true,
    sensitive: false,
    sampleValue: '75',
  },
  {
    fieldName: 'risk_level',
    fieldType: 'enum',
    description: '风险等级: LOW, MEDIUM, HIGH',
    required: true,
    sensitive: false,
    sampleValue: 'MEDIUM',
  },
]

export default function ProductDetailPage() {
  const params = useParams()
  const [activeTab, setActiveTab] = useState('overview')
  const [isDrawerOpen, setIsDrawerOpen] = useState(false)

  return (
    <div className="min-h-screen bg-gray-50">
      <Header />

      {/* Hero Section */}
      <section className="bg-primary-900 text-white py-12">
        <div className="container-custom">
          <div className="flex items-start gap-4 mb-4">
            <h1 className="text-4xl font-bold flex-1">{MOCK_LISTING.title}</h1>
            {MOCK_LISTING.chainRegistered && (
              <div className="chain-badge text-base px-3 py-1.5">
                <Shield className="w-4 h-4" />
                <span>链上存证</span>
              </div>
            )}
          </div>
          <p className="text-xl text-primary-100 mb-6">{MOCK_LISTING.summary}</p>

          {/* 元数据 */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
            <div>
              <div className="text-sm text-primary-300 mb-1">供应商</div>
              <div className="font-medium">{MOCK_LISTING.supplierName}</div>
            </div>
            <div>
              <div className="text-sm text-primary-300 mb-1">行业分类</div>
              <div className="font-medium">{MOCK_LISTING.industry}</div>
            </div>
            <div>
              <div className="text-sm text-primary-300 mb-1">更新频率</div>
              <div className="font-medium flex items-center gap-1">
                <Clock className="w-4 h-4" />
                {MOCK_LISTING.updateFrequency}
              </div>
            </div>
            <div>
              <div className="text-sm text-primary-300 mb-1">质量评分</div>
              <div className="font-medium flex items-center gap-1">
                <Star className="w-4 h-4 fill-yellow-400 text-yellow-400" />
                {MOCK_LISTING.qualityScore.toFixed(1)}
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Sticky Tabs */}
      <StickyTabs activeTab={activeTab} onTabChange={setActiveTab} />

      {/* 主体内容 */}
      <div className="container-custom py-8">
        <div className="flex gap-8">
          {/* 左侧内容 */}
          <main className="flex-1">
            {/* Overview */}
            {activeTab === 'overview' && (
              <div className="space-y-8">
                <section className="card">
                  <h2 className="text-2xl font-bold text-gray-900 mb-4">商品简介</h2>
                  <p className="text-gray-700 leading-relaxed mb-6">{MOCK_LISTING.summary}</p>
                  
                  <h3 className="text-lg font-bold text-gray-900 mb-3">适用场景</h3>
                  <ul className="list-disc list-inside space-y-2 text-gray-700 mb-6">
                    <li>企业尽职调查与风险评估</li>
                    <li>供应链合作伙伴筛选</li>
                    <li>信贷审批与风险控制</li>
                    <li>投资决策支持</li>
                  </ul>

                  <h3 className="text-lg font-bold text-gray-900 mb-3">数据来源说明</h3>
                  <p className="text-gray-700 leading-relaxed mb-6">
                    数据来源于国家企业信用信息公示系统、人民法院公告网、中国执行信息公开网等权威渠道，
                    经过多维度交叉验证和实时更新，确保数据的准确性和时效性。
                  </p>

                  <h3 className="text-lg font-bold text-gray-900 mb-3">覆盖范围</h3>
                  <div className="flex flex-wrap gap-2 mb-6">
                    <span className="tag">全国范围</span>
                    <span className="tag">5000 万+企业</span>
                    <span className="tag">实时更新</span>
                  </div>

                  <h3 className="text-lg font-bold text-gray-900 mb-3">合规说明</h3>
                  <div className="bg-green-50 border border-green-200 rounded-lg p-4">
                    <div className="flex flex-wrap gap-2">
                      {MOCK_LISTING.complianceTags.map((tag) => (
                        <span key={tag} className="inline-flex items-center gap-1 px-3 py-1 bg-green-100 text-green-800 rounded-full text-sm font-medium">
                          <CheckCircle className="w-4 h-4" />
                          {tag}
                        </span>
                      ))}
                    </div>
                  </div>
                </section>

                {/* 供应商信息 */}
                <section className="card">
                  <h2 className="text-2xl font-bold text-gray-900 mb-4">供应商信息</h2>
                  <div className="flex items-start gap-4">
                    <div className="w-16 h-16 bg-gradient-to-br from-primary-500 to-primary-700 rounded-xl flex items-center justify-center">
                      <Building2 className="w-8 h-8 text-white" />
                    </div>
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-2">
                        <h3 className="text-xl font-bold text-gray-900">{MOCK_LISTING.supplierName}</h3>
                        <CheckCircle className="w-5 h-5 text-success-600" />
                        <span className="px-2 py-0.5 bg-yellow-100 text-yellow-800 text-xs font-medium rounded">
                          金牌数据商
                        </span>
                      </div>
                      <div className="grid grid-cols-3 gap-4 mt-4">
                        <div>
                          <div className="text-sm text-gray-600 mb-1">历史成交</div>
                          <div className="text-lg font-bold text-gray-900">1,280</div>
                        </div>
                        <div>
                          <div className="text-sm text-gray-600 mb-1">响应时效</div>
                          <div className="text-lg font-bold text-gray-900">2 小时</div>
                        </div>
                        <div>
                          <div className="text-sm text-gray-600 mb-1">合规认证</div>
                          <div className="text-lg font-bold text-gray-900">3 项</div>
                        </div>
                      </div>
                    </div>
                  </div>
                </section>

                {/* 链上信息 */}
                <ChainProofCard chainProof={MOCK_CHAIN_PROOF} />
              </div>
            )}

            {/* Schema */}
            {activeTab === 'schema' && (
              <div className="card">
                <h2 className="text-2xl font-bold text-gray-900 mb-6">数据字段</h2>
                <div className="overflow-x-auto">
                  <table className="w-full">
                    <thead>
                      <tr className="border-b-2 border-gray-200">
                        <th className="text-left py-3 px-4 font-medium text-gray-700">字段名</th>
                        <th className="text-left py-3 px-4 font-medium text-gray-700">类型</th>
                        <th className="text-left py-3 px-4 font-medium text-gray-700">说明</th>
                        <th className="text-center py-3 px-4 font-medium text-gray-700">必填</th>
                        <th className="text-center py-3 px-4 font-medium text-gray-700">敏感</th>
                        <th className="text-left py-3 px-4 font-medium text-gray-700">脱敏方式</th>
                        <th className="text-left py-3 px-4 font-medium text-gray-700">示例值</th>
                      </tr>
                    </thead>
                    <tbody>
                      {MOCK_SCHEMA_FIELDS.map((field, index) => (
                        <tr key={index} className="border-b border-gray-100 hover:bg-gray-50">
                          <td className="py-3 px-4">
                            <code className="font-hash text-sm text-primary-600">{field.fieldName}</code>
                          </td>
                          <td className="py-3 px-4">
                            <span className="tag">{field.fieldType}</span>
                          </td>
                          <td className="py-3 px-4 text-sm text-gray-700">{field.description}</td>
                          <td className="py-3 px-4 text-center">
                            {field.required ? (
                              <span className="text-red-600">✓</span>
                            ) : (
                              <span className="text-gray-400">-</span>
                            )}
                          </td>
                          <td className="py-3 px-4 text-center">
                            {field.sensitive ? (
                              <span className="text-yellow-600">✓</span>
                            ) : (
                              <span className="text-gray-400">-</span>
                            )}
                          </td>
                          <td className="py-3 px-4 text-sm text-gray-700">
                            {field.maskingStrategy || '-'}
                          </td>
                          <td className="py-3 px-4">
                            <code className="font-hash text-xs text-gray-600">{field.sampleValue}</code>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {/* Sample */}
            {activeTab === 'sample' && (
              <div className="card">
                <h2 className="text-2xl font-bold text-gray-900 mb-4">数据样例</h2>
                <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
                  <p className="text-sm text-yellow-800">
                    以下为脱敏后的样例数据，仅供参考。真实数据需申请访问后获取。
                  </p>
                </div>
                <div className="bg-gray-900 rounded-lg p-6 overflow-x-auto">
                  <pre className="font-hash text-sm text-green-400">
{`{
  "company_name": "某某科技有限公司",
  "credit_code": "91110000****0000XX",
  "legal_person": "张*",
  "registered_capital": "1000万元",
  "establishment_date": "2020-01-15",
  "business_status": "存续",
  "risk_score": 75,
  "risk_level": "MEDIUM",
  "risk_items": [
    {
      "type": "LEGAL_CASE",
      "count": 2,
      "severity": "LOW"
    }
  ],
  "update_time": "2026-04-28T10:00:00Z"
}`}
                  </pre>
                </div>
                <div className="mt-6">
                  <button className="btn-primary">
                    申请完整样例数据
                  </button>
                </div>
              </div>
            )}

            {/* Pricing */}
            {activeTab === 'pricing' && (
              <div className="space-y-6">
                {MOCK_LISTING.pricingPlans.map((plan) => (
                  <div key={plan.id} className="card">
                    <div className="flex items-start justify-between mb-4">
                      <div>
                        <h3 className="text-2xl font-bold text-gray-900 mb-2">{plan.name}</h3>
                        <p className="text-gray-600">{plan.description}</p>
                      </div>
                      <div className="text-right">
                        <div className="text-3xl font-bold text-primary-600">
                          {plan.price ? `¥${plan.price.toLocaleString()}` : '面议'}
                        </div>
                        {plan.pricingModel === 'MONTHLY' && (
                          <div className="text-sm text-gray-600 mt-1">/ 月</div>
                        )}
                        {plan.pricingModel === 'YEARLY' && (
                          <div className="text-sm text-gray-600 mt-1">/ 年</div>
                        )}
                      </div>
                    </div>
                    <div className="grid grid-cols-3 gap-4 pt-4 border-t border-gray-200">
                      <div>
                        <div className="text-sm text-gray-600 mb-1">调用额度</div>
                        <div className="font-bold text-gray-900">
                          {plan.quota?.toLocaleString()} 次
                        </div>
                      </div>
                      <div>
                        <div className="text-sm text-gray-600 mb-1">授权期限</div>
                        <div className="font-bold text-gray-900">{plan.durationDays} 天</div>
                      </div>
                      <div>
                        <div className="text-sm text-gray-600 mb-1">交付方式</div>
                        <div className="font-bold text-gray-900">
                          {plan.deliveryMethods.join(' / ')}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* Docs */}
            {activeTab === 'docs' && (
              <div className="card">
                <h2 className="text-2xl font-bold text-gray-900 mb-6">API 文档</h2>
                <div className="space-y-6">
                  <section>
                    <h3 className="text-lg font-bold text-gray-900 mb-3">接入说明</h3>
                    <p className="text-gray-700 mb-4">
                      本 API 采用 RESTful 风格，支持 HTTPS 协议。所有请求需要在 Header 中携带 API Key。
                    </p>
                    <div className="bg-gray-900 rounded-lg p-4">
                      <code className="font-hash text-sm text-green-400">
                        Authorization: Bearer YOUR_API_KEY
                      </code>
                    </div>
                  </section>

                  <section>
                    <h3 className="text-lg font-bold text-gray-900 mb-3">请求示例</h3>
                    <div className="bg-gray-900 rounded-lg p-4 overflow-x-auto">
                      <pre className="font-hash text-sm text-green-400">
{`curl -X GET "https://api.example.com/v1/company/risk" \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "company_name": "某某科技有限公司"
  }'`}
                      </pre>
                    </div>
                  </section>

                  <section>
                    <h3 className="text-lg font-bold text-gray-900 mb-3">错误码</h3>
                    <table className="w-full">
                      <thead>
                        <tr className="border-b-2 border-gray-200">
                          <th className="text-left py-2 px-4 font-medium text-gray-700">错误码</th>
                          <th className="text-left py-2 px-4 font-medium text-gray-700">说明</th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr className="border-b border-gray-100">
                          <td className="py-2 px-4"><code className="font-hash text-sm">401</code></td>
                          <td className="py-2 px-4 text-sm text-gray-700">API Key 无效或已过期</td>
                        </tr>
                        <tr className="border-b border-gray-100">
                          <td className="py-2 px-4"><code className="font-hash text-sm">429</code></td>
                          <td className="py-2 px-4 text-sm text-gray-700">调用频率超限</td>
                        </tr>
                        <tr className="border-b border-gray-100">
                          <td className="py-2 px-4"><code className="font-hash text-sm">500</code></td>
                          <td className="py-2 px-4 text-sm text-gray-700">服务器内部错误</td>
                        </tr>
                      </tbody>
                    </table>
                  </section>
                </div>
              </div>
            )}

            {/* Reviews */}
            {activeTab === 'reviews' && (
              <div className="card">
                <h2 className="text-2xl font-bold text-gray-900 mb-6">用户评价</h2>
                <div className="text-center py-12 text-gray-500">
                  暂无评价，成为第一个评价的用户
                </div>
              </div>
            )}
          </main>

          {/* 右侧 Sticky 面板 */}
          <aside className="w-96 flex-shrink-0">
            <RightStickyApplyPanel listing={MOCK_LISTING} onApply={() => setIsDrawerOpen(true)} />
          </aside>
        </div>
      </div>

      <Footer />

      {/* 申请访问 Drawer */}
      <AccessRequestDrawer
        listing={MOCK_LISTING}
        isOpen={isDrawerOpen}
        onClose={() => setIsDrawerOpen(false)}
      />
    </div>
  )
}
