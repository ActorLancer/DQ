'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function BuyerLatencyPercentileChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const el = ref.current
    const chart = echarts.init(el)

    const option: echarts.EChartsOption = {
      tooltip: { trigger: 'axis' },
      grid: { left: 28, right: 18, top: 20, bottom: 24 },
      xAxis: {
        type: 'category',
        data: ['P50', 'P75', 'P90', 'P95', 'P99'],
        axisLabel: { color: '#64748b', fontSize: 11 },
        axisTick: { show: false },
        axisLine: { lineStyle: { color: '#e2e8f0' } },
      },
      yAxis: {
        type: 'value',
        axisLabel: { color: '#64748b', fontSize: 11, formatter: '{value}ms' },
        splitLine: { lineStyle: { color: '#f1f5f9' } },
      },
      series: [
        {
          type: 'bar',
          data: [122, 188, 266, 302, 458],
          barWidth: 18,
          itemStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#334155' },
              { offset: 1, color: '#94a3b8' },
            ]),
            borderRadius: [5, 5, 0, 0],
          },
          label: { show: true, position: 'top', color: '#334155', fontSize: 11 },
        },
      ],
    }

    chart.setOption(option, true)
    const resize = () => chart.resize()
    const ro = new ResizeObserver(() => resize())
    ro.observe(el)
    const t1 = window.setTimeout(resize, 60)
    const t2 = window.setTimeout(resize, 220)
    window.addEventListener('resize', resize)
    return () => {
      window.clearTimeout(t1)
      window.clearTimeout(t2)
      ro.disconnect()
      window.removeEventListener('resize', resize)
      chart.dispose()
    }
  }, [])

  return <div ref={ref} className="h-full w-full" />
}
