import { Card } from "@/components/ui/card";

export default function DocsPage() {
  return (
    <div className="grid h-full grid-cols-2 gap-3 overflow-hidden">
      <Card className="overflow-auto p-4">
        <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">API 文档</p>
        <ul className="mt-2 space-y-2 text-sm text-slate-700">
          <li className="rounded-lg border border-slate-200 p-2">catalog: 产品检索、详情、供应商主页</li>
          <li className="rounded-lg border border-slate-200 p-2">trade: 下单、订单详情、支付锁</li>
          <li className="rounded-lg border border-slate-200 p-2">delivery: 交付记录、验收、回滚链路</li>
          <li className="rounded-lg border border-slate-200 p-2">billing: 账单、发票、争议处理</li>
          <li className="rounded-lg border border-slate-200 p-2">audit/ops: 审计联查、风险事件、通知 replay</li>
        </ul>
      </Card>
      <Card className="overflow-auto p-4">
        <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">Service Route Map</p>
        <div className="mt-2 space-y-2 text-sm text-slate-700">
          <div className="rounded-lg border border-slate-200 p-3">
            前端路由: `/marketplace` → `/marketplace/:id` → `/buyer` → `/ops`
          </div>
          <div className="rounded-lg border border-slate-200 p-3">
            API 路由: `/api/platform/**` → platform-core → domain modules
          </div>
          <div className="rounded-lg border border-slate-200 p-3">
            区块链相关展示字段：`request_id`、`tx_hash`、`chain_status`、`projection_status`
          </div>
        </div>
      </Card>
    </div>
  );
}
