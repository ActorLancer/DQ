import { describe, expect, it } from "vitest";

import { portalRouteMap } from "./portal-routes";
import { standardDemoGuides } from "./standard-demo";

describe("portal route registry", () => {
  it("keeps official V1 frozen paths for the first key routes", () => {
    expect(portalRouteMap.portal_home.path).toBe("/");
    expect(portalRouteMap.catalog_search.path).toBe("/search");
    expect(portalRouteMap.product_detail.path).toBe("/products/:productId");
    expect(portalRouteMap.order_create.path).toBe("/trade/orders/new");
    expect(portalRouteMap.delivery_acceptance.path).toBe(
      "/delivery/orders/:orderId/acceptance",
    );
    expect(standardDemoGuides.map((guide) => portalRouteMap[guide.routeKey].path))
      .toEqual(["/demos/S1", "/demos/S2", "/demos/S3", "/demos/S4", "/demos/S5"]);
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
    for (const guide of standardDemoGuides) {
      expect(portalRouteMap[guide.routeKey].viewPermission).toBe("portal.home.read");
      expect(portalRouteMap[guide.routeKey].apiBindings).toEqual(
        expect.arrayContaining([
          "GET /api/v1/catalog/standard-scenarios",
          "GET /api/v1/orders/standard-templates",
          "GET /api/v1/recommendations?placement_code=home_featured",
          "GET /api/v1/catalog/search",
        ]),
      );
    }
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
    expect(portalRouteMap.delivery_file.apiBindings).toEqual(
      expect.arrayContaining([
        "/api/v1/auth/me",
        "GET /api/v1/orders/{id}",
        "POST /api/v1/orders/{id}/deliver",
        "GET /api/v1/orders/{id}/download-ticket",
      ]),
    );
    expect(portalRouteMap.delivery_template_query.apiBindings).toEqual(
      expect.arrayContaining([
        "POST /api/v1/orders/{id}/template-grants",
        "GET /api/v1/orders/{id}/template-runs",
      ]),
    );
    expect(portalRouteMap.delivery_sandbox.apiBindings).toContain(
      "POST /api/v1/orders/{id}/sandbox-workspaces",
    );
    expect(portalRouteMap.delivery_acceptance.apiBindings).toEqual(
      expect.arrayContaining([
        "/api/v1/auth/me",
        "GET /api/v1/orders/{id}/lifecycle-snapshots",
        "POST /api/v1/orders/{id}/accept",
        "POST /api/v1/orders/{id}/reject",
      ]),
    );
    expect(portalRouteMap.billing_center.apiBindings).toEqual(
      expect.arrayContaining([
        "/api/v1/auth/me",
        "GET /api/v1/billing/{order_id}",
      ]),
    );
    expect(portalRouteMap.billing_refund_compensation.apiBindings).toEqual(
      expect.arrayContaining([
        "GET /api/v1/billing/{order_id}",
        "POST /api/v1/refunds",
        "POST /api/v1/compensations",
      ]),
    );
    expect(portalRouteMap.dispute_create.apiBindings).toEqual(
      expect.arrayContaining([
        "/api/v1/auth/me",
        "GET /api/v1/orders/{id}",
        "POST /api/v1/cases",
        "POST /api/v1/cases/{id}/evidence",
        "POST /api/v1/cases/{id}/resolve",
      ]),
    );
  });
});
