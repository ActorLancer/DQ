'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

const points = ['W1', 'W2', 'W3', 'W4', 'W5', 'W6']

export default function BuyerQuotaForecastChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const el = ref.current
    const chart = echarts.init(el)

    const option: echarts.EChartsOption = {
      tooltip: { trigger: 'axis' },
      grid: { left: 34, right: 18, top: 24, bottom: 30 },
      xAxis: { type: 'category', data: points, axisLabel: { color: '#64748b', fontSize: 11 }, axisLine: { lineStyle: { color: '#e2e8f0' } }, axisTick: { show: false } },
      yAxis: { type: 'value', axisLabel: { color: '#64748b', fontSize: 11, formatter: '{value}%' }, splitLine: { lineStyle: { color: '#f1f5f9' } } },
      series: [
        {
          name: '配额消耗',
          type: 'line',
          smooth: true,
          symbol: 'circle',
          symbolSize: 6,
          data: [28, 42, 56, 68, 79, 88],
          lineStyle: { width: 3, color: '#0f172a' },
          itemStyle: { color: '#0f172a' },
          areaStyle: { color: 'rgba(15, 23, 42, 0.08)' },
          markLine: {
            symbol: 'none',
            label: { formatter: '预警线 80%', color: '#b45309' },
            lineStyle: { color: '#f59e0b', type: 'dashed' },
            data: [{ yAxis: 80 }],
          },
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
