'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

interface UsageData {
  name: string
  value: number
}

interface UsageDistributionChartProps {
  data?: UsageData[]
}

export default function UsageDistributionChart({ data }: UsageDistributionChartProps) {
  const chartRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!chartRef.current) return

    const chart = echarts.init(chartRef.current)

    // Mock 数据
    const mockData: UsageData[] = data || [
      { name: '企业工商风险数据', value: 6580 },
      { name: '消费者行为分析数据', value: 12350 },
      { name: '物流轨迹实时数据', value: 8920 },
    ]

    const option: echarts.EChartsOption = {
      tooltip: {
        trigger: 'item',
        formatter: '{b}: {c} ({d}%)'
      },
      legend: {
        orient: 'horizontal',
        left: 'center',
        bottom: 0,
        textStyle: {
          fontSize: 11,
        }
      },
      series: [
        {
          type: 'pie',
          radius: ['44%', '68%'],
          center: ['50%', '44%'],
          avoidLabelOverlap: false,
          itemStyle: {
            borderRadius: 8,
            borderColor: '#fff',
            borderWidth: 2
          },
          label: {
            show: false,
            position: 'center'
          },
          emphasis: {
            label: {
              show: true,
              fontSize: 14,
              fontWeight: 'bold',
              formatter: '{b}\n{d}%'
            }
          },
          labelLine: {
            show: false
          },
          data: mockData,
          color: ['#2563EB', '#10B981', '#8B5CF6', '#F59E0B', '#EF4444']
        }
      ]
    }

    chart.setOption(option)

    const handleResize = () => {
      chart.resize()
    }
    window.addEventListener('resize', handleResize)

    return () => {
      window.removeEventListener('resize', handleResize)
      chart.dispose()
    }
  }, [data])

  return <div ref={chartRef} className="w-full h-full" />
}
