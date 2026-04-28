import { ProductSampleRow } from "@/lib/types";

export function SamplePreview({ rows }: { rows: ProductSampleRow[] }) {
  return (
    <div className="relative rounded-xl border border-slate-200">
      <div className="absolute inset-0 pointer-events-none bg-[linear-gradient(135deg,transparent_0%,transparent_45%,rgba(15,23,42,0.04)_50%,transparent_55%,transparent_100%)]" />
      <div className="flex items-center justify-between border-b border-slate-200 bg-slate-50 px-3 py-2 text-xs text-slate-600">
        <span>样例预览（前 100 行，脱敏）</span>
        <span>下载需授权 · 水印: tenant_developer@buyer-a</span>
      </div>
      <div className="max-h-72 overflow-auto">
        <table className="w-full text-xs">
          <thead className="sticky top-0 bg-white">
            <tr className="border-b border-slate-200 text-slate-500">
              <th className="px-3 py-2 text-left font-medium">#</th>
              <th className="px-3 py-2 text-left font-medium">merchant_id</th>
              <th className="px-3 py-2 text-left font-medium">txn_amount</th>
              <th className="px-3 py-2 text-left font-medium">txn_city</th>
              <th className="px-3 py-2 text-left font-medium">event_time</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.index} className="border-b border-slate-100 text-slate-700 last:border-none">
                <td className="px-3 py-1.5">{row.index}</td>
                <td className="px-3 py-1.5">{row.merchant_id}</td>
                <td className="px-3 py-1.5">{row.txn_amount}</td>
                <td className="px-3 py-1.5">{row.txn_city}</td>
                <td className="px-3 py-1.5">{row.event_time}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
