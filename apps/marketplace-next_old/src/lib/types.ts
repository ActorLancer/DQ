export type Industry =
  | "finance"
  | "transport"
  | "healthcare"
  | "government"
  | "enterprise"
  | "ai_training";

export type DataType = "api" | "file" | "share" | "report" | "model";
export type DeliveryMode = "api" | "file_download" | "share_link";
export type PriceMode = "subscription" | "ppu" | "one_off";
export type Region = "cn_north" | "cn_east" | "global";

export interface DataProduct {
  id: string;
  name: string;
  supplier: string;
  industry: Industry[];
  dataType: DataType;
  delivery: DeliveryMode;
  priceMode: PriceMode;
  updateFrequency: "daily" | "weekly" | "monthly" | "realtime";
  coverage: string;
  region: Region;
  trial: boolean;
  tags: string[];
  description: string;
  priceCents: number;
  fieldCount: number;
  sampleRows: number;
  qualityScore: number;
  pii: "none" | "masked" | "approval_required";
}

export interface ProductField {
  name: string;
  type: string;
  description: string;
  sample: string;
  sensitive: boolean;
  quality: number;
}

export interface ProductSampleRow {
  index: number;
  merchant_id: string;
  txn_amount: string;
  txn_city: string;
  event_time: string;
}

export interface SupplierMetric {
  month: string;
  revenue: number;
  subscriptions: number;
  applications: number;
  calls: number;
  conversion: number;
}

export interface OpsRiskEvent {
  id: string;
  riskLevel: "low" | "medium" | "high";
  type: string;
  subject: string;
  requestId: string;
  txHash: string;
  chainStatus: "confirmed" | "pending";
  projectionStatus: "synced" | "lagging";
  at: string;
}
