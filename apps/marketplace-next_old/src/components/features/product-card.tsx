"use client";

import { motion } from "motion/react";
import Link from "next/link";
import { useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { DataProduct } from "@/lib/types";
import { formatPrice } from "@/lib/utils";

export function ProductCard({ product }: { product: DataProduct }) {
  const [hovered, setHovered] = useState(false);

  return (
    <motion.div layout>
      <Card
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        className="group h-full p-4 transition hover:-translate-y-0.5 hover:shadow-md"
      >
        <div className="flex items-start justify-between gap-3">
          <div>
            <p className="text-base font-semibold text-slate-900">{product.name}</p>
            <p className="mt-1 text-xs text-slate-500">{product.supplier}</p>
          </div>
          <Badge>{product.dataType.toUpperCase()}</Badge>
        </div>

        <div className="mt-3 flex flex-wrap gap-1.5">
          {product.tags.slice(0, 4).map((tag) => (
            <Badge key={tag} className="bg-white">
              {tag}
            </Badge>
          ))}
        </div>

        <p className="mt-3 line-clamp-2 text-sm text-slate-600">{product.description}</p>
        <div className="mt-3 flex items-center justify-between text-xs text-slate-500">
          <span>{product.coverage}</span>
          <span>{product.updateFrequency}</span>
        </div>

        <motion.div
          animate={{
            height: hovered ? "auto" : 0,
            opacity: hovered ? 1 : 0,
          }}
          className="overflow-hidden"
        >
          <div className="mt-3 grid grid-cols-2 gap-2 rounded-xl border border-slate-200 bg-slate-50 p-2 text-xs text-slate-600">
            <div>字段数: {product.fieldCount}</div>
            <div>样例数: {product.sampleRows}</div>
            <div>质量分: {product.qualityScore}</div>
            <div>授权: {product.pii === "approval_required" ? "审批后访问" : "标准授权"}</div>
          </div>
        </motion.div>

        <div className="mt-4 flex items-center justify-between">
          <p className="text-sm font-semibold text-slate-900">
            {formatPrice(product.priceCents, product.priceMode)}
          </p>
          <Link
            href={`/marketplace/${product.id}`}
            className="rounded-lg border border-slate-200 px-3 py-1.5 text-xs font-medium text-slate-700 transition hover:border-slate-300 hover:bg-slate-50"
          >
            查看详情
          </Link>
        </div>
      </Card>
    </motion.div>
  );
}
