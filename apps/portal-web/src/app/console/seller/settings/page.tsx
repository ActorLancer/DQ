'use client'

import { useState } from 'react'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import { Bell, Shield, Globe, Save } from 'lucide-react'

export default function SellerSettingsPage() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()
  const [notifyEmail, setNotifyEmail] = useState(true)
  const [notifySms, setNotifySms] = useState(false)
  const [apiWhitelist, setApiWhitelist] = useState('192.168.1.100\n10.0.0.50')
  const [defaultLocale, setDefaultLocale] = useState('zh-CN')

  return (
    <>
      <SessionIdentityBar
        subjectName="天眼数据科技有限公司"
        roleName="供应商管理员"
        tenantId="tenant_supplier_001"
        scope="seller:settings:write"
        sessionExpiresAt={sessionExpiresAt}
      />

      <div className="p-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">设置</h1>
          <p className="text-gray-600">配置通知、安全策略和后台偏好</p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center gap-2 mb-4"><Bell className="w-5 h-5 text-gray-600" /><h2 className="text-lg font-bold">通知设置</h2></div>
            <div className="space-y-4">
              <label className="flex items-center justify-between">
                <span className="text-sm text-gray-700">邮件通知（申请审批、交付异常）</span>
                <input type="checkbox" checked={notifyEmail} onChange={(e) => setNotifyEmail(e.target.checked)} />
              </label>
              <label className="flex items-center justify-between">
                <span className="text-sm text-gray-700">短信通知（高危告警）</span>
                <input type="checkbox" checked={notifySms} onChange={(e) => setNotifySms(e.target.checked)} />
              </label>
            </div>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center gap-2 mb-4"><Shield className="w-5 h-5 text-gray-600" /><h2 className="text-lg font-bold">安全设置</h2></div>
            <label className="block text-sm text-gray-700 mb-2">后台 IP 白名单</label>
            <textarea className="input min-h-[120px]" value={apiWhitelist} onChange={(e) => setApiWhitelist(e.target.value)} />
            <p className="text-xs text-gray-500 mt-2">每行一个 IP，留空表示仅依赖账户权限控制。</p>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-6">
            <div className="flex items-center gap-2 mb-4"><Globe className="w-5 h-5 text-gray-600" /><h2 className="text-lg font-bold">偏好设置</h2></div>
            <label className="block text-sm text-gray-700 mb-2">默认语言</label>
            <select className="input" value={defaultLocale} onChange={(e) => setDefaultLocale(e.target.value)}>
              <option value="zh-CN">简体中文</option>
              <option value="en-US">English</option>
            </select>
          </div>
        </div>

        <div className="mt-6 flex justify-end">
          <button className="px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 inline-flex items-center gap-2">
            <Save className="w-4 h-4" />
            <span>保存设置</span>
          </button>
        </div>
      </div>
    </>
  )
}
