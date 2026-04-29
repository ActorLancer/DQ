'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function SellerRevenueCompositionChart() {
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
          radius: ['40%', '72%'],
          center: ['50%', '45%'],
          data: [
            { value: 420, name: '新订阅' },
            { value: 230, name: '续订' },
            { value: 150, name: '升级' },
          ],
          color: ['#22c55e', '#3b82f6', '#a855f7'],
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
