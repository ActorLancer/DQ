'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

export default function ResponseTimeChart() {
  const chartRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!chartRef.current) return

    const chart = echarts.init(chartRef.current)

    // Mock 数据 - 响应时间分布
    const data = [
      { range: '0-100ms', count: 15230 },
      { range: '100-200ms', count: 8560 },
      { range: '200-300ms', count: 3420 },
      { range: '300-500ms', count: 1250 },
      { range: '500-1000ms', count: 380 },
      { range: '>1000ms', count: 160 },
    ]

    const option: echarts.EChartsOption = {
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          type: 'shadow'
        },
        formatter: (params: any) => {
          const item = params[0]
          return `${item.name}<br/>调用次数: ${item.value.toLocaleString()}`
        }
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: '5%',
        containLabel: true
      },
      xAxis: {
        type: 'category',
        data: data.map(item => item.range),
        axisLabel: {
          rotate: 30,
          fontSize: 11,
        }
      },
      yAxis: {
        type: 'value',
        name: '调用次数',
        axisLabel: {
          formatter: (value: number) => {
            if (value >= 1000) {
              return (value / 1000).toFixed(1) + 'k'
            }
            return value.toString()
          }
        }
      },
      series: [
        {
          type: 'bar',
          data: data.map(item => item.count),
          itemStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: '#8B5CF6' },
              { offset: 1, color: '#6366F1' }
            ])
          },
          barWidth: '60%',
          label: {
            show: true,
            position: 'top',
            formatter: (params: any) => {
              if (params.value >= 1000) {
                return (params.value / 1000).toFixed(1) + 'k'
              }
              return params.value
            },
            fontSize: 10,
          }
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
  }, [])

  return <div ref={chartRef} className="w-full h-full" />
}
