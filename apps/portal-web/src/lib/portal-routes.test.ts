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
    expect(portalRouteMap.order_create.primaryPermissions).toContain(
      "trade.order.create",
    );
  });
});
