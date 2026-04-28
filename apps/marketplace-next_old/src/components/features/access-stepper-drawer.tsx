"use client";

import * as Dialog from "@radix-ui/react-dialog";
import { Check, ChevronRight } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";

const steps = ["选择套餐", "填写用途", "合规确认", "提交审批"] as const;

export function AccessStepperDrawer() {
  const [open, setOpen] = useState(false);
  const [current, setCurrent] = useState(0);

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger asChild>
        <Button className="w-full">申请访问</Button>
      </Dialog.Trigger>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-slate-900/45 backdrop-blur-sm" />
        <Dialog.Content className="fixed right-0 top-0 h-screen w-[520px] border-l border-slate-200 bg-white p-5 shadow-2xl">
          <Dialog.Title className="text-base font-semibold text-slate-900">申请访问流程</Dialog.Title>
          <div className="mt-4 flex items-center gap-2">
            {steps.map((step, idx) => (
              <div key={step} className="flex items-center gap-2">
                <div
                  className={`flex size-6 items-center justify-center rounded-full text-xs ${
                    idx <= current ? "bg-slate-900 text-white" : "bg-slate-100 text-slate-500"
                  }`}
                >
                  {idx < current ? <Check className="size-3" /> : idx + 1}
                </div>
                <span className={`text-xs ${idx <= current ? "text-slate-900" : "text-slate-500"}`}>
                  {step}
                </span>
              </div>
            ))}
          </div>

          <Card className="mt-4 p-4">
            {current === 0 ? (
              <div className="space-y-2">
                <p className="text-sm font-medium text-slate-900">选择套餐</p>
                <button className="w-full rounded-lg border border-slate-200 p-2 text-left text-sm hover:border-slate-300">
                  标准订阅 · 每日更新 · API + 样例访问
                </button>
                <button className="w-full rounded-lg border border-slate-200 p-2 text-left text-sm hover:border-slate-300">
                  企业增强 · 实时更新 · 专属 SLA
                </button>
              </div>
            ) : null}
            {current === 1 ? (
              <div className="space-y-2">
                <p className="text-sm font-medium text-slate-900">填写用途</p>
                <Input placeholder="项目名称" />
                <textarea
                  className="min-h-28 w-full rounded-xl border border-slate-200 p-3 text-sm outline-none focus:border-slate-300 focus:ring-2 focus:ring-slate-900/10"
                  placeholder="请说明使用场景、数据范围、保留周期..."
                />
              </div>
            ) : null}
            {current === 2 ? (
              <div className="space-y-2 text-sm text-slate-700">
                <p className="font-medium text-slate-900">合规确认</p>
                <label className="flex items-start gap-2">
                  <input type="checkbox" className="mt-1" />
                  我方已完成最小权限评估，并同意接入审计追踪与水印策略
                </label>
                <label className="flex items-start gap-2">
                  <input type="checkbox" className="mt-1" />
                  涉及 PII 字段的访问需审批通过后自动解锁
                </label>
              </div>
            ) : null}
            {current === 3 ? (
              <div className="space-y-2 text-sm text-slate-700">
                <p className="font-medium text-slate-900">提交成功</p>
                <p>审批流已创建，预计 2 小时内完成初审。</p>
                <div className="rounded-lg border border-slate-200 bg-slate-50 p-2 text-xs text-slate-600">
                  request_id: req-8adf219 · audit_action: market.access.request
                </div>
              </div>
            ) : null}
          </Card>

          <div className="mt-4 flex justify-between">
            <Button
              variant="secondary"
              onClick={() => setCurrent((value) => Math.max(0, value - 1))}
              disabled={current === 0}
            >
              上一步
            </Button>
            <Button
              onClick={() => setCurrent((value) => Math.min(steps.length - 1, value + 1))}
              disabled={current === steps.length - 1}
            >
              下一步 <ChevronRight className="ml-1 size-4" />
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
