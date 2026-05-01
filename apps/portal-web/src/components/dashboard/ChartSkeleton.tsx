export default function ChartSkeleton({ heightClass = 'h-64' }: { heightClass?: string }) {
  return (
    <div className={`${heightClass} w-full rounded-xl border border-gray-200 bg-gradient-to-br from-gray-50 to-white p-4 animate-pulse`}>
      <div className="h-4 w-40 rounded bg-gray-200 mb-4" />
      <div className="h-[calc(100%-2rem)] rounded-lg bg-gray-100" />
    </div>
  )
}
