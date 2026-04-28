import { PageHeader } from "@/components/templates/page-header";
import { WorkspaceGrid } from "@/components/templates/workspace-grid";
import { Card } from "@/components/ui/card";
import { getSupplierMetrics } from "@/lib/repository";

export default async function SellerPage() {
  const metrics = await getSupplierMetrics();
  const latest = metrics[metrics.length - 1];

  return (
    <div className="grid h-full grid-rows-[auto_1fr] gap-3 overflow-hidden">
      <PageHeader
        crumbs={[
          { label: "Supplier" },
          { label: "Workbench", current: true },
        ]}
        title="供应商后台"
        description="管理 Listing、审批、订阅客户与收益。"
        metrics={[
          { label: "收入", value: `$${latest.revenue.toLocaleString()}` },
          { label: "订阅数", value: `${latest.subscriptions}` },
          { label: "申请数", value: `${latest.applications}` },
          { label: "调用量", value: `${(latest.calls / 10000).toFixed(1)}万` },
          { label: "转化率", value: `${(latest.conversion * 100).toFixed(1)}%` },
        ]}
      />

      <WorkspaceGrid
        center={
          <div className="p-4">
            <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">数据产品列表</p>
            <table className="mt-3 w-full text-sm">
              <thead className="sticky top-0 bg-white">
                <tr className="border-b border-slate-200 text-left text-xs text-slate-500">
                  <th className="pb-2">产品</th>
                  <th className="pb-2">状态</th>
                  <th className="pb-2">订阅</th>
                  <th className="pb-2">调用</th>
                  <th className="pb-2">收入</th>
                </tr>
              </thead>
              <tbody>
                {[
                  ["企业交易风险评分 API", "已上架", "129", "53.2万", "$210,000"],
                  ["门急诊就诊结局共享数据集", "审核中", "44", "18.9万", "$95,000"],
                  ["企业景气度指数报告", "已上架", "72", "-", "$62,000"],
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
              <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">创建 Listing 向导</p>
              <ol className="mt-2 space-y-1 text-sm text-slate-700">
                <li>1. 上传数据 / 配置 API / 配置共享</li>
                <li>2. 字段标注与合规标签</li>
                <li>3. 定价套餐与试用策略</li>
                <li>4. 发布审批与审计确认</li>
              </ol>
            </Card>
            <Card className="flex-1 overflow-auto rounded-lg p-4">
              <p className="text-xs font-semibold uppercase tracking-[0.1em] text-slate-500">审批与客户</p>
              <div className="mt-2 space-y-2 text-sm text-slate-700">
                <div className="rounded-lg border border-slate-200 p-3">
                  访问申请待审批: <span className="font-semibold">23</span>
                </div>
                <div className="rounded-lg border border-slate-200 p-3">
                  订阅客户: <span className="font-semibold">168</span>
                </div>
                <div className="rounded-lg border border-slate-200 p-3">
                  合同与发票处理中: <span className="font-semibold">12</span>
                </div>
              </div>
            </Card>
          </div>
        }
      />
    </div>
  );
}
