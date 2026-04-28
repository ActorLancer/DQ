import { isLiveDataEnabled } from "./env";
import { marketplaceProducts, opsRiskEvents, productFields, sampleRows, supplierMetrics } from "./mock-data";
import { DataProduct } from "./types";

export interface MarketplaceFilters {
  q?: string;
  industry?: string[];
  dataType?: string[];
  delivery?: string[];
  priceMode?: string[];
  update?: string[];
}

function withFilter(items: DataProduct[], filters: MarketplaceFilters) {
  return items.filter((item) => {
    const query = (filters.q ?? "").trim().toLowerCase();
    if (query) {
      const target = `${item.name} ${item.supplier} ${item.description}`.toLowerCase();
      if (!target.includes(query)) {
        return false;
      }
    }

    if (filters.industry?.length) {
      if (!filters.industry.some((industry) => item.industry.includes(industry as never))) {
        return false;
      }
    }

    if (filters.dataType?.length && !filters.dataType.includes(item.dataType)) {
      return false;
    }

    if (filters.delivery?.length && !filters.delivery.includes(item.delivery)) {
      return false;
    }

    if (filters.priceMode?.length && !filters.priceMode.includes(item.priceMode)) {
      return false;
    }

    if (filters.update?.length && !filters.update.includes(item.updateFrequency)) {
      return false;
    }

    return true;
  });
}

export async function listProducts(filters: MarketplaceFilters) {
  if (isLiveDataEnabled()) {
    try {
      const params = new URLSearchParams();
      if (filters.q) params.set("q", filters.q);
      const response = await fetch(`/api/platform/api/v1/catalog/products?${params.toString()}`, {
        cache: "no-store",
      });
      if (response.ok) {
        const payload = (await response.json()) as {
          data?: Array<{ product_id?: string; title?: string; provider_name?: string }>;
        };
        if (payload.data?.length) {
          return payload.data.map((item, idx) => ({
            ...marketplaceProducts[idx % marketplaceProducts.length],
            id: item.product_id ?? marketplaceProducts[idx % marketplaceProducts.length].id,
            name: item.title ?? marketplaceProducts[idx % marketplaceProducts.length].name,
            supplier:
              item.provider_name ?? marketplaceProducts[idx % marketplaceProducts.length].supplier,
          }));
        }
      }
    } catch {
      return withFilter(marketplaceProducts, filters);
    }
  }

  return withFilter(marketplaceProducts, filters);
}

export async function getProduct(productId: string) {
  const product =
    marketplaceProducts.find((item) => item.id === productId) ?? marketplaceProducts[0];
  return {
    product,
    fields: productFields,
    rows: sampleRows,
  };
}

export async function getSupplierMetrics() {
  return supplierMetrics;
}

export async function getOpsRiskEvents() {
  return opsRiskEvents;
}
