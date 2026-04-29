import Header from '@/components/layout/Header'
import Footer from '@/components/layout/Footer'
import GlobalSearchBar from '@/components/home/GlobalSearchBar'
import IndustryCategoryGrid from '@/components/home/IndustryCategoryGrid'
import ProductCard from '@/components/home/ProductCard'
import SupplierCard from '@/components/home/SupplierCard'
import TrustCapabilityCards from '@/components/home/TrustCapabilityCards'
import StandardFlowEntrance from '@/components/home/StandardFlowEntrance'
import RoleConsoleEntrances from '@/components/home/RoleConsoleEntrances'
import type { Listing, Supplier } from '@/types'

// Mock 数据 - 推荐商品
const MOCK_PRODUCTS: Listing[] = [
  {
    id: 'listing_001',
    title: '企业工商风险数据',
    summary: '覆盖全国 5000 万+企业的工商信息、司法风险、经营异常等多维度数据',
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
      },
    ],
    qualityScore: 9.2,
    complianceTags: ['数据安全认证', '隐私保护', '合规审查'],
    chainRegistered: true,
    status: 'LISTED',
    trialSupported: true,
    createdAt: '2026-04-01T00:00:00Z',
    updatedAt: '2026-04-20T00:00:00Z',
  },
  {
    id: 'listing_002',
    title: '消费者行为分析数据',
    summary: '基于 1 亿+用户的消费行为、偏好分析、购买预测等数据服务',
    supplierId: 'supplier_002',
    supplierName: '智慧消费研究院',
    industry: '消费',
    dataType: '用户行为',
    deliveryMethods: ['API'],
    licenseTypes: ['SUBSCRIPTION'],
    pricingPlans: [
      {
        id: 'plan_002',
        listingId: 'listing_002',
        name: '企业版',
        pricingModel: 'YEARLY',
        price: 99999,
        currency: 'CNY',
        quota: 100000,
        durationDays: 365,
        deliveryMethods: ['API'],
      },
    ],
    qualityScore: 8.8,
    complianceTags: ['脱敏处理', '合规认证'],
    chainRegistered: true,
    status: 'LISTED',
    trialSupported: false,
    createdAt: '2026-03-15T00:00:00Z',
    updatedAt: '2026-04-18T00:00:00Z',
  },
  {
    id: 'listing_003',
    title: '物流轨迹实时数据',
    summary: '覆盖全国主要物流公司的实时轨迹、配送状态、时效预测数据',
    supplierId: 'supplier_003',
    supplierName: '智运物流数据',
    industry: '交通',
    dataType: '物流轨迹',
    deliveryMethods: ['API'],
    licenseTypes: ['USAGE_BASED'],
    pricingPlans: [
      {
        id: 'plan_003',
        listingId: 'listing_003',
        name: '按量计费',
        pricingModel: 'USAGE_BASED',
        price: 0.5,
        currency: 'CNY',
        deliveryMethods: ['API'],
      },
    ],
    qualityScore: 9.5,
    complianceTags: ['实时更新', 'API 稳定'],
    chainRegistered: true,
    status: 'LISTED',
    trialSupported: true,
    createdAt: '2026-04-10T00:00:00Z',
    updatedAt: '2026-04-25T00:00:00Z',
  },
  {
    id: 'listing_004',
    title: '医疗健康知识图谱',
    summary: '包含疾病、药品、症状、治疗方案等医疗知识的结构化数据',
    supplierId: 'supplier_004',
    supplierName: '医疗大数据中心',
    industry: '医疗',
    dataType: '知识图谱',
    deliveryMethods: ['API', 'FILE'],
    licenseTypes: ['COMMERCIAL'],
    pricingPlans: [
      {
        id: 'plan_004',
        listingId: 'listing_004',
        name: '定制版',
        pricingModel: 'CUSTOM',
        currency: 'CNY',
        deliveryMethods: ['API', 'FILE'],
      },
    ],
    qualityScore: 9.0,
    complianceTags: ['医疗合规', '数据脱敏', '专业审核'],
    chainRegistered: true,
    status: 'LISTED',
    trialSupported: false,
    createdAt: '2026-03-20T00:00:00Z',
    updatedAt: '2026-04-22T00:00:00Z',
  },
  {
    id: 'listing_005',
    title: '金融市场行情数据',
    summary: '股票、期货、外汇等金融市场的实时行情、历史数据与分析指标',
    supplierId: 'supplier_005',
    supplierName: '金融数据服务',
    industry: '金融',
    dataType: '市场行情',
    deliveryMethods: ['API'],
    licenseTypes: ['SUBSCRIPTION'],
    pricingPlans: [
      {
        id: 'plan_005',
        listingId: 'listing_005',
        name: '专业版',
        pricingModel: 'MONTHLY',
        price: 19999,
        currency: 'CNY',
        quota: 50000,
        durationDays: 30,
        deliveryMethods: ['API'],
      },
    ],
    qualityScore: 9.7,
    complianceTags: ['实时数据', '高可用', '金融级安全'],
    chainRegistered: true,
    status: 'LISTED',
    trialSupported: true,
    createdAt: '2026-04-05T00:00:00Z',
    updatedAt: '2026-04-27T00:00:00Z',
  },
  {
    id: 'listing_006',
    title: '政务公开数据集',
    summary: '政府公开的政策文件、统计数据、公共服务信息等结构化数据',
    supplierId: 'supplier_006',
    supplierName: '政务数据开放平台',
    industry: '政务',
    dataType: '公开数据',
    deliveryMethods: ['FILE', 'API'],
    licenseTypes: ['TRIAL'],
    pricingPlans: [
      {
        id: 'plan_006',
        listingId: 'listing_006',
        name: '免费版',
        pricingModel: 'TRIAL',
        price: 0,
        currency: 'CNY',
        quota: 1000,
        durationDays: 90,
        deliveryMethods: ['API'],
      },
    ],
    qualityScore: 8.5,
    complianceTags: ['官方数据', '公开透明'],
    chainRegistered: false,
    status: 'LISTED',
    trialSupported: true,
    createdAt: '2026-03-10T00:00:00Z',
    updatedAt: '2026-04-15T00:00:00Z',
  },
]

// Mock 数据 - 优质供应商
const MOCK_SUPPLIERS: Supplier[] = [
  {
    id: 'supplier_001',
    name: '天眼数据科技',
    certificationStatus: 'APPROVED',
    subjectType: 'SUPPLIER',
    tier: '金牌数据商',
    totalTransactions: 1280,
    responseTime: '2小时',
    complianceCertifications: ['ISO27001', '数据安全认证', '隐私保护认证'],
  },
  {
    id: 'supplier_002',
    name: '智慧消费研究院',
    certificationStatus: 'APPROVED',
    subjectType: 'SUPPLIER',
    tier: '银牌数据商',
    totalTransactions: 856,
    responseTime: '4小时',
    complianceCertifications: ['数据安全认证', '合规审查'],
  },
  {
    id: 'supplier_003',
    name: '智运物流数据',
    certificationStatus: 'APPROVED',
    subjectType: 'SUPPLIER',
    tier: '金牌数据商',
    totalTransactions: 2150,
    responseTime: '1小时',
    complianceCertifications: ['ISO27001', 'API 稳定性认证'],
  },
  {
    id: 'supplier_004',
    name: '医疗大数据中心',
    certificationStatus: 'APPROVED',
    subjectType: 'SUPPLIER',
    tier: '金牌数据商',
    totalTransactions: 645,
    responseTime: '3小时',
    complianceCertifications: ['医疗合规认证', '数据脱敏认证', '专业审核'],
  },
]

export default function Home() {
  return (
    <div className="min-h-screen bg-gray-50">
      <Header />

      {/* Hero Section */}
      <section className="bg-gradient-to-br from-primary-900 via-primary-800 to-primary-900 text-white py-24">
        <div className="container-custom">
          <div className="max-w-4xl mx-auto text-center">
            <h1 className="text-5xl md:text-6xl font-bold mb-6 text-balance">
              可信数据商品交易平台
            </h1>
            <p className="text-xl md:text-2xl text-primary-100 mb-12 text-balance">
              安全 · 合规 · 可追溯 · 链上存证
            </p>
            <GlobalSearchBar />
            <RoleConsoleEntrances />
          </div>
        </div>
      </section>

      {/* 行业分类 */}
      <section className="py-16">
        <div className="container-custom">
          <h2 className="text-3xl font-bold text-gray-900 mb-8 text-center">
            行业分类
          </h2>
          <IndustryCategoryGrid />
        </div>
      </section>

      {/* 推荐商品 */}
      <section className="py-16 bg-white">
        <div className="container-custom">
          <div className="flex items-center justify-between mb-8">
            <h2 className="text-3xl font-bold text-gray-900">
              推荐数据商品
            </h2>
            <a href="/marketplace" className="text-primary-600 hover:text-primary-700 font-medium">
              查看全部 →
            </a>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {MOCK_PRODUCTS.map((product) => (
              <ProductCard key={product.id} product={product} />
            ))}
          </div>
        </div>
      </section>

      {/* 优质供应商 */}
      <section className="py-16">
        <div className="container-custom">
          <div className="flex items-center justify-between mb-8">
            <h2 className="text-3xl font-bold text-gray-900">
              优质供应商
            </h2>
            <a href="/suppliers" className="text-primary-600 hover:text-primary-700 font-medium">
              查看全部 →
            </a>
          </div>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {MOCK_SUPPLIERS.map((supplier) => (
              <SupplierCard key={supplier.id} supplier={supplier} />
            ))}
          </div>
        </div>
      </section>

      {/* 可信能力 */}
      <section className="py-16 bg-white">
        <div className="container-custom">
          <h2 className="text-3xl font-bold text-gray-900 mb-8 text-center">
            可信能力
          </h2>
          <TrustCapabilityCards />
        </div>
      </section>

      {/* 标准链路 */}
      <section className="py-16">
        <div className="container-custom">
          <h2 className="text-3xl font-bold text-gray-900 mb-8 text-center">
            标准链路
          </h2>
          <StandardFlowEntrance />
        </div>
      </section>

      <Footer />
    </div>
  )
}
