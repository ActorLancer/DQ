'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function SellerRevenueTrendChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const chart = echarts.init(ref.current)

    const dates = Array.from({ length: 14 }).map((_, i) => `04-${(i + 16).toString().padStart(2, '0')}`)
    const sub = [12, 9, 14, 10, 16, 18, 15, 19, 21, 17, 24, 20, 26, 22]
    const renew = [6, 5, 7, 8, 7, 9, 8, 10, 12, 11, 10, 13, 12, 14]
    const upgrade = [2, 1, 3, 2, 3, 4, 2, 4, 5, 4, 6, 5, 7, 6]

    chart.setOption({
      tooltip: { trigger: 'axis' },
      legend: { bottom: 0 },
      grid: { left: '3%', right: '4%', top: '5%', bottom: '12%', containLabel: true },
      xAxis: { type: 'category', data: dates },
      yAxis: { type: 'value', name: '收入(千元)' },
      series: [
        { name: '新订阅', type: 'line', smooth: true, data: sub, color: '#22c55e' },
        { name: '续订', type: 'line', smooth: true, data: renew, color: '#3b82f6' },
        { name: '升级', type: 'line', smooth: true, data: upgrade, color: '#a855f7' },
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
