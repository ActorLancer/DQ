'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function SellerStatusCodeChart() {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!ref.current) return
    const chart = echarts.init(ref.current)

    chart.setOption({
      tooltip: { trigger: 'item', formatter: '{b}: {c} ({d}%)' },
      legend: { bottom: 0 },
      series: [
        {
          type: 'pie',
          radius: ['38%', '68%'],
          center: ['50%', '45%'],
          data: [
            { value: 9950, name: '200 OK' },
            { value: 35, name: '4xx 错误' },
            { value: 15, name: '5xx 错误' },
          ],
          color: ['#22c55e', '#eab308', '#ef4444'],
          itemStyle: { borderColor: '#fff', borderWidth: 2 },
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
