'use client'

import { useEffect, useRef } from 'react'
import * as echarts from 'echarts'

interface ApiCallsTrendChartProps {
  data?: Array<{ date: string; calls: number; success: number; failed: number }>
}

export default function ApiCallsTrendChart({ data }: ApiCallsTrendChartProps) {
  const chartRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!chartRef.current) return

    // 初始化 ECharts 实例
    const chart = echarts.init(chartRef.current)

    // Mock 数据（如果没有传入数据）
    const mockData = data || generateMockData()

    // 配置选项
    const option: echarts.EChartsOption = {
      tooltip: {
        trigger: 'axis',
        axisPointer: {
          type: 'cross',
          label: {
            backgroundColor: '#6a7985'
          }
        }
      },
      legend: {
        data: ['总调用', '成功', '失败'],
        bottom: 0,
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '10%',
        top: '5%',
        containLabel: true
      },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: mockData.map(item => item.date),
        axisLabel: {
          rotate: 45,
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
          name: '总调用',
          type: 'line',
          smooth: true,
          data: mockData.map(item => item.calls),
          itemStyle: {
            color: '#2563EB'
          },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: 'rgba(37, 99, 235, 0.3)' },
              { offset: 1, color: 'rgba(37, 99, 235, 0.05)' }
            ])
          }
        },
        {
          name: '成功',
          type: 'line',
          smooth: true,
          data: mockData.map(item => item.success),
          itemStyle: {
            color: '#10B981'
          }
        },
        {
          name: '失败',
          type: 'line',
          smooth: true,
          data: mockData.map(item => item.failed),
          itemStyle: {
            color: '#EF4444'
          }
        }
      ]
    }

    chart.setOption(option)

    // 响应式
    const handleResize = () => {
      chart.resize()
    }
    window.addEventListener('resize', handleResize)

    // 清理
    return () => {
      window.removeEventListener('resize', handleResize)
      chart.dispose()
    }
  }, [data])

  return <div ref={chartRef} className="w-full h-full" />
}

// 生成 Mock 数据
function generateMockData() {
  const data = []
  const today = new Date()
  
  for (let i = 29; i >= 0; i--) {
    const date = new Date(today)
    date.setDate(date.getDate() - i)
    const dateStr = `${date.getMonth() + 1}/${date.getDate()}`
    
    const calls = Math.floor(Math.random() * 3000) + 2000
    const failRate = Math.random() * 0.05 // 0-5% 失败率
    const failed = Math.floor(calls * failRate)
    const success = calls - failed
    
    data.push({
      date: dateStr,
      calls,
      success,
      failed
    })
  }
  
  return data
}
