'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function SellerLatencyDistributionChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const chart = echarts.init(ref.current)

    const ranges = ['0-100ms', '100-200ms', '200-300ms', '300-500ms', '500-1000ms', '>1000ms']
    const counts = [7120, 2140, 520, 180, 60, 15]

    chart.setOption({
      tooltip: { trigger: 'axis', axisPointer: { type: 'shadow' } },
      grid: { left: '3%', right: '4%', top: '8%', bottom: '8%', containLabel: true },
      xAxis: { type: 'category', data: ranges },
      yAxis: { type: 'value', name: '调用次数' },
      series: [
        {
          type: 'bar',
          data: counts,
          barWidth: '55%',
          itemStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#8b5cf6' },
              { offset: 1, color: '#6366f1' },
            ]),
          },
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

  return <div ref={ref} className="w-full h-full" />
}
