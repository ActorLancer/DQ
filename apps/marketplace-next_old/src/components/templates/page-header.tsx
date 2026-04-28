import { ReactNode } from "react";

type Crumb = {
  label: string;
  current?: boolean;
};

export function PageHeader({
  crumbs,
  title,
  description,
  actions,
  metrics,
}: {
  crumbs: Crumb[];
  title: string;
  description?: string;
  actions?: ReactNode;
  metrics?: Array<{ label: string; value: string }>;
}) {
  return (
    <header className="rounded-lg panel p-3">
      <div className="mb-1 flex items-center gap-1 text-xs text-slate-500">
        {crumbs.map((crumb, index) => (
          <span key={crumb.label} className={crumb.current ? "font-medium text-slate-700" : ""}>
            {index > 0 ? " / " : ""}
            {crumb.label}
          </span>
        ))}
      </div>
      <div className="flex items-start justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold text-slate-900">{title}</h1>
          {description ? <p className="mt-1 text-sm text-slate-600">{description}</p> : null}
        </div>
        {actions ? <div className="shrink-0">{actions}</div> : null}
      </div>
      {metrics?.length ? (
        <div className="mt-3 grid grid-cols-5 gap-2">
          {metrics.map((item) => (
            <div key={item.label} className="rounded-lg panel-muted px-2 py-1.5">
              <p className="text-[11px] text-slate-500">{item.label}</p>
              <p className="text-sm font-semibold text-slate-900">{item.value}</p>
            </div>
          ))}
        </div>
      ) : null}
    </header>
  );
}
