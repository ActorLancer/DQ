'use client'

import { useState } from 'react'
import { ChevronDown, ChevronUp } from 'lucide-react'

interface FilterSection {
  id: string
  title: string
  options: Array<{ value: string; label: string; count?: number }>
}

const FILTER_SECTIONS: FilterSection[] = [
  {
    id: 'industry',
    title: '行业分类',
    options: [
      { value: 'finance', label: '金融', count: 128 },
      { value: 'government', label: '政务', count: 85 },
      { value: 'healthcare', label: '医疗', count: 92 },
      { value: 'manufacturing', label: '工业', count: 67 },
      { value: 'logistics', label: '交通', count: 54 },
      { value: 'retail', label: '消费', count: 103 },
    ],
  },
  {
    id: 'deliveryMethod',
    title: '交付方式',
    options: [
      { value: 'API', label: 'API', count: 256 },
      { value: 'FILE', label: '文件', count: 142 },
      { value: 'SANDBOX', label: '沙箱', count: 38 },
      { value: 'PRIVACY_COMPUTING', label: '隐私计算', count: 21 },
    ],
  },
  {
    id: 'licenseType',
    title: '授权方式',
    options: [
      { value: 'TRIAL', label: '试用', count: 89 },
      { value: 'COMMERCIAL', label: '商用', count: 234 },
      { value: 'SUBSCRIPTION', label: '订阅', count: 178 },
      { value: 'ONE_TIME', label: '单次', count: 56 },
    ],
  },
  {
    id: 'pricingModel',
    title: '价格模式',
    options: [
      { value: 'TRIAL', label: '免费试用', count: 89 },
      { value: 'MONTHLY', label: '月付', count: 145 },
      { value: 'YEARLY', label: '年付', count: 98 },
      { value: 'USAGE_BASED', label: '按量', count: 67 },
      { value: 'CUSTOM', label: '定制', count: 43 },
    ],
  },
]

interface LeftFilterPanelProps {
  selectedFilters: Record<string, string[]>
  onFilterChange: (filterId: string, values: string[]) => void
}

export default function LeftFilterPanel({ selectedFilters, onFilterChange }: LeftFilterPanelProps) {
  const [expandedSections, setExpandedSections] = useState<Record<string, boolean>>({
    industry: true,
    deliveryMethod: true,
    licenseType: true,
    pricingModel: true,
  })

  const toggleSection = (sectionId: string) => {
    setExpandedSections((prev) => ({
      ...prev,
      [sectionId]: !prev[sectionId],
    }))
  }

  const handleCheckboxChange = (filterId: string, value: string, checked: boolean) => {
    const currentValues = selectedFilters[filterId] || []
    const newValues = checked
      ? [...currentValues, value]
      : currentValues.filter((v) => v !== value)
    onFilterChange(filterId, newValues)
  }

  return (
    <div className="w-full bg-white rounded-lg border border-gray-200 p-6">
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-lg font-bold text-gray-900">筛选条件</h3>
        <button
          onClick={() => onFilterChange('reset', [])}
          className="text-sm text-primary-600 hover:text-primary-700"
        >
          清空
        </button>
      </div>

      <div className="space-y-6">
        {FILTER_SECTIONS.map((section) => (
          <div key={section.id} className="border-b border-gray-100 pb-6 last:border-b-0">
            <button
              onClick={() => toggleSection(section.id)}
              className="flex items-center justify-between w-full mb-3"
            >
              <span className="font-medium text-gray-900">{section.title}</span>
              {expandedSections[section.id] ? (
                <ChevronUp className="w-4 h-4 text-gray-500" />
              ) : (
                <ChevronDown className="w-4 h-4 text-gray-500" />
              )}
            </button>

            {expandedSections[section.id] && (
              <div className="space-y-2">
                {section.options.map((option) => {
                  const isChecked = (selectedFilters[section.id] || []).includes(option.value)
                  return (
                    <label
                      key={option.value}
                      className="flex items-center justify-between cursor-pointer group"
                    >
                      <div className="flex items-center">
                        <input
                          type="checkbox"
                          checked={isChecked}
                          onChange={(e) =>
                            handleCheckboxChange(section.id, option.value, e.target.checked)
                          }
                          className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
                        />
                        <span className="ml-2 text-sm text-gray-700 group-hover:text-gray-900">
                          {option.label}
                        </span>
                      </div>
                      {option.count !== undefined && (
                        <span className="text-xs text-gray-500">{option.count}</span>
                      )}
                    </label>
                  )
                })}
              </div>
            )}
          </div>
        ))}

        {/* 其他筛选项 */}
        <div className="border-b border-gray-100 pb-6">
          <h4 className="font-medium text-gray-900 mb-3">质量评分</h4>
          <div className="space-y-2">
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
              />
              <span className="ml-2 text-sm text-gray-700">9.0 分以上</span>
            </label>
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
              />
              <span className="ml-2 text-sm text-gray-700">8.0 - 9.0 分</span>
            </label>
          </div>
        </div>

        <div className="border-b border-gray-100 pb-6">
          <h4 className="font-medium text-gray-900 mb-3">其他</h4>
          <div className="space-y-2">
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
              />
              <span className="ml-2 text-sm text-gray-700">支持试用</span>
            </label>
            <label className="flex items-center cursor-pointer">
              <input
                type="checkbox"
                className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
              />
              <span className="ml-2 text-sm text-gray-700">链上登记</span>
            </label>
          </div>
        </div>
      </div>
    </div>
  )
}
