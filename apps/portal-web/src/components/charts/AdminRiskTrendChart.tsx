'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function AdminRiskTrendChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const chart = echarts.init(ref.current)

    const days = ['04-24', '04-25', '04-26', '04-27', '04-28', '04-29', '04-30']
    const high = [4, 5, 6, 7, 5, 4, 3]
    const medium = [9, 11, 10, 13, 12, 10, 9]
    const low = [16, 18, 17, 15, 18, 19, 20]

    chart.setOption({
      tooltip: { trigger: 'axis' },
      legend: { bottom: 0 },
      grid: { left: '3%', right: '4%', top: '6%', bottom: '14%', containLabel: true },
      xAxis: { type: 'category', data: days },
      yAxis: { type: 'value', name: '事件数' },
      series: [
        { name: '高风险', type: 'bar', stack: 'risk', data: high, color: '#ef4444' },
        { name: '中风险', type: 'bar', stack: 'risk', data: medium, color: '#f59e0b' },
        { name: '低风险', type: 'bar', stack: 'risk', data: low, color: '#10b981' },
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
