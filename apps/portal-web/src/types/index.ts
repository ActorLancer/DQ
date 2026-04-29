// 主体类型
export type SubjectType = 'BUYER' | 'SUPPLIER' | 'PLATFORM' | 'SERVICE_PROVIDER';

export type CertificationStatus = 'PENDING' | 'APPROVED' | 'REJECTED';

export type RiskLevel = 'LOW' | 'MEDIUM' | 'HIGH';

export interface Subject {
  id: string;
  name: string;
  type: SubjectType;
  certificationStatus: CertificationStatus;
  tenantId: string;
  riskLevel: RiskLevel;
}

// 用户类型
export interface User {
  id: string;
  name: string;
  mobile?: string;
  email?: string;
  subjects: Subject[];
  currentSubjectId: string;
  roles: string[];
}

// 商品状态
export type ListingStatus = 
  | 'DRAFT' 
  | 'PENDING_REVIEW' 
  | 'APPROVED' 
  | 'REJECTED' 
  | 'LISTED' 
  | 'SUSPENDED' 
  | 'DELISTED';

// 交付方式
export type DeliveryMethod = 'API' | 'FILE' | 'SANDBOX' | 'PRIVACY_COMPUTING' | 'TRUSTED_SPACE';

// 授权类型
export type LicenseType = 'TRIAL' | 'COMMERCIAL' | 'SUBSCRIPTION' | 'ONE_TIME' | 'USAGE_BASED';

// 定价模型
export type PricingModel = 'TRIAL' | 'ONE_TIME' | 'USAGE_BASED' | 'MONTHLY' | 'YEARLY' | 'CUSTOM';

// 链状态
export type ChainStatus = 
  | 'NOT_SUBMITTED' 
  | 'PENDING_SUBMIT' 
  | 'SUBMITTING' 
  | 'SUBMITTED' 
  | 'CONFIRMED' 
  | 'FAILED' 
  | 'RETRYING' 
  | 'UNKNOWN';

// 投影状态
export type ProjectionStatus = 
  | 'PENDING' 
  | 'PROJECTED' 
  | 'OUT_OF_SYNC' 
  | 'REBUILDING' 
  | 'FAILED';

// 定价计划
export interface PricingPlan {
  id: string;
  listingId: string;
  name: string;
  pricingModel: PricingModel;
  price?: number;
  currency: 'CNY';
  quota?: number;
  durationDays?: number;
  deliveryMethods: DeliveryMethod[];
  description?: string;
}

// 数据商品
export interface Listing {
  id: string;
  title: string;
  summary: string;
  supplierId: string;
  supplierName: string;
  industry: string;
  dataType: string;
  deliveryMethods: DeliveryMethod[];
  licenseTypes: LicenseType[];
  pricingPlans: PricingPlan[];
  qualityScore: number;
  complianceTags: string[];
  chainRegistered: boolean;
  status: ListingStatus;
  coverageScope?: string;
  updateFrequency?: string;
  trialSupported: boolean;
  createdAt: string;
  updatedAt: string;
}

// 供应商
export interface Supplier {
  id: string;
  name: string;
  certificationStatus: CertificationStatus;
  subjectType: SubjectType;
  tier?: string;
  totalTransactions?: number;
  responseTime?: string;
  complianceCertifications?: string[];
}

// 链上凭证
export interface ChainProof {
  id: string;
  requestId: string;
  businessId: string;
  businessType: 'LISTING' | 'ACCESS_REQUEST' | 'ORDER' | 'SUBSCRIPTION' | 'DELIVERY';
  txHash?: string;
  chainStatus: ChainStatus;
  projectionStatus: ProjectionStatus;
  blockHeight?: number;
  contractName?: string;
  contractMethod?: string;
  errorMessage?: string;
  submittedAt?: string;
  confirmedAt?: string;
  lastCheckedAt?: string;
  createdAt: string;
  updatedAt: string;
}

// 访问申请状态
export type AccessRequestStatus = 
  | 'DRAFT' 
  | 'SUBMITTED' 
  | 'PENDING_SUPPLIER_REVIEW' 
  | 'PENDING_PLATFORM_REVIEW' 
  | 'NEED_MORE_INFO' 
  | 'APPROVED' 
  | 'REJECTED' 
  | 'CANCELLED';

// 访问申请
export interface AccessRequest {
  id: string;
  requestId: string;
  idempotencyKey: string;
  listingId: string;
  buyerSubjectId: string;
  supplierSubjectId: string;
  planId: string;
  usagePurpose: string;
  expectedUsage: string;
  complianceConfirmed: boolean;
  status: AccessRequestStatus;
  workflowStatus: string;
  chainStatus: ChainStatus;
  projectionStatus: ProjectionStatus;
  createdAt: string;
  updatedAt: string;
}

// 市场筛选参数
export interface MarketplaceFilters {
  keyword?: string;
  industry?: string[];
  dataType?: string[];
  deliveryMethod?: DeliveryMethod[];
  licenseType?: LicenseType[];
  pricingModel?: PricingModel[];
  supplierType?: SubjectType[];
  qualityScoreMin?: number;
  trialSupported?: boolean;
  chainRegistered?: boolean;
  updatedAfter?: string;
  complianceTags?: string[];
  sort?: 'comprehensive' | 'latest' | 'quality_desc' | 'calls_desc' | 'transactions_desc' | 'price_asc' | 'price_desc';
  page?: number;
  pageSize?: number;
}

// 市场响应
export interface MarketplaceResponse {
  requestId: string;
  items: Listing[];
  facets: {
    industry: Array<{ value: string; count: number }>;
    deliveryMethod: Array<{ value: string; count: number }>;
    licenseType: Array<{ value: string; count: number }>;
    complianceTags: Array<{ value: string; count: number }>;
  };
  pagination: {
    page: number;
    pageSize: number;
    total: number;
  };
}
