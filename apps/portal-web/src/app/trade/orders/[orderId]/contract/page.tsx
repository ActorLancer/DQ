import { PortalRoutePage } from "@/components/portal/route-page";

export default async function ContractConfirmPage({
  params,
}: {
  params: Promise<{ orderId: string }>;
}) {
  return (
    <PortalRoutePage
      routeKey="order_contract_confirm"
      params={await params}
    />
  );
}
