'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

const days = ['04/01', '04/05', '04/09', '04/13', '04/17', '04/21', '04/25', '04/29']

export default function BuyerErrorBreakdownChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const el = ref.current
    const chart = echarts.init(el)

    const option: echarts.EChartsOption = {
      tooltip: { trigger: 'axis' },
      legend: { top: 0, textStyle: { color: '#64748b', fontSize: 11 } },
      grid: { left: 34, right: 16, top: 34, bottom: 28 },
      xAxis: { type: 'category', data: days, axisLabel: { color: '#64748b', fontSize: 11 }, axisTick: { show: false }, axisLine: { lineStyle: { color: '#e2e8f0' } } },
      yAxis: { type: 'value', axisLabel: { color: '#64748b', fontSize: 11 }, splitLine: { lineStyle: { color: '#f1f5f9' } } },
      series: [
        { name: '4xx 参数类', type: 'bar', stack: 'err', data: [32, 36, 28, 44, 39, 26, 22, 30], itemStyle: { color: '#f59e0b', borderRadius: [4, 4, 0, 0] } },
        { name: '5xx 服务类', type: 'bar', stack: 'err', data: [14, 20, 16, 24, 18, 13, 12, 15], itemStyle: { color: '#ef4444', borderRadius: [4, 4, 0, 0] } },
        { name: '超时类', type: 'bar', stack: 'err', data: [7, 9, 8, 12, 10, 8, 6, 7], itemStyle: { color: '#6366f1', borderRadius: [4, 4, 0, 0] } },
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
