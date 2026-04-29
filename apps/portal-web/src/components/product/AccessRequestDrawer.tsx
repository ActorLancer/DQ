'use client'

import { useState } from 'react'
import { X, Check, AlertCircle } from 'lucide-react'
import type { Listing, PricingPlan } from '@/types'

interface AccessRequestDrawerProps {
  listing: Listing
  isOpen: boolean
  onClose: () => void
}

type Step = 1 | 2 | 3 | 4

export default function AccessRequestDrawer({ listing, isOpen, onClose }: AccessRequestDrawerProps) {
  const [currentStep, setCurrentStep] = useState<Step>(1)
  const [selectedPlan, setSelectedPlan] = useState<string>(listing.pricingPlans[0]?.id || '')
  const [formData, setFormData] = useState({
    usageSubject: '',
    usageDepartment: '',
    usageScenario: '',
    businessPurpose: '',
    expectedCalls: '',
    involvesPersonalInfo: false,
    usedForTraining: false,
    usedForRedistribution: false,
    dataRetentionDays: '90',
    contactPerson: '',
    contactPhone: '',
  })
  const [complianceChecks, setComplianceChecks] = useState({
    authorizedUseOnly: false,
    noUnauthorizedResale: false,
    prohibitedUsesAcknowledged: false,
    auditAccepted: false,
    informationAccurate: false,
  })
  const [submitStatus, setSubmitStatus] = useState<'idle' | 'submitting' | 'success' | 'error'>('idle')
  const [requestId, setRequestId] = useState<string>('')

  if (!isOpen) return null

  const handleNext = () => {
    if (currentStep < 4) {
      setCurrentStep((currentStep + 1) as Step)
    }
  }

  const handleBack = () => {
    if (currentStep > 1) {
      setCurrentStep((currentStep - 1) as Step)
    }
  }

  const handleSubmit = async () => {
    setSubmitStatus('submitting')
    
    // 模拟提交
    setTimeout(() => {
      setRequestId('req_20260428_000001')
      setSubmitStatus('success')
    }, 1500)
  }

  const allComplianceChecked = Object.values(complianceChecks).every((v) => v)

  return (
    <div className="fixed inset-0 z-50 overflow-hidden">
      {/* 背景遮罩 */}
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />

      {/* Drawer */}
      <div className="absolute right-0 top-0 h-full w-full max-w-2xl bg-white shadow-2xl animate-slide-in overflow-y-auto">
        {/* 头部 */}
        <div className="sticky top-0 z-10 bg-white border-b border-gray-200 px-6 py-4">
          <div className="flex items-center justify-between">
            <h2 className="text-xl font-bold text-gray-900">申请访问</h2>
            <button
              onClick={onClose}
              className="p-2 text-gray-400 hover:text-gray-600 rounded-lg hover:bg-gray-100"
            >
              <X className="w-5 h-5" />
            </button>
          </div>

          {/* 步骤指示器 */}
          {submitStatus !== 'success' && (
            <div className="mt-4 flex items-center">
              {[1, 2, 3, 4].map((step) => (
                <div key={step} className="flex items-center flex-1">
                  <div
                    className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
                      step === currentStep
                        ? 'bg-primary-600 text-white'
                        : step < currentStep
                        ? 'bg-success-600 text-white'
                        : 'bg-gray-200 text-gray-600'
                    }`}
                  >
                    {step < currentStep ? <Check className="w-4 h-4" /> : step}
                  </div>
                  {step < 4 && (
                    <div
                      className={`flex-1 h-1 mx-2 ${
                        step < currentStep ? 'bg-success-600' : 'bg-gray-200'
                      }`}
                    />
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        {/* 内容区 */}
        <div className="p-6">
          {submitStatus === 'success' ? (
            /* 提交成功状态 */
            <div className="text-center py-12">
              <div className="w-16 h-16 bg-success-100 rounded-full flex items-center justify-center mx-auto mb-4">
                <Check className="w-8 h-8 text-success-600" />
              </div>
              <h3 className="text-2xl font-bold text-gray-900 mb-2">申请已提交</h3>
              <p className="text-gray-600 mb-6">供应商将在 2 小时内处理您的申请</p>

              <div className="bg-gray-50 rounded-lg p-6 mb-6 text-left">
                <div className="space-y-3">
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-600">Request ID</span>
                    <code className="font-hash text-sm text-gray-900">{requestId}</code>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-600">工作流状态</span>
                    <span className="status-tag status-warning">待供应商审核</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-600">链状态</span>
                    <span className="status-tag status-info">未提交</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-sm text-gray-600">投影状态</span>
                    <span className="status-tag status-success">已投影</span>
                  </div>
                </div>
              </div>

              <button onClick={onClose} className="btn-primary">
                返回商品详情
              </button>
            </div>
          ) : (
            <>
              {/* Step 1: 选择套餐 */}
              {currentStep === 1 && (
                <div className="space-y-6">
                  <h3 className="text-lg font-bold text-gray-900">选择套餐</h3>
                  <div className="space-y-3">
                    {listing.pricingPlans.map((plan) => (
                      <label
                        key={plan.id}
                        className={`block p-4 border-2 rounded-lg cursor-pointer transition-colors ${
                          selectedPlan === plan.id
                            ? 'border-primary-600 bg-primary-50'
                            : 'border-gray-200 hover:border-gray-300'
                        }`}
                      >
                        <input
                          type="radio"
                          name="plan"
                          value={plan.id}
                          checked={selectedPlan === plan.id}
                          onChange={(e) => setSelectedPlan(e.target.value)}
                          className="sr-only"
                        />
                        <div className="flex items-center justify-between">
                          <div>
                            <div className="font-bold text-gray-900">{plan.name}</div>
                            <div className="text-sm text-gray-600 mt-1">
                              {plan.deliveryMethods.join(' / ')}
                            </div>
                          </div>
                          <div className="text-right">
                            <div className="text-2xl font-bold text-primary-600">
                              {plan.price ? `¥${plan.price}` : '面议'}
                            </div>
                            {plan.quota && (
                              <div className="text-xs text-gray-500 mt-1">
                                {plan.quota.toLocaleString()} 次调用
                              </div>
                            )}
                          </div>
                        </div>
                      </label>
                    ))}
                  </div>
                </div>
              )}

              {/* Step 2: 填写用途 */}
              {currentStep === 2 && (
                <div className="space-y-6">
                  <h3 className="text-lg font-bold text-gray-900">填写用途</h3>
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        使用主体 <span className="text-red-500">*</span>
                      </label>
                      <input
                        type="text"
                        value={formData.usageSubject}
                        onChange={(e) => setFormData({ ...formData, usageSubject: e.target.value })}
                        className="input"
                        placeholder="请输入使用主体名称"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        使用部门
                      </label>
                      <input
                        type="text"
                        value={formData.usageDepartment}
                        onChange={(e) => setFormData({ ...formData, usageDepartment: e.target.value })}
                        className="input"
                        placeholder="请输入使用部门"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        使用场景 <span className="text-red-500">*</span>
                      </label>
                      <textarea
                        value={formData.usageScenario}
                        onChange={(e) => setFormData({ ...formData, usageScenario: e.target.value })}
                        className="input min-h-[100px]"
                        placeholder="请详细描述数据使用场景"
                      />
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        业务用途 <span className="text-red-500">*</span>
                      </label>
                      <textarea
                        value={formData.businessPurpose}
                        onChange={(e) => setFormData({ ...formData, businessPurpose: e.target.value })}
                        className="input min-h-[100px]"
                        placeholder="请说明业务用途"
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          预计调用量
                        </label>
                        <input
                          type="text"
                          value={formData.expectedCalls}
                          onChange={(e) => setFormData({ ...formData, expectedCalls: e.target.value })}
                          className="input"
                          placeholder="例如: 10000/月"
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          数据保存周期
                        </label>
                        <select
                          value={formData.dataRetentionDays}
                          onChange={(e) => setFormData({ ...formData, dataRetentionDays: e.target.value })}
                          className="input"
                        >
                          <option value="30">30 天</option>
                          <option value="90">90 天</option>
                          <option value="180">180 天</option>
                          <option value="365">365 天</option>
                        </select>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          checked={formData.involvesPersonalInfo}
                          onChange={(e) => setFormData({ ...formData, involvesPersonalInfo: e.target.checked })}
                          className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
                        />
                        <span className="ml-2 text-sm text-gray-700">涉及个人信息处理</span>
                      </label>
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          checked={formData.usedForTraining}
                          onChange={(e) => setFormData({ ...formData, usedForTraining: e.target.checked })}
                          className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
                        />
                        <span className="ml-2 text-sm text-gray-700">用于模型训练</span>
                      </label>
                      <label className="flex items-center">
                        <input
                          type="checkbox"
                          checked={formData.usedForRedistribution}
                          onChange={(e) => setFormData({ ...formData, usedForRedistribution: e.target.checked })}
                          className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500"
                        />
                        <span className="ml-2 text-sm text-gray-700">用于再分发</span>
                      </label>
                    </div>
                  </div>
                </div>
              )}

              {/* Step 3: 合规确认 */}
              {currentStep === 3 && (
                <div className="space-y-6">
                  <h3 className="text-lg font-bold text-gray-900">合规确认</h3>
                  <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-4">
                    <div className="flex gap-2">
                      <AlertCircle className="w-5 h-5 text-yellow-600 flex-shrink-0 mt-0.5" />
                      <div className="text-sm text-yellow-800">
                        请仔细阅读并确认以下条款，这些是数据使用的法律约束
                      </div>
                    </div>
                  </div>
                  <div className="space-y-4">
                    <label className="flex items-start cursor-pointer">
                      <input
                        type="checkbox"
                        checked={complianceChecks.authorizedUseOnly}
                        onChange={(e) =>
                          setComplianceChecks({ ...complianceChecks, authorizedUseOnly: e.target.checked })
                        }
                        className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500 mt-1"
                      />
                      <span className="ml-3 text-sm text-gray-700">
                        我确认仅在授权范围内使用数据，不会超出申请的使用场景和用途
                      </span>
                    </label>
                    <label className="flex items-start cursor-pointer">
                      <input
                        type="checkbox"
                        checked={complianceChecks.noUnauthorizedResale}
                        onChange={(e) =>
                          setComplianceChecks({ ...complianceChecks, noUnauthorizedResale: e.target.checked })
                        }
                        className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500 mt-1"
                      />
                      <span className="ml-3 text-sm text-gray-700">
                        我确认不会进行未授权的转售或转授权
                      </span>
                    </label>
                    <label className="flex items-start cursor-pointer">
                      <input
                        type="checkbox"
                        checked={complianceChecks.prohibitedUsesAcknowledged}
                        onChange={(e) =>
                          setComplianceChecks({
                            ...complianceChecks,
                            prohibitedUsesAcknowledged: e.target.checked,
                          })
                        }
                        className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500 mt-1"
                      />
                      <span className="ml-3 text-sm text-gray-700">
                        我确认遵守禁止用途，不会将数据用于违法违规活动
                      </span>
                    </label>
                    <label className="flex items-start cursor-pointer">
                      <input
                        type="checkbox"
                        checked={complianceChecks.auditAccepted}
                        onChange={(e) =>
                          setComplianceChecks({ ...complianceChecks, auditAccepted: e.target.checked })
                        }
                        className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500 mt-1"
                      />
                      <span className="ml-3 text-sm text-gray-700">
                        我确认接受审计与调用记录留存，配合平台和供应商的合规检查
                      </span>
                    </label>
                    <label className="flex items-start cursor-pointer">
                      <input
                        type="checkbox"
                        checked={complianceChecks.informationAccurate}
                        onChange={(e) =>
                          setComplianceChecks({ ...complianceChecks, informationAccurate: e.target.checked })
                        }
                        className="w-4 h-4 text-primary-600 border-gray-300 rounded focus:ring-primary-500 mt-1"
                      />
                      <span className="ml-3 text-sm text-gray-700">
                        我确认提交的信息真实有效，如有虚假将承担相应法律责任
                      </span>
                    </label>
                  </div>
                </div>
              )}

              {/* Step 4: 提交审批 */}
              {currentStep === 4 && (
                <div className="space-y-6">
                  <h3 className="text-lg font-bold text-gray-900">确认提交</h3>
                  <div className="bg-gray-50 rounded-lg p-6 space-y-4">
                    <div>
                      <div className="text-sm text-gray-600 mb-1">商品名称</div>
                      <div className="font-medium text-gray-900">{listing.title}</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-600 mb-1">选择套餐</div>
                      <div className="font-medium text-gray-900">
                        {listing.pricingPlans.find((p) => p.id === selectedPlan)?.name}
                      </div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-600 mb-1">使用场景</div>
                      <div className="text-sm text-gray-900">{formData.usageScenario || '未填写'}</div>
                    </div>
                    <div>
                      <div className="text-sm text-gray-600 mb-1">合规确认</div>
                      <div className="text-sm text-success-600">✓ 已确认所有合规条款</div>
                    </div>
                  </div>

                  {submitStatus === 'submitting' && (
                    <div className="text-center py-8">
                      <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600 mx-auto mb-4"></div>
                      <div className="text-gray-600">正在提交申请...</div>
                    </div>
                  )}
                </div>
              )}

              {/* 底部按钮 */}
              {submitStatus !== 'submitting' && (
                <div className="flex gap-3 mt-8 pt-6 border-t border-gray-200">
                  {currentStep > 1 && (
                    <button onClick={handleBack} className="flex-1 btn-secondary">
                      上一步
                    </button>
                  )}
                  {currentStep < 4 ? (
                    <button onClick={handleNext} className="flex-1 btn-primary">
                      下一步
                    </button>
                  ) : (
                    <button
                      onClick={handleSubmit}
                      disabled={!allComplianceChecked}
                      className="flex-1 btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      提交申请
                    </button>
                  )}
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  )
}
