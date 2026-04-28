import { OpsRiskWorkbench } from "@/components/features/ops-risk-workbench";
import { PageHeader } from "@/components/templates/page-header";
import { WorkspaceGrid } from "@/components/templates/workspace-grid";
import { getOpsRiskEvents } from "@/lib/repository";

export default async function OpsPage() {
  const events = await getOpsRiskEvents();

  return (
    <div className="grid h-full grid-rows-[auto_1fr] gap-3 overflow-hidden">
      <PageHeader
        crumbs={[
          { label: "Operations" },
          { label: "Control Plane", current: true },
        ]}
        title="平台运营后台"
        description="审核、风控、审计联查统一工作台。"
        metrics={[
          { label: "数据商审核", value: "14" },
          { label: "产品审核", value: "37" },
          { label: "交易订单", value: "1,238" },
          { label: "纠纷/退款", value: "29" },
          { label: "风险事件", value: `${events.length}` },
        ]}
      />
      <WorkspaceGrid
        center={
          <div className="p-4">
            <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">首页推荐配置</p>
            <div className="mt-2 space-y-2 text-sm text-slate-700">
              <div className="rounded-lg border border-slate-200 p-3">Top Banner: 金融风控专题周</div>
              <div className="rounded-lg border border-slate-200 p-3">热门试用: API 交付类产品</div>
              <div className="rounded-lg border border-slate-200 p-3">运营活动: 新供应商扶持计划</div>
            </div>
            <p className="mt-4 text-xs font-semibold uppercase tracking-[0.1em] text-slate-500" id="audit">
              合规与审计
            </p>
            <div className="mt-2 rounded-xl border border-slate-200 bg-slate-50 p-3 text-xs text-slate-600">
              高风险动作要求 step-up token；审计结果包含 request_id / tx_hash / chain_state / projection_state。
            </div>
          </div>
        }
        right={
          <OpsRiskWorkbench events={events} />
        }
      />
    </div>
  );
}
