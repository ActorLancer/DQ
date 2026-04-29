'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

const weekdays = ['周一', '周二', '周三', '周四', '周五', '周六', '周日']
const hours = Array.from({ length: 24 }, (_, i) => `${i}:00`)

const data = weekdays.flatMap((day, di) =>
  hours.map((h, hi) => {
    const base = di < 5 ? 120 : 65
    const peak = hi >= 10 && hi <= 17 ? 90 : hi >= 19 && hi <= 22 ? 40 : 0
    const noise = ((di + 2) * (hi + 3)) % 22
    return [hi, di, base + peak + noise]
  }),
)

export default function BuyerTrafficHeatmapChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const el = ref.current
    const chart = echarts.init(el)

    const option: echarts.EChartsOption = {
      tooltip: { position: 'top' },
      grid: { left: 48, right: 12, top: 24, bottom: 36 },
      xAxis: { type: 'category', data: hours, axisLabel: { color: '#64748b', fontSize: 11 }, axisTick: { show: false }, axisLine: { lineStyle: { color: '#e2e8f0' } } },
      yAxis: { type: 'category', data: weekdays, axisLabel: { color: '#64748b', fontSize: 11 }, axisTick: { show: false }, axisLine: { lineStyle: { color: '#e2e8f0' } } },
      visualMap: {
        min: 40,
        max: 240,
        calculable: false,
        orient: 'horizontal',
        left: 'center',
        bottom: 0,
        textStyle: { color: '#64748b', fontSize: 11 },
        inRange: { color: ['#eef2ff', '#c7d2fe', '#818cf8', '#4338ca'] },
      },
      series: [
        {
          type: 'heatmap',
          data,
          label: { show: false },
          emphasis: { itemStyle: { shadowBlur: 10, shadowColor: 'rgba(0, 0, 0, 0.2)' } },
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
