'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function SellerApiTrendChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const chart = echarts.init(ref.current)

    const hours = Array.from({ length: 24 }).map((_, i) => `${i}:00`)
    const success = [410, 380, 350, 320, 340, 390, 510, 690, 740, 810, 860, 900, 920, 880, 830, 790, 760, 720, 680, 640, 600, 560, 500, 450]
    const failed = [8, 7, 5, 6, 6, 8, 10, 14, 13, 17, 19, 18, 16, 15, 12, 11, 10, 11, 9, 8, 8, 7, 6, 6]

    chart.setOption({
      tooltip: { trigger: 'axis' },
      legend: { bottom: 0 },
      grid: { left: '3%', right: '4%', top: '5%', bottom: '12%', containLabel: true },
      xAxis: { type: 'category', data: hours },
      yAxis: { type: 'value', name: '调用次数' },
      series: [
        { name: '成功', type: 'line', smooth: true, data: success, color: '#22c55e' },
        { name: '失败', type: 'line', smooth: true, data: failed, color: '#ef4444' },
        { name: '总计', type: 'line', smooth: true, data: success.map((v, i) => v + failed[i]), color: '#3b82f6' },
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
