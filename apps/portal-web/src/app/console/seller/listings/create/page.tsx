'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  Check, 
  ChevronRight, 
  AlertCircle, 
  Plus, 
  Trash2, 
  Upload, 
  FileText,
  Shield,
  DollarSign,
  Zap,
  Activity
} from 'lucide-react'

type Step = 1 | 2 | 3 | 4 | 5 | 6

const STEPS = [
  { id: 1, label: '基础信息', description: '商品名称、分类、简介' },
  { id: 2, label: 'Schema 与样例', description: '字段定义、样例数据' },
  { id: 3, label: '质量与合规', description: '数据来源、合规材料' },
  { id: 4, label: '交付配置', description: 'API、文件交付方式' },
  { id: 5, label: '套餐定价', description: '价格、额度、期限' },
  { id: 6, label: '提交审核', description: '确认并提交' },
]

export default function CreateListingPage() {
  const [currentStep, setCurrentStep] = useState<Step>(1)
  const [formData, setFormData] = useState({
    // Step 1
    title: '',
    industry: '',
    dataType: '',
    summary: '',
    coverageScope: '',
    updateFrequency: '',
    
    // Step 2
    schemaFields: [] as any[],
    sampleData: '',
    
    // Step 3
    dataSource: '',
    ownershipProof: '',
    privacyStrategy: '',
    qualityReport: '',
    complianceDocs: '',
    prohibitedUses: '',
    
    // Step 4
    deliveryMethods: [] as string[],
    apiEndpoint: '',
    apiAuthType: '',
    fileFormat: '',
    
    // Step 5
    pricingPlans: [] as any[],
  })

  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const handleNext = () => {
    if (currentStep < 6) {
      setCurrentStep((currentStep + 1) as Step)
    }
  }

  const handleBack = () => {
    if (currentStep > 1) {
      setCurrentStep((currentStep - 1) as Step)
    }
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:listings:write"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">创建数据商品</h1>
          <p className="text-gray-600">按照向导步骤完成商品信息填写</p>
        </div>

        <div className="grid grid-cols-12 gap-8">
          {/* 左侧步骤指示器 */}
          <div className="col-span-3">
            <div className="bg-white rounded-xl border border-gray-200 p-6 sticky top-28">
              <h3 className="font-bold text-gray-900 mb-4">创建步骤</h3>
              <div className="space-y-4">
                {STEPS.map((step, index) => {
                  const isActive = step.id === currentStep
                  const isCompleted = step.id < currentStep
                  
                  return (
                    <div key={step.id} className="relative">
                      {index < STEPS.length - 1 && (
                        <div
                          className={`absolute left-4 top-10 w-0.5 h-12 ${
                            isCompleted ? 'bg-success-600' : 'bg-gray-200'
                          }`}
                        />
                      )}
                      <div className="flex items-start gap-3">
                        <div
                          className={`w-8 h-8 rounded-full flex items-center justify-center flex-shrink-0 ${
                            isCompleted
                              ? 'bg-success-600 text-white'
                              : isActive
                              ? 'bg-primary-600 text-white'
                              : 'bg-gray-200 text-gray-600'
                          }`}
                        >
                          {isCompleted ? (
                            <Check className="w-4 h-4" />
                          ) : (
                            <span className="text-sm font-medium">{step.id}</span>
                          )}
                        </div>
                        <div className="flex-1">
                          <div
                            className={`text-sm font-medium mb-1 ${
                              isActive ? 'text-primary-600' : isCompleted ? 'text-gray-900' : 'text-gray-600'
                            }`}
                          >
                            {step.label}
                          </div>
                          <div className="text-xs text-gray-500">{step.description}</div>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            </div>
          </div>

          {/* 右侧表单内容 */}
          <div className="col-span-9">
            <div className="bg-white rounded-xl border border-gray-200 p-8">
              {/* Step 1: 基础信息 */}
              {currentStep === 1 && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-2xl font-bold text-gray-900 mb-2">基础信息</h2>
                    <p className="text-gray-600">填写商品的基本信息</p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      商品名称 <span className="text-red-500">*</span>
                    </label>
                    <input
                      type="text"
                      value={formData.title}
                      onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                      placeholder="例如：企业工商风险数据"
                      className="input"
                    />
                    <p className="text-xs text-gray-500 mt-1">清晰、准确地描述您的数据商品</p>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        行业分类 <span className="text-red-500">*</span>
                      </label>
                      <select
                        value={formData.industry}
                        onChange={(e) => setFormData({ ...formData, industry: e.target.value })}
                        className="input"
                      >
                        <option value="">请选择</option>
                        <option value="finance">金融</option>
                        <option value="government">政务</option>
                        <option value="healthcare">医疗</option>
                        <option value="manufacturing">工业</option>
                        <option value="logistics">交通</option>
                        <option value="retail">消费</option>
                        <option value="enterprise">企业服务</option>
                        <option value="energy">能源</option>
                      </select>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        数据类型 <span className="text-red-500">*</span>
                      </label>
                      <input
                        type="text"
                        value={formData.dataType}
                        onChange={(e) => setFormData({ ...formData, dataType: e.target.value })}
                        placeholder="例如：企业征信"
                        className="input"
                      />
                    </div>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      商品简介 <span className="text-red-500">*</span>
                    </label>
                    <textarea
                      value={formData.summary}
                      onChange={(e) => setFormData({ ...formData, summary: e.target.value })}
                      placeholder="详细描述数据商品的内容、特点和价值..."
                      className="input min-h-[120px]"
                    />
                    <p className="text-xs text-gray-500 mt-1">建议 100-500 字</p>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        覆盖范围
                      </label>
                      <input
                        type="text"
                        value={formData.coverageScope}
                        onChange={(e) => setFormData({ ...formData, coverageScope: e.target.value })}
                        placeholder="例如：全国"
                        className="input"
                      />
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        更新频率
                      </label>
                      <select
                        value={formData.updateFrequency}
                        onChange={(e) => setFormData({ ...formData, updateFrequency: e.target.value })}
                        className="input"
                      >
                        <option value="">请选择</option>
                        <option value="realtime">实时更新</option>
                        <option value="daily">每日更新</option>
                        <option value="weekly">每周更新</option>
                        <option value="monthly">每月更新</option>
                        <option value="quarterly">每季度更新</option>
                      </select>
                    </div>
                  </div>

                  <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <div className="flex gap-3">
                      <AlertCircle className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
                      <div className="text-sm text-blue-800">
                        <p className="font-medium mb-1">填写提示</p>
                        <ul className="list-disc list-inside space-y-1 text-blue-700">
                          <li>商品名称应简洁明了，突出核心价值</li>
                          <li>简介需要详细说明数据内容、来源和应用场景</li>
                          <li>准确选择行业分类有助于买方快速找到您的商品</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Step 2: Schema 与样例 */}
              {currentStep === 2 && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-2xl font-bold text-gray-900 mb-2">Schema 与样例</h2>
                    <p className="text-gray-600">定义数据字段和提供样例数据</p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      数据字段定义 <span className="text-red-500">*</span>
                    </label>
                    <div className="space-y-3">
                      {formData.schemaFields.length === 0 && (
                        <div className="text-center py-8 border-2 border-dashed border-gray-300 rounded-lg">
                          <FileText className="w-12 h-12 text-gray-400 mx-auto mb-2" />
                          <p className="text-sm text-gray-600 mb-4">还没有添加字段</p>
                          <button
                            onClick={() => setFormData({
                              ...formData,
                              schemaFields: [{
                                name: '',
                                type: 'string',
                                description: '',
                                required: true,
                                sensitive: false,
                                maskStrategy: 'none'
                              }]
                            })}
                            className="inline-flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
                          >
                            <Plus className="w-4 h-4" />
                            <span>添加字段</span>
                          </button>
                        </div>
                      )}

                      {formData.schemaFields.map((field, index) => (
                        <div key={index} className="border border-gray-200 rounded-lg p-4">
                          <div className="grid grid-cols-12 gap-4">
                            <div className="col-span-3">
                              <input
                                type="text"
                                placeholder="字段名"
                                value={field.name}
                                onChange={(e) => {
                                  const newFields = [...formData.schemaFields]
                                  newFields[index].name = e.target.value
                                  setFormData({ ...formData, schemaFields: newFields })
                                }}
                                className="input text-sm"
                              />
                            </div>
                            <div className="col-span-2">
                              <select
                                value={field.type}
                                onChange={(e) => {
                                  const newFields = [...formData.schemaFields]
                                  newFields[index].type = e.target.value
                                  setFormData({ ...formData, schemaFields: newFields })
                                }}
                                className="input text-sm"
                              >
                                <option value="string">字符串</option>
                                <option value="number">数字</option>
                                <option value="boolean">布尔</option>
                                <option value="date">日期</option>
                                <option value="object">对象</option>
                                <option value="array">数组</option>
                              </select>
                            </div>
                            <div className="col-span-4">
                              <input
                                type="text"
                                placeholder="字段描述"
                                value={field.description}
                                onChange={(e) => {
                                  const newFields = [...formData.schemaFields]
                                  newFields[index].description = e.target.value
                                  setFormData({ ...formData, schemaFields: newFields })
                                }}
                                className="input text-sm"
                              />
                            </div>
                            <div className="col-span-2 flex items-center gap-2">
                              <label className="flex items-center gap-1 text-xs">
                                <input
                                  type="checkbox"
                                  checked={field.required}
                                  onChange={(e) => {
                                    const newFields = [...formData.schemaFields]
                                    newFields[index].required = e.target.checked
                                    setFormData({ ...formData, schemaFields: newFields })
                                  }}
                                  className="rounded"
                                />
                                必填
                              </label>
                              <label className="flex items-center gap-1 text-xs">
                                <input
                                  type="checkbox"
                                  checked={field.sensitive}
                                  onChange={(e) => {
                                    const newFields = [...formData.schemaFields]
                                    newFields[index].sensitive = e.target.checked
                                    setFormData({ ...formData, schemaFields: newFields })
                                  }}
                                  className="rounded"
                                />
                                敏感
                              </label>
                            </div>
                            <div className="col-span-1 flex items-center justify-end">
                              <button
                                onClick={() => {
                                  const newFields = formData.schemaFields.filter((_, i) => i !== index)
                                  setFormData({ ...formData, schemaFields: newFields })
                                }}
                                className="p-2 text-red-600 hover:bg-red-50 rounded-lg"
                              >
                                <Trash2 className="w-4 h-4" />
                              </button>
                            </div>
                          </div>
                        </div>
                      ))}

                      {formData.schemaFields.length > 0 && (
                        <button
                          onClick={() => setFormData({
                            ...formData,
                            schemaFields: [...formData.schemaFields, {
                              name: '',
                              type: 'string',
                              description: '',
                              required: true,
                              sensitive: false,
                              maskStrategy: 'none'
                            }]
                          })}
                          className="w-full py-2 border-2 border-dashed border-gray-300 rounded-lg text-gray-600 hover:border-primary-500 hover:text-primary-600 text-sm font-medium"
                        >
                          + 添加字段
                        </button>
                      )}
                    </div>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      样例数据 <span className="text-red-500">*</span>
                    </label>
                    <textarea
                      value={formData.sampleData}
                      onChange={(e) => setFormData({ ...formData, sampleData: e.target.value })}
                      placeholder='请提供 JSON 格式的样例数据，例如：&#10;{&#10;  "companyName": "某某科技有限公司",&#10;  "creditCode": "91110000XXXXXXXXXX",&#10;  "riskLevel": "低风险"&#10;}'
                      className="input min-h-[200px] font-mono text-sm"
                    />
                    <p className="text-xs text-gray-500 mt-1">提供真实的样例数据有助于买方理解数据结构</p>
                  </div>

                  <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <div className="flex gap-3">
                      <AlertCircle className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
                      <div className="text-sm text-blue-800">
                        <p className="font-medium mb-1">Schema 定义提示</p>
                        <ul className="list-disc list-inside space-y-1 text-blue-700">
                          <li>字段名使用驼峰命名法（camelCase）</li>
                          <li>敏感字段需要标记并设置脱敏策略</li>
                          <li>样例数据应该是真实且有代表性的</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Step 3: 质量与合规 */}
              {currentStep === 3 && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-2xl font-bold text-gray-900 mb-2">质量与合规</h2>
                    <p className="text-gray-600">上传数据来源说明和合规材料</p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      数据来源说明 <span className="text-red-500">*</span>
                    </label>
                    <textarea
                      value={formData.dataSource}
                      onChange={(e) => setFormData({ ...formData, dataSource: e.target.value })}
                      placeholder="详细说明数据的来源渠道、采集方式、更新机制..."
                      className="input min-h-[100px]"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      权属证明 <span className="text-red-500">*</span>
                    </label>
                    <div className="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center hover:border-primary-500 cursor-pointer">
                      <Upload className="w-12 h-12 text-gray-400 mx-auto mb-2" />
                      <p className="text-sm text-gray-600 mb-1">点击上传或拖拽文件到此处</p>
                      <p className="text-xs text-gray-500">支持 PDF、JPG、PNG，最大 10MB</p>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">上传数据权属证明文件（授权书、合同等）</p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      隐私保护策略 <span className="text-red-500">*</span>
                    </label>
                    <textarea
                      value={formData.privacyStrategy}
                      onChange={(e) => setFormData({ ...formData, privacyStrategy: e.target.value })}
                      placeholder="说明如何保护个人隐私和敏感信息，包括脱敏、加密等措施..."
                      className="input min-h-[100px]"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      质量报告
                    </label>
                    <div className="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center hover:border-primary-500 cursor-pointer">
                      <Upload className="w-12 h-12 text-gray-400 mx-auto mb-2" />
                      <p className="text-sm text-gray-600 mb-1">上传数据质量报告</p>
                      <p className="text-xs text-gray-500">包括准确性、完整性、及时性等指标</p>
                    </div>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      合规材料
                    </label>
                    <div className="border-2 border-dashed border-gray-300 rounded-lg p-6 text-center hover:border-primary-500 cursor-pointer">
                      <Upload className="w-12 h-12 text-gray-400 mx-auto mb-2" />
                      <p className="text-sm text-gray-600 mb-1">上传合规相关材料</p>
                      <p className="text-xs text-gray-500">如行业许可证、安全认证等</p>
                    </div>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-2">
                      禁止使用场景 <span className="text-red-500">*</span>
                    </label>
                    <textarea
                      value={formData.prohibitedUses}
                      onChange={(e) => setFormData({ ...formData, prohibitedUses: e.target.value })}
                      placeholder="明确列出数据不得用于的场景，例如：不得用于非法催收、不得用于歧视性决策..."
                      className="input min-h-[100px]"
                    />
                  </div>

                  <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                    <div className="flex gap-3">
                      <Shield className="w-5 h-5 text-yellow-600 flex-shrink-0 mt-0.5" />
                      <div className="text-sm text-yellow-800">
                        <p className="font-medium mb-1">合规要求</p>
                        <ul className="list-disc list-inside space-y-1 text-yellow-700">
                          <li>必须提供真实有效的数据来源说明</li>
                          <li>涉及个人信息的数据必须符合《个人信息保护法》</li>
                          <li>必须明确数据使用的合法边界</li>
                          <li>平台将对合规材料进行严格审核</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Step 4: 交付配置 */}
              {currentStep === 4 && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-2xl font-bold text-gray-900 mb-2">交付配置</h2>
                    <p className="text-gray-600">配置 API 或文件交付方式</p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-3">
                      交付方式 <span className="text-red-500">*</span>
                    </label>
                    <div className="grid grid-cols-3 gap-4">
                      <label className={`border-2 rounded-lg p-4 cursor-pointer transition-all ${
                        formData.deliveryMethods.includes('api') 
                          ? 'border-primary-500 bg-primary-50' 
                          : 'border-gray-200 hover:border-gray-300'
                      }`}>
                        <input
                          type="checkbox"
                          checked={formData.deliveryMethods.includes('api')}
                          onChange={(e) => {
                            const methods = e.target.checked
                              ? [...formData.deliveryMethods, 'api']
                              : formData.deliveryMethods.filter(m => m !== 'api')
                            setFormData({ ...formData, deliveryMethods: methods })
                          }}
                          className="sr-only"
                        />
                        <Zap className="w-8 h-8 text-primary-600 mx-auto mb-2" />
                        <div className="text-center">
                          <div className="font-medium text-gray-900 mb-1">API 接口</div>
                          <div className="text-xs text-gray-600">实时查询</div>
                        </div>
                      </label>

                      <label className={`border-2 rounded-lg p-4 cursor-pointer transition-all ${
                        formData.deliveryMethods.includes('file') 
                          ? 'border-primary-500 bg-primary-50' 
                          : 'border-gray-200 hover:border-gray-300'
                      }`}>
                        <input
                          type="checkbox"
                          checked={formData.deliveryMethods.includes('file')}
                          onChange={(e) => {
                            const methods = e.target.checked
                              ? [...formData.deliveryMethods, 'file']
                              : formData.deliveryMethods.filter(m => m !== 'file')
                            setFormData({ ...formData, deliveryMethods: methods })
                          }}
                          className="sr-only"
                        />
                        <FileText className="w-8 h-8 text-primary-600 mx-auto mb-2" />
                        <div className="text-center">
                          <div className="font-medium text-gray-900 mb-1">文件下载</div>
                          <div className="text-xs text-gray-600">批量数据</div>
                        </div>
                      </label>

                      <label className={`border-2 rounded-lg p-4 cursor-pointer transition-all ${
                        formData.deliveryMethods.includes('stream') 
                          ? 'border-primary-500 bg-primary-50' 
                          : 'border-gray-200 hover:border-gray-300'
                      }`}>
                        <input
                          type="checkbox"
                          checked={formData.deliveryMethods.includes('stream')}
                          onChange={(e) => {
                            const methods = e.target.checked
                              ? [...formData.deliveryMethods, 'stream']
                              : formData.deliveryMethods.filter(m => m !== 'stream')
                            setFormData({ ...formData, deliveryMethods: methods })
                          }}
                          className="sr-only"
                        />
                        <Activity className="w-8 h-8 text-primary-600 mx-auto mb-2" />
                        <div className="text-center">
                          <div className="font-medium text-gray-900 mb-1">数据流</div>
                          <div className="text-xs text-gray-600">实时推送</div>
                        </div>
                      </label>
                    </div>
                  </div>

                  {formData.deliveryMethods.includes('api') && (
                    <div className="border border-gray-200 rounded-lg p-6 space-y-4">
                      <h3 className="font-bold text-gray-900">API 配置</h3>
                      
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          API Endpoint <span className="text-red-500">*</span>
                        </label>
                        <input
                          type="text"
                          value={formData.apiEndpoint}
                          onChange={(e) => setFormData({ ...formData, apiEndpoint: e.target.value })}
                          placeholder="https://api.example.com/v1/data"
                          className="input"
                        />
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          鉴权方式 <span className="text-red-500">*</span>
                        </label>
                        <select
                          value={formData.apiAuthType}
                          onChange={(e) => setFormData({ ...formData, apiAuthType: e.target.value })}
                          className="input"
                        >
                          <option value="">请选择</option>
                          <option value="apikey">API Key</option>
                          <option value="oauth2">OAuth 2.0</option>
                          <option value="jwt">JWT Token</option>
                        </select>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          API 文档
                        </label>
                        <div className="border-2 border-dashed border-gray-300 rounded-lg p-4 text-center hover:border-primary-500 cursor-pointer">
                          <Upload className="w-8 h-8 text-gray-400 mx-auto mb-2" />
                          <p className="text-sm text-gray-600">上传 API 文档（OpenAPI/Swagger）</p>
                        </div>
                      </div>
                    </div>
                  )}

                  {formData.deliveryMethods.includes('file') && (
                    <div className="border border-gray-200 rounded-lg p-6 space-y-4">
                      <h3 className="font-bold text-gray-900">文件配置</h3>
                      
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          文件格式 <span className="text-red-500">*</span>
                        </label>
                        <div className="grid grid-cols-4 gap-3">
                          {['CSV', 'JSON', 'Excel', 'Parquet'].map(format => (
                            <label key={format} className="flex items-center gap-2 p-3 border border-gray-200 rounded-lg cursor-pointer hover:border-primary-500">
                              <input
                                type="radio"
                                name="fileFormat"
                                value={format.toLowerCase()}
                                checked={formData.fileFormat === format.toLowerCase()}
                                onChange={(e) => setFormData({ ...formData, fileFormat: e.target.value })}
                                className="text-primary-600"
                              />
                              <span className="text-sm font-medium">{format}</span>
                            </label>
                          ))}
                        </div>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          更新周期
                        </label>
                        <select className="input">
                          <option value="daily">每日更新</option>
                          <option value="weekly">每周更新</option>
                          <option value="monthly">每月更新</option>
                        </select>
                      </div>
                    </div>
                  )}

                  <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <div className="flex gap-3">
                      <AlertCircle className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
                      <div className="text-sm text-blue-800">
                        <p className="font-medium mb-1">交付配置提示</p>
                        <ul className="list-disc list-inside space-y-1 text-blue-700">
                          <li>API 方式适合实时查询场景</li>
                          <li>文件方式适合批量数据分析</li>
                          <li>可以同时支持多种交付方式</li>
                          <li>请确保 API 文档完整准确</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Step 5: 套餐定价 */}
              {currentStep === 5 && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-2xl font-bold text-gray-900 mb-2">套餐定价</h2>
                    <p className="text-gray-600">设置价格、额度和授权期限</p>
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-3">
                      定价模型 <span className="text-red-500">*</span>
                    </label>
                    <div className="grid grid-cols-3 gap-4">
                      <label className="border-2 border-gray-200 rounded-lg p-4 cursor-pointer hover:border-primary-500">
                        <input type="radio" name="pricingModel" className="sr-only" />
                        <DollarSign className="w-8 h-8 text-primary-600 mx-auto mb-2" />
                        <div className="text-center">
                          <div className="font-medium text-gray-900 mb-1">按量计费</div>
                          <div className="text-xs text-gray-600">按调用次数收费</div>
                        </div>
                      </label>

                      <label className="border-2 border-gray-200 rounded-lg p-4 cursor-pointer hover:border-primary-500">
                        <input type="radio" name="pricingModel" className="sr-only" />
                        <DollarSign className="w-8 h-8 text-primary-600 mx-auto mb-2" />
                        <div className="text-center">
                          <div className="font-medium text-gray-900 mb-1">订阅制</div>
                          <div className="text-xs text-gray-600">按月/年订阅</div>
                        </div>
                      </label>

                      <label className="border-2 border-gray-200 rounded-lg p-4 cursor-pointer hover:border-primary-500">
                        <input type="radio" name="pricingModel" className="sr-only" />
                        <DollarSign className="w-8 h-8 text-primary-600 mx-auto mb-2" />
                        <div className="text-center">
                          <div className="font-medium text-gray-900 mb-1">混合模式</div>
                          <div className="text-xs text-gray-600">基础费+超量费</div>
                        </div>
                      </label>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <h3 className="font-bold text-gray-900">套餐配置</h3>
                      <button
                        onClick={() => setFormData({
                          ...formData,
                          pricingPlans: [...formData.pricingPlans, {
                            name: '',
                            price: 0,
                            quota: 0,
                            duration: 30,
                            features: []
                          }]
                        })}
                        className="flex items-center gap-2 px-4 py-2 text-primary-600 border border-primary-600 rounded-lg hover:bg-primary-50"
                      >
                        <Plus className="w-4 h-4" />
                        <span>添加套餐</span>
                      </button>
                    </div>

                    {formData.pricingPlans.length === 0 && (
                      <div className="text-center py-12 border-2 border-dashed border-gray-300 rounded-lg">
                        <DollarSign className="w-12 h-12 text-gray-400 mx-auto mb-2" />
                        <p className="text-sm text-gray-600 mb-4">还没有添加套餐</p>
                        <button
                          onClick={() => setFormData({
                            ...formData,
                            pricingPlans: [
                              {
                                name: '试用版',
                                price: 0,
                                quota: 100,
                                duration: 7,
                                features: ['基础功能', '7天试用']
                              },
                              {
                                name: '标准版',
                                price: 999,
                                quota: 10000,
                                duration: 30,
                                features: ['全部功能', '技术支持']
                              },
                              {
                                name: '企业版',
                                price: 9999,
                                quota: 100000,
                                duration: 365,
                                features: ['全部功能', '专属客服', '定制开发']
                              }
                            ]
                          })}
                          className="inline-flex items-center gap-2 px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
                        >
                          <Plus className="w-4 h-4" />
                          <span>使用默认套餐模板</span>
                        </button>
                      </div>
                    )}

                    <div className="grid grid-cols-3 gap-4">
                      {formData.pricingPlans.map((plan, index) => (
                        <div key={index} className="border border-gray-200 rounded-lg p-6 relative">
                          <button
                            onClick={() => {
                              const newPlans = formData.pricingPlans.filter((_, i) => i !== index)
                              setFormData({ ...formData, pricingPlans: newPlans })
                            }}
                            className="absolute top-4 right-4 p-1 text-red-600 hover:bg-red-50 rounded"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>

                          <div className="space-y-4">
                            <div>
                              <label className="block text-xs font-medium text-gray-700 mb-1">套餐名称</label>
                              <input
                                type="text"
                                value={plan.name}
                                onChange={(e) => {
                                  const newPlans = [...formData.pricingPlans]
                                  newPlans[index].name = e.target.value
                                  setFormData({ ...formData, pricingPlans: newPlans })
                                }}
                                placeholder="例如：标准版"
                                className="input text-sm"
                              />
                            </div>

                            <div>
                              <label className="block text-xs font-medium text-gray-700 mb-1">价格（元）</label>
                              <input
                                type="number"
                                value={plan.price}
                                onChange={(e) => {
                                  const newPlans = [...formData.pricingPlans]
                                  newPlans[index].price = Number(e.target.value)
                                  setFormData({ ...formData, pricingPlans: newPlans })
                                }}
                                className="input text-sm"
                              />
                            </div>

                            <div>
                              <label className="block text-xs font-medium text-gray-700 mb-1">调用额度</label>
                              <input
                                type="number"
                                value={plan.quota}
                                onChange={(e) => {
                                  const newPlans = [...formData.pricingPlans]
                                  newPlans[index].quota = Number(e.target.value)
                                  setFormData({ ...formData, pricingPlans: newPlans })
                                }}
                                className="input text-sm"
                              />
                            </div>

                            <div>
                              <label className="block text-xs font-medium text-gray-700 mb-1">有效期（天）</label>
                              <input
                                type="number"
                                value={plan.duration}
                                onChange={(e) => {
                                  const newPlans = [...formData.pricingPlans]
                                  newPlans[index].duration = Number(e.target.value)
                                  setFormData({ ...formData, pricingPlans: newPlans })
                                }}
                                className="input text-sm"
                              />
                            </div>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
                    <div className="flex gap-3">
                      <AlertCircle className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
                      <div className="text-sm text-blue-800">
                        <p className="font-medium mb-1">定价建议</p>
                        <ul className="list-disc list-inside space-y-1 text-blue-700">
                          <li>建议提供免费试用套餐，降低买方决策门槛</li>
                          <li>标准版适合中小企业，企业版适合大型客户</li>
                          <li>定价应考虑数据价值、市场行情和成本</li>
                          <li>可以根据市场反馈随时调整价格</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>
              )}

              {/* Step 6: 提交审核 */}
              {currentStep === 6 && (
                <div className="space-y-6">
                  <div>
                    <h2 className="text-2xl font-bold text-gray-900 mb-2">提交审核</h2>
                    <p className="text-gray-600">确认信息并提交平台审核</p>
                  </div>

                  {/* 基础信息摘要 */}
                  <div className="bg-white border border-gray-200 rounded-lg p-6">
                    <h3 className="font-bold text-gray-900 mb-4 flex items-center gap-2">
                      <Check className="w-5 h-5 text-success-600" />
                      基础信息
                    </h3>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <div className="text-sm text-gray-600 mb-1">商品名称</div>
                        <div className="font-medium text-gray-900">{formData.title || '未填写'}</div>
                      </div>
                      <div>
                        <div className="text-sm text-gray-600 mb-1">行业分类</div>
                        <div className="font-medium text-gray-900">{formData.industry || '未选择'}</div>
                      </div>
                      <div>
                        <div className="text-sm text-gray-600 mb-1">数据类型</div>
                        <div className="font-medium text-gray-900">{formData.dataType || '未填写'}</div>
                      </div>
                      <div>
                        <div className="text-sm text-gray-600 mb-1">更新频率</div>
                        <div className="font-medium text-gray-900">{formData.updateFrequency || '未选择'}</div>
                      </div>
                      <div className="col-span-2">
                        <div className="text-sm text-gray-600 mb-1">商品简介</div>
                        <div className="text-sm text-gray-900">{formData.summary || '未填写'}</div>
                      </div>
                    </div>
                  </div>

                  {/* Schema 摘要 */}
                  <div className="bg-white border border-gray-200 rounded-lg p-6">
                    <h3 className="font-bold text-gray-900 mb-4 flex items-center gap-2">
                      <Check className="w-5 h-5 text-success-600" />
                      Schema 与样例
                    </h3>
                    <div>
                      <div className="text-sm text-gray-600 mb-2">数据字段</div>
                      <div className="text-sm text-gray-900">
                        {formData.schemaFields.length > 0 
                          ? `已定义 ${formData.schemaFields.length} 个字段` 
                          : '未定义字段'}
                      </div>
                      {formData.schemaFields.length > 0 && (
                        <div className="mt-2 flex flex-wrap gap-2">
                          {formData.schemaFields.map((field, index) => (
                            <span key={index} className="px-2 py-1 bg-gray-100 text-gray-700 rounded text-xs">
                              {field.name || `字段${index + 1}`} ({field.type})
                            </span>
                          ))}
                        </div>
                      )}
                    </div>
                  </div>

                  {/* 质量与合规摘要 */}
                  <div className="bg-white border border-gray-200 rounded-lg p-6">
                    <h3 className="font-bold text-gray-900 mb-4 flex items-center gap-2">
                      <Check className="w-5 h-5 text-success-600" />
                      质量与合规
                    </h3>
                    <div className="space-y-3">
                      <div>
                        <div className="text-sm text-gray-600 mb-1">数据来源</div>
                        <div className="text-sm text-gray-900">
                          {formData.dataSource ? '已填写' : '未填写'}
                        </div>
                      </div>
                      <div>
                        <div className="text-sm text-gray-600 mb-1">隐私保护策略</div>
                        <div className="text-sm text-gray-900">
                          {formData.privacyStrategy ? '已填写' : '未填写'}
                        </div>
                      </div>
                      <div>
                        <div className="text-sm text-gray-600 mb-1">禁止使用场景</div>
                        <div className="text-sm text-gray-900">
                          {formData.prohibitedUses ? '已填写' : '未填写'}
                        </div>
                      </div>
                    </div>
                  </div>

                  {/* 交付配置摘要 */}
                  <div className="bg-white border border-gray-200 rounded-lg p-6">
                    <h3 className="font-bold text-gray-900 mb-4 flex items-center gap-2">
                      <Check className="w-5 h-5 text-success-600" />
                      交付配置
                    </h3>
                    <div>
                      <div className="text-sm text-gray-600 mb-2">交付方式</div>
                      <div className="flex flex-wrap gap-2">
                        {formData.deliveryMethods.length > 0 ? (
                          formData.deliveryMethods.map(method => (
                            <span key={method} className="px-3 py-1 bg-primary-100 text-primary-700 rounded-full text-sm font-medium">
                              {method === 'api' ? 'API 接口' : method === 'file' ? '文件下载' : '数据流'}
                            </span>
                          ))
                        ) : (
                          <span className="text-sm text-gray-900">未选择</span>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* 套餐定价摘要 */}
                  <div className="bg-white border border-gray-200 rounded-lg p-6">
                    <h3 className="font-bold text-gray-900 mb-4 flex items-center gap-2">
                      <Check className="w-5 h-5 text-success-600" />
                      套餐定价
                    </h3>
                    <div>
                      <div className="text-sm text-gray-600 mb-2">定价套餐</div>
                      {formData.pricingPlans.length > 0 ? (
                        <div className="grid grid-cols-3 gap-4">
                          {formData.pricingPlans.map((plan, index) => (
                            <div key={index} className="border border-gray-200 rounded-lg p-4">
                              <div className="font-medium text-gray-900 mb-2">{plan.name || `套餐${index + 1}`}</div>
                              <div className="text-2xl font-bold text-primary-600 mb-2">
                                ¥{plan.price.toLocaleString()}
                              </div>
                              <div className="text-xs text-gray-600 space-y-1">
                                <div>额度：{plan.quota.toLocaleString()} 次</div>
                                <div>有效期：{plan.duration} 天</div>
                              </div>
                            </div>
                          ))}
                        </div>
                      ) : (
                        <div className="text-sm text-gray-900">未配置套餐</div>
                      )}
                    </div>
                  </div>

                  {/* 提交确认 */}
                  <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                    <div className="flex gap-3">
                      <AlertCircle className="w-5 h-5 text-yellow-600 flex-shrink-0 mt-0.5" />
                      <div className="text-sm text-yellow-800">
                        <p className="font-medium mb-1">提交前确认</p>
                        <ul className="list-disc list-inside space-y-1 text-yellow-700">
                          <li>所有必填信息已完整填写</li>
                          <li>数据来源和合规材料已上传</li>
                          <li>定价策略已设置完成</li>
                          <li>提交后将进入平台审核流程，预计 1-3 个工作日</li>
                          <li>审核期间可以查看审核进度，但无法修改</li>
                        </ul>
                      </div>
                    </div>
                  </div>

                  {/* 服务协议 */}
                  <div className="flex items-start gap-3 p-4 bg-gray-50 rounded-lg">
                    <input type="checkbox" className="mt-1" />
                    <div className="text-sm text-gray-700">
                      我已阅读并同意 <a href="#" className="text-primary-600 hover:underline">《数据供应商服务协议》</a> 和 <a href="#" className="text-primary-600 hover:underline">《数据合规承诺书》</a>，承诺所提供的数据来源合法、内容真实，并承担相应的法律责任。
                    </div>
                  </div>
                </div>
              )}

              {/* 底部按钮 */}
              <div className="flex items-center justify-between mt-8 pt-6 border-t border-gray-200">
                <button
                  onClick={handleBack}
                  disabled={currentStep === 1}
                  className="px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed font-medium"
                >
                  上一步
                </button>

                <div className="flex items-center gap-3">
                  <button className="px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                    保存草稿
                  </button>
                  {currentStep < 6 ? (
                    <button
                      onClick={handleNext}
                      className="flex items-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
                    >
                      <span>下一步</span>
                      <ChevronRight className="w-4 h-4" />
                    </button>
                  ) : (
                    <button className="px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium">
                      提交审核
                    </button>
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
