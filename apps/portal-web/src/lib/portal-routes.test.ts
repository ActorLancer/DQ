import { describe, expect, it } from "vitest";

import { portalRouteMap } from "./portal-routes";

describe("portal route registry", () => {
  it("keeps official V1 frozen paths for the first key routes", () => {
    expect(portalRouteMap.portal_home.path).toBe("/");
    expect(portalRouteMap.catalog_search.path).toBe("/search");
    expect(portalRouteMap.product_detail.path).toBe("/products/:productId");
    expect(portalRouteMap.order_create.path).toBe("/trade/orders/new");
    expect(portalRouteMap.delivery_acceptance.path).toBe(
      "/delivery/orders/:orderId/acceptance",
    );
  });

  it("keeps view permissions and primary action permissions explicit", () => {
    expect(portalRouteMap.catalog_search.viewPermission).toBe(
      "portal.search.read",
    );
    expect(portalRouteMap.portal_home.apiBindings).toEqual(
      expect.arrayContaining([
        "/api/v1/catalog/standard-scenarios",
        "/api/v1/recommendations",
        "/api/v1/catalog/search",
      ]),
    );
    expect(portalRouteMap.seller_profile.apiBindings).toEqual(
      expect.arrayContaining([
        "/api/v1/sellers/{orgId}/profile",
        "/api/v1/recommendations?placement_code=seller_profile_featured",
      ]),
    );
    expect(portalRouteMap.seller_product_center.apiBindings).toEqual(
      expect.arrayContaining([
        "GET /api/v1/products",
        "POST /api/v1/products",
      ]),
    );
    expect(portalRouteMap.seller_sku_config.apiBindings).toEqual(
      expect.arrayContaining([
        "POST /api/v1/products/{id}/skus",
        "PATCH /api/v1/skus/{id}",
      ]),
    );
    expect(portalRouteMap.order_create.primaryPermissions).toContain(
      "trade.order.create",
    );
    expect(portalRouteMap.order_create.apiBindings).toEqual(
      expect.arrayContaining([
        "/api/v1/orders/standard-templates",
        "POST /api/v1/orders",
      ]),
    );
    expect(portalRouteMap.order_detail.apiBindings).toEqual(
      expect.arrayContaining([
        "/api/v1/orders/{id}/lifecycle-snapshots",
        "POST /api/v1/orders/{id}/cancel",
      ]),
    );
  });
});
