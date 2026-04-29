'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { 
  User,
  Building2,
  Bell,
  Shield,
  Key,
  CreditCard,
  FileText,
  Mail,
  Phone,
  MapPin,
  Save,
  CheckCircle,
  AlertCircle
} from 'lucide-react'

export default function BuyerSettingsPage() {
  const [activeTab, setActiveTab] = useState<'profile' | 'company' | 'notifications' | 'security' | 'billing'>('profile')
  const [showSaveSuccess, setShowSaveSuccess] = useState(false)
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const handleSave = () => {
    setShowSaveSuccess(true)
    setTimeout(() => setShowSaveSuccess(false), 3000)
  }

  const tabs = [
    { id: 'profile', label: '个人信息', icon: User },
    { id: 'company', label: '企业信息', icon: Building2 },
    { id: 'notifications', label: '通知设置', icon: Bell },
    { id: 'security', label: '安全设置', icon: Shield },
    { id: 'billing', label: '账单设置', icon: CreditCard },
  ]

  return (
    <>
      <SessionIdentityBar
        subjectName="某某科技有限公司"
        roleName="买家管理员"
        tenantId="tenant_buyer_001"
        scope="buyer:settings:write"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        {/* 页面标题 */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">设置</h1>
          <p className="text-gray-600">管理您的账户和偏好设置</p>
        </div>

        {/* 保存成功提示 */}
        {showSaveSuccess && (
          <div className="fixed top-24 right-8 z-50 animate-fade-in">
            <div className="bg-green-50 border border-green-200 rounded-lg p-4 shadow-lg">
              <div className="flex items-center gap-3">
                <CheckCircle className="w-5 h-5 text-green-600" />
                <span className="text-sm font-medium text-green-800">设置已保存</span>
              </div>
            </div>
          </div>
        )}

        <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
          {/* 左侧导航 */}
          <div className="lg:col-span-1">
            <div className="bg-white rounded-xl border border-gray-200 p-4 sticky top-28">
              <nav className="space-y-1">
                {tabs.map((tab) => {
                  const Icon = tab.icon
                  const isActive = activeTab === tab.id
                  
                  return (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id as any)}
                      className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg text-left transition-colors ${
                        isActive
                          ? 'bg-primary-50 text-primary-700 font-medium'
                          : 'text-gray-700 hover:bg-gray-50'
                      }`}
                    >
                      <Icon className="w-5 h-5" />
                      <span>{tab.label}</span>
                    </button>
                  )
                })}
              </nav>
            </div>
          </div>

          {/* 右侧内容 */}
          <div className="lg:col-span-3">
            <div className="bg-white rounded-xl border border-gray-200 p-8">
              {/* 个人信息 */}
              {activeTab === 'profile' && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 mb-6">个人信息</h2>
                  
                  <div className="space-y-6">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          姓名 <span className="text-red-500">*</span>
                        </label>
                        <input
                          type="text"
                          defaultValue="李四"
                          className="input"
                        />
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          职位
                        </label>
                        <input
                          type="text"
                          defaultValue="数据采购经理"
                          className="input"
                        />
                      </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          邮箱 <span className="text-red-500">*</span>
                        </label>
                        <div className="relative">
                          <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
                          <input
                            type="email"
                            defaultValue="lisi@example.com"
                            className="input pl-10"
                          />
                        </div>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          手机号 <span className="text-red-500">*</span>
                        </label>
                        <div className="relative">
                          <Phone className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
                          <input
                            type="tel"
                            defaultValue="13800138000"
                            className="input pl-10"
                          />
                        </div>
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        头像
                      </label>
                      <div className="flex items-center gap-4">
                        <div className="w-20 h-20 bg-primary-100 rounded-full flex items-center justify-center">
                          <User className="w-10 h-10 text-primary-600" />
                        </div>
                        <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 text-sm font-medium">
                          上传头像
                        </button>
                      </div>
                    </div>
                  </div>

                  <div className="flex justify-end gap-3 mt-8 pt-6 border-t border-gray-200">
                    <button className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                      取消
                    </button>
                    <button
                      onClick={handleSave}
                      className="flex items-center gap-2 px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
                    >
                      <Save className="w-4 h-4" />
                      <span>保存</span>
                    </button>
                  </div>
                </div>
              )}

              {/* 企业信息 */}
              {activeTab === 'company' && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 mb-6">企业信息</h2>
                  
                  <div className="space-y-6">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        企业名称 <span className="text-red-500">*</span>
                      </label>
                      <input
                        type="text"
                        defaultValue="某某科技有限公司"
                        className="input"
                      />
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          统一社会信用代码 <span className="text-red-500">*</span>
                        </label>
                        <input
                          type="text"
                          defaultValue="91110000XXXXXXXXXX"
                          className="input"
                        />
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          企业类型
                        </label>
                        <select className="input">
                          <option>有限责任公司</option>
                          <option>股份有限公司</option>
                          <option>个人独资企业</option>
                          <option>合伙企业</option>
                        </select>
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        注册地址
                      </label>
                      <div className="relative">
                        <MapPin className="absolute left-3 top-3 w-5 h-5 text-gray-400" />
                        <textarea
                          defaultValue="北京市朝阳区某某街道某某大厦 1001 室"
                          className="input pl-10 min-h-[80px]"
                        />
                      </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          法定代表人
                        </label>
                        <input
                          type="text"
                          defaultValue="张三"
                          className="input"
                        />
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-2">
                          注册资本（万元）
                        </label>
                        <input
                          type="text"
                          defaultValue="1000"
                          className="input"
                        />
                      </div>
                    </div>

                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-2">
                        营业执照
                      </label>
                      <div className="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center hover:border-primary-400 transition-colors cursor-pointer">
                        <FileText className="w-12 h-12 mx-auto text-gray-400 mb-2" />
                        <p className="text-sm text-gray-600 mb-1">点击上传或拖拽文件到此处</p>
                        <p className="text-xs text-gray-500">支持 PDF、JPG、PNG 格式，最大 10MB</p>
                      </div>
                    </div>
                  </div>

                  <div className="flex justify-end gap-3 mt-8 pt-6 border-t border-gray-200">
                    <button className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                      取消
                    </button>
                    <button
                      onClick={handleSave}
                      className="flex items-center gap-2 px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
                    >
                      <Save className="w-4 h-4" />
                      <span>保存</span>
                    </button>
                  </div>
                </div>
              )}

              {/* 通知设置 */}
              {activeTab === 'notifications' && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 mb-6">通知设置</h2>
                  
                  <div className="space-y-6">
                    <div>
                      <h3 className="font-medium text-gray-900 mb-4">邮件通知</h3>
                      <div className="space-y-3">
                        {[
                          { id: 'email_order', label: '订单状态更新', defaultChecked: true },
                          { id: 'email_request', label: '申请审批结果', defaultChecked: true },
                          { id: 'email_quota', label: '配额使用警告', defaultChecked: true },
                          { id: 'email_expiry', label: '订阅到期提醒', defaultChecked: true },
                          { id: 'email_invoice', label: '发票开具通知', defaultChecked: false },
                          { id: 'email_marketing', label: '产品推荐和营销', defaultChecked: false },
                        ].map((item) => (
                          <label key={item.id} className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 cursor-pointer">
                            <input
                              type="checkbox"
                              defaultChecked={item.defaultChecked}
                              className="w-4 h-4 text-primary-600 rounded focus:ring-2 focus:ring-primary-500"
                            />
                            <span className="text-sm text-gray-900">{item.label}</span>
                          </label>
                        ))}
                      </div>
                    </div>

                    <div className="pt-6 border-t border-gray-200">
                      <h3 className="font-medium text-gray-900 mb-4">短信通知</h3>
                      <div className="space-y-3">
                        {[
                          { id: 'sms_security', label: '安全验证码', defaultChecked: true },
                          { id: 'sms_urgent', label: '紧急通知', defaultChecked: true },
                          { id: 'sms_quota', label: '配额耗尽警告', defaultChecked: true },
                        ].map((item) => (
                          <label key={item.id} className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 cursor-pointer">
                            <input
                              type="checkbox"
                              defaultChecked={item.defaultChecked}
                              className="w-4 h-4 text-primary-600 rounded focus:ring-2 focus:ring-primary-500"
                            />
                            <span className="text-sm text-gray-900">{item.label}</span>
                          </label>
                        ))}
                      </div>
                    </div>

                    <div className="pt-6 border-t border-gray-200">
                      <h3 className="font-medium text-gray-900 mb-4">站内通知</h3>
                      <div className="space-y-3">
                        {[
                          { id: 'web_all', label: '接收所有站内通知', defaultChecked: true },
                        ].map((item) => (
                          <label key={item.id} className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 cursor-pointer">
                            <input
                              type="checkbox"
                              defaultChecked={item.defaultChecked}
                              className="w-4 h-4 text-primary-600 rounded focus:ring-2 focus:ring-primary-500"
                            />
                            <span className="text-sm text-gray-900">{item.label}</span>
                          </label>
                        ))}
                      </div>
                    </div>
                  </div>

                  <div className="flex justify-end gap-3 mt-8 pt-6 border-t border-gray-200">
                    <button className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                      取消
                    </button>
                    <button
                      onClick={handleSave}
                      className="flex items-center gap-2 px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
                    >
                      <Save className="w-4 h-4" />
                      <span>保存</span>
                    </button>
                  </div>
                </div>
              )}

              {/* 安全设置 */}
              {activeTab === 'security' && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 mb-6">安全设置</h2>
                  
                  <div className="space-y-6">
                    <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
                      <div className="flex gap-3">
                        <Shield className="w-5 h-5 text-blue-600 flex-shrink-0 mt-0.5" />
                        <div>
                          <h3 className="font-medium text-blue-900 mb-1">账户安全提示</h3>
                          <p className="text-sm text-blue-800">
                            为了保护您的账户安全，建议定期更新密码，并启用双因素认证。
                          </p>
                        </div>
                      </div>
                    </div>

                    <div>
                      <h3 className="font-medium text-gray-900 mb-4">登录密码</h3>
                      <div className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
                        <div>
                          <p className="text-sm text-gray-900 mb-1">当前密码强度: <span className="font-medium text-green-600">强</span></p>
                          <p className="text-xs text-gray-500">上次修改: 2026-03-15</p>
                        </div>
                        <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 text-sm font-medium">
                          修改密码
                        </button>
                      </div>
                    </div>

                    <div>
                      <h3 className="font-medium text-gray-900 mb-4">双因素认证</h3>
                      <div className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
                        <div>
                          <p className="text-sm text-gray-900 mb-1">状态: <span className="font-medium text-green-600">已启用</span></p>
                          <p className="text-xs text-gray-500">通过手机短信验证码进行二次验证</p>
                        </div>
                        <button className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 text-sm font-medium">
                          管理
                        </button>
                      </div>
                    </div>

                    <div>
                      <h3 className="font-medium text-gray-900 mb-4">登录设备</h3>
                      <div className="space-y-3">
                        {[
                          { device: 'Chrome on Windows', location: '北京市', time: '当前设备', active: true },
                          { device: 'Safari on macOS', location: '上海市', time: '2 天前', active: false },
                        ].map((item, index) => (
                          <div key={index} className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
                            <div>
                              <p className="text-sm font-medium text-gray-900">{item.device}</p>
                              <p className="text-xs text-gray-500">{item.location} · {item.time}</p>
                            </div>
                            {item.active ? (
                              <span className="status-tag bg-green-100 text-green-800 text-xs">
                                <CheckCircle className="w-3 h-3" />
                                <span>当前</span>
                              </span>
                            ) : (
                              <button className="text-sm text-red-600 hover:text-red-700 font-medium">
                                移除
                              </button>
                            )}
                          </div>
                        ))}
                      </div>
                    </div>

                    <div>
                      <h3 className="font-medium text-gray-900 mb-4">API 访问控制</h3>
                      <div className="space-y-3">
                        <label className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 cursor-pointer">
                          <input
                            type="checkbox"
                            defaultChecked={true}
                            className="w-4 h-4 text-primary-600 rounded focus:ring-2 focus:ring-primary-500"
                          />
                          <div>
                            <p className="text-sm text-gray-900">启用 IP 白名单</p>
                            <p className="text-xs text-gray-500">仅允许白名单内的 IP 地址访问 API</p>
                          </div>
                        </label>
                        <label className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 cursor-pointer">
                          <input
                            type="checkbox"
                            defaultChecked={true}
                            className="w-4 h-4 text-primary-600 rounded focus:ring-2 focus:ring-primary-500"
                          />
                          <div>
                            <p className="text-sm text-gray-900">启用请求签名验证</p>
                            <p className="text-xs text-gray-500">要求所有 API 请求包含有效签名</p>
                          </div>
                        </label>
                      </div>
                    </div>
                  </div>

                  <div className="flex justify-end gap-3 mt-8 pt-6 border-t border-gray-200">
                    <button className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                      取消
                    </button>
                    <button
                      onClick={handleSave}
                      className="flex items-center gap-2 px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
                    >
                      <Save className="w-4 h-4" />
                      <span>保存</span>
                    </button>
                  </div>
                </div>
              )}

              {/* 账单设置 */}
              {activeTab === 'billing' && (
                <div>
                  <h2 className="text-2xl font-bold text-gray-900 mb-6">账单设置</h2>
                  
                  <div className="space-y-6">
                    <div>
                      <h3 className="font-medium text-gray-900 mb-4">发票信息</h3>
                      <div className="space-y-4">
                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-2">
                            发票抬头 <span className="text-red-500">*</span>
                          </label>
                          <input
                            type="text"
                            defaultValue="某某科技有限公司"
                            className="input"
                          />
                        </div>

                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-2">
                            纳税人识别号 <span className="text-red-500">*</span>
                          </label>
                          <input
                            type="text"
                            defaultValue="91110000XXXXXXXXXX"
                            className="input"
                          />
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                          <div>
                            <label className="block text-sm font-medium text-gray-700 mb-2">
                              开户银行
                            </label>
                            <input
                              type="text"
                              defaultValue="中国工商银行北京分行"
                              className="input"
                            />
                          </div>

                          <div>
                            <label className="block text-sm font-medium text-gray-700 mb-2">
                              银行账号
                            </label>
                            <input
                              type="text"
                              defaultValue="1234 5678 9012 3456"
                              className="input"
                            />
                          </div>
                        </div>

                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-2">
                            注册地址
                          </label>
                          <input
                            type="text"
                            defaultValue="北京市朝阳区某某街道某某大厦 1001 室"
                            className="input"
                          />
                        </div>

                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-2">
                            注册电话
                          </label>
                          <input
                            type="tel"
                            defaultValue="010-12345678"
                            className="input"
                          />
                        </div>
                      </div>
                    </div>

                    <div className="pt-6 border-t border-gray-200">
                      <h3 className="font-medium text-gray-900 mb-4">收件信息</h3>
                      <div className="space-y-4">
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                          <div>
                            <label className="block text-sm font-medium text-gray-700 mb-2">
                              收件人 <span className="text-red-500">*</span>
                            </label>
                            <input
                              type="text"
                              defaultValue="李四"
                              className="input"
                            />
                          </div>

                          <div>
                            <label className="block text-sm font-medium text-gray-700 mb-2">
                              联系电话 <span className="text-red-500">*</span>
                            </label>
                            <input
                              type="tel"
                              defaultValue="13800138000"
                              className="input"
                            />
                          </div>
                        </div>

                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-2">
                            收件地址 <span className="text-red-500">*</span>
                          </label>
                          <textarea
                            defaultValue="北京市朝阳区某某街道某某大厦 1001 室"
                            className="input min-h-[80px]"
                          />
                        </div>

                        <div>
                          <label className="block text-sm font-medium text-gray-700 mb-2">
                            收件邮箱
                          </label>
                          <input
                            type="email"
                            defaultValue="lisi@example.com"
                            placeholder="用于接收电子发票"
                            className="input"
                          />
                        </div>
                      </div>
                    </div>

                    <div className="pt-6 border-t border-gray-200">
                      <h3 className="font-medium text-gray-900 mb-4">自动开票</h3>
                      <label className="flex items-center gap-3 p-3 rounded-lg hover:bg-gray-50 cursor-pointer">
                        <input
                          type="checkbox"
                          defaultChecked={false}
                          className="w-4 h-4 text-primary-600 rounded focus:ring-2 focus:ring-primary-500"
                        />
                        <div>
                          <p className="text-sm text-gray-900">启用自动开票</p>
                          <p className="text-xs text-gray-500">订单支付成功后自动申请开具发票</p>
                        </div>
                      </label>
                    </div>
                  </div>

                  <div className="flex justify-end gap-3 mt-8 pt-6 border-t border-gray-200">
                    <button className="px-6 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium">
                      取消
                    </button>
                    <button
                      onClick={handleSave}
                      className="flex items-center gap-2 px-6 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
                    >
                      <Save className="w-4 h-4" />
                      <span>保存</span>
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
