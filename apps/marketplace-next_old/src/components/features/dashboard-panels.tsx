import { Card } from "@/components/ui/card";

export function MetricCards({
  items,
}: {
  items: Array<{ label: string; value: string; trend?: string }>;
}) {
  return (
    <div className="grid grid-cols-5 gap-3">
      {items.map((item) => (
        <Card key={item.label} className="p-3">
          <p className="text-xs text-slate-500">{item.label}</p>
          <p className="mt-1 text-xl font-semibold text-slate-900">{item.value}</p>
          {item.trend ? <p className="mt-1 text-xs text-emerald-600">{item.trend}</p> : null}
        </Card>
      ))}
    </div>
  );
}
