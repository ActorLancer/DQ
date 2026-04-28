import { PageHeader } from "@/components/templates/page-header";
import { WorkspaceGrid } from "@/components/templates/workspace-grid";
import { Card } from "@/components/ui/card";

export default function BuyerPage() {
  return (
    <div className="grid h-full grid-rows-[auto_1fr] gap-3 overflow-hidden">
      <PageHeader
        crumbs={[
          { label: "Buyer" },
          { label: "Workspace", current: true },
        ]}
        title="买家控制台"
        description="订阅、申请、凭证、交付、账单与使用量统一视图。"
        metrics={[
          { label: "我的订阅", value: "28" },
          { label: "我的申请", value: "12" },
          { label: "API Key", value: "7" },
          { label: "本月调用量", value: "312万" },
          { label: "本月账单", value: "$84,200" },
        ]}
      />

      <WorkspaceGrid
        center={
          <div className="p-4">
            <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">数据交付记录</p>
            <table className="mt-3 w-full text-sm">
              <thead className="sticky top-0 bg-white">
                <tr className="border-b border-slate-200 text-left text-xs text-slate-500">
                  <th className="pb-2">订单</th>
                  <th className="pb-2">产品</th>
                  <th className="pb-2">状态</th>
                  <th className="pb-2">交付</th>
                  <th className="pb-2">时间</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ["ORD-22081", "企业交易风险评分 API", "已验收", "API Token", "2026-04-23"],
                  ["ORD-22077", "物流流量月报", "待验收", "文件下载", "2026-04-22"],
                  ["ORD-22074", "门急诊结局共享数据集", "审批中", "共享链接", "2026-04-21"],
                ].map((item) => (
                  <tr key={item[0]} className="border-b border-slate-100 text-slate-700">
                    <td className="py-2">{item[0]}</td>
                    <td>{item[1]}</td>
                    <td>{item[2]}</td>
                    <td>{item[3]}</td>
                    <td>{item[4]}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        }
        right={
          <div className="flex h-full flex-col gap-3 p-3">
            <Card className="rounded-lg p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">权限与凭证</p>
              <div className="mt-2 space-y-2 text-sm text-slate-700">
                <div className="rounded-lg border border-slate-200 p-2">Scope: market.read / order.write</div>
                <div className="rounded-lg border border-slate-200 p-2">Step-up: required for replay/refund actions</div>
                <div className="rounded-lg border border-slate-200 p-2">API Key rotation every 30 days</div>
              </div>
            </Card>
            <Card className="flex-1 overflow-auto rounded-lg p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">收藏夹</p>
              <ul className="mt-2 space-y-2 text-sm text-slate-700">
                <li className="rounded-lg border border-slate-200 p-2">企业交易风险评分 API</li>
                <li className="rounded-lg border border-slate-200 p-2">企业经营景气度指数报告</li>
                <li className="rounded-lg border border-slate-200 p-2">跨区域物流拥堵预测模型</li>
              </ul>
            </Card>
          </div>
        }
      />
    </div>
  );
}
