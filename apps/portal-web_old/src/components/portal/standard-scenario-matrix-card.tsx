"use client";

import type { ReactNode } from "react";

import { Boxes, CheckCircle2, Waypoints } from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import type {
  StandardScenarioCoverage,
  StandardScenarioSkuMapping,
} from "@/lib/standard-scenario-view";
import { cn } from "@/lib/utils";

type StandardScenarioMatrixCardProps<T extends StandardScenarioSkuMapping> = {
  title: string;
  description: string;
  items: readonly T[];
  summaryBadges?: readonly string[];
  note?: ReactNode;
  emptyTitle?: string;
  emptyDescription?: string;
  coverageForItem?: (item: T) => StandardScenarioCoverage | null;
  renderMeta?: (item: T) => ReactNode;
  renderAction?: (item: T) => ReactNode;
};

export function StandardScenarioMatrixCard<T extends StandardScenarioSkuMapping>({
  title,
  description,
  items,
  summaryBadges = [],
  note,
  emptyTitle = "暂无标准链路映射",
  emptyDescription = "当前页面没有可展示的标准链路与 SKU 映射。",
  coverageForItem,
  renderMeta,
  renderAction,
}: StandardScenarioMatrixCardProps<T>) {
  return (
    <Card className="space-y-4">
      <div className="flex items-start gap-3">
        <div className="rounded-2xl bg-[var(--accent-soft)] p-3 text-[var(--accent-strong)]">
          <Waypoints className="size-5" />
        </div>
        <div>
          <CardTitle>{title}</CardTitle>
          <CardDescription className="mt-1">{description}</CardDescription>
        </div>
      </div>
      {summaryBadges.length ? (
        <div className="flex flex-wrap gap-2">
          {summaryBadges.map((badge) => (
            <Badge
              key={badge}
              className="bg-white/75 text-[var(--ink-strong)]"
            >
              {badge}
            </Badge>
          ))}
        </div>
      ) : null}
      {note ? (
        <div className="rounded-[24px] bg-black/[0.035] px-4 py-3 text-sm text-[var(--ink-soft)]">
          {note}
        </div>
      ) : null}
      {items.length ? (
        <div className="grid gap-3 lg:grid-cols-2 2xl:grid-cols-5">
          {items.map((item) => {
            const coverage = coverageForItem?.(item) ?? null;

            return (
              <div
                key={item.scenario_code}
                className="rounded-[24px] border border-black/8 bg-black/[0.03] p-4"
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <Badge>{item.scenario_code}</Badge>
                  {coverage ? (
                    <CoverageBadge coverage={coverage} />
                  ) : null}
                </div>
                <div className="mt-3 text-sm font-semibold text-[var(--ink-strong)]">
                  {item.scenario_name}
                </div>
                <div className="mt-3 text-[11px] uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
                  场景名 -&gt; 主 SKU / 补充 SKU
                </div>
                <div className="mt-2 flex flex-wrap gap-2">
                  <ScenarioSkuBadge
                    sku={item.primary_sku}
                    role="主 SKU"
                    active={coverage?.primary_matched ?? false}
                  />
                  {item.supplementary_skus.map((sku) => (
                    <ScenarioSkuBadge
                      key={sku}
                      sku={sku}
                      role="补充 SKU"
                      active={coverage?.supplementary_matched.includes(sku) ?? false}
                    />
                  ))}
                </div>
                {coverage ? (
                  <div className="mt-3 grid gap-2">
                    <CoverageRow
                      label="当前商品命中"
                      value={coverage.matched_skus.join(" / ")}
                    />
                    <CoverageRow
                      label="待补充 SKU"
                      value={coverage.missing_skus.length ? coverage.missing_skus.join(" / ") : "无"}
                      muted={!coverage.missing_skus.length}
                    />
                  </div>
                ) : null}
                {renderMeta ? (
                  <div className="mt-3">{renderMeta(item)}</div>
                ) : null}
                {renderAction ? (
                  <div className="mt-4">{renderAction(item)}</div>
                ) : null}
              </div>
            );
          })}
        </div>
      ) : (
        <div className="flex min-h-48 flex-col items-center justify-center gap-3 rounded-[24px] bg-[var(--panel-muted)] p-6 text-center text-[var(--ink-soft)]">
          <Boxes className="size-8" />
          <CardTitle>{emptyTitle}</CardTitle>
          <CardDescription>{emptyDescription}</CardDescription>
        </div>
      )}
    </Card>
  );
}

export function ScenarioSkuBadge({
  sku,
  role,
  active = false,
}: {
  sku: string;
  role: string;
  active?: boolean;
}) {
  return (
    <span
      className={cn(
        "rounded-full px-2.5 py-1 text-[11px] font-semibold",
        active
          ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]"
          : "bg-white/75 text-[var(--ink-soft)]",
      )}
    >
      {role} {sku}
    </span>
  );
}

function CoverageBadge({ coverage }: { coverage: StandardScenarioCoverage }) {
  const content =
    coverage.match_level === "primary_with_supplementary"
      ? "主/补充 SKU 均命中"
      : coverage.match_level === "primary_only"
        ? "命中主 SKU"
        : "仅命中补充 SKU";

  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-[11px] font-semibold",
        coverage.match_level === "supplementary_only"
          ? "bg-[var(--warning-soft)] text-[var(--warning-ink)]"
          : "bg-emerald-50 text-emerald-700",
      )}
    >
      <CheckCircle2 className="size-3.5" />
      {content}
    </span>
  );
}

function CoverageRow({
  label,
  value,
  muted = false,
}: {
  label: string;
  value: string;
  muted?: boolean;
}) {
  return (
    <div className="rounded-2xl bg-white/75 px-3 py-3">
      <div className="text-[11px] uppercase tracking-[0.16em] text-[var(--ink-subtle)]">
        {label}
      </div>
      <div
        className={cn(
          "mt-1 text-sm font-semibold text-[var(--ink-strong)]",
          muted && "text-[var(--ink-soft)]",
        )}
      >
        {value}
      </div>
    </div>
  );
}
