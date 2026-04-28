export function LineageGraph() {
  return (
    <div className="rounded-xl border border-slate-200 bg-white p-3">
      <p className="mb-2 text-xs font-medium uppercase tracking-[0.1em] text-slate-500">
        数据血缘 / 来源图谱
      </p>
      <svg viewBox="0 0 720 220" className="h-56 w-full rounded-lg border border-slate-100 bg-slate-50">
        <defs>
          <marker id="arrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto">
            <path d="M0,0 L8,4 L0,8 z" fill="#64748b" />
          </marker>
        </defs>
        <g fontSize="11" fill="#334155" stroke="#cbd5e1">
          <rect x="40" y="80" width="140" height="50" rx="10" fill="#fff" />
          <text x="55" y="108">Raw Ingest Bucket</text>
          <rect x="280" y="30" width="150" height="50" rx="10" fill="#fff" />
          <text x="300" y="58">PII Masking Job</text>
          <rect x="280" y="140" width="150" height="50" rx="10" fill="#fff" />
          <text x="300" y="168">Quality Validation</text>
          <rect x="520" y="80" width="160" height="50" rx="10" fill="#fff" />
          <text x="540" y="108">Marketplace Product</text>
        </g>
        <path d="M180 105 L280 55" stroke="#64748b" strokeWidth="2" fill="none" markerEnd="url(#arrow)" />
        <path d="M180 105 L280 165" stroke="#64748b" strokeWidth="2" fill="none" markerEnd="url(#arrow)" />
        <path d="M430 55 L520 105" stroke="#64748b" strokeWidth="2" fill="none" markerEnd="url(#arrow)" />
        <path d="M430 165 L520 105" stroke="#64748b" strokeWidth="2" fill="none" markerEnd="url(#arrow)" />
      </svg>
    </div>
  );
}
