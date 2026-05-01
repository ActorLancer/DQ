'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function AdminWorkflowFunnelChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const chart = echarts.init(ref.current)

    chart.setOption({
      tooltip: { trigger: 'item', formatter: '{b}: {c}' },
      series: [
        {
          type: 'funnel',
          left: '5%',
          top: 10,
          bottom: 20,
          width: '90%',
          min: 0,
          max: 120,
          minSize: '40%',
          maxSize: '100%',
          sort: 'descending',
          gap: 2,
          label: { show: true, position: 'inside', fontSize: 12 },
          itemStyle: { borderColor: '#fff', borderWidth: 1 },
          emphasis: { label: { fontSize: 13, fontWeight: 'bold' } },
          data: [
            { value: 112, name: '待审核提交' },
            { value: 88, name: '进入复核' },
            { value: 63, name: '完成审计' },
            { value: 49, name: '通过/生效' },
          ],
          color: ['#3b82f6', '#6366f1', '#8b5cf6', '#10b981'],
        },
      ],
    })

    const onResize = () => chart.resize()
    window.addEventListener('resize', onResize)
    return () => {
      window.removeEventListener('resize', onResize)
      chart.dispose()
    }
  }, [])

  return <div ref={ref} className="h-full w-full" />
}
