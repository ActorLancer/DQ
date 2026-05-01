'use client'

import { FormEvent, useMemo, useState } from 'react'
import Link from 'next/link'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import {
  Bell,
  Shield,
  Globe,
  Save,
  CheckCircle2,
  AlertTriangle,
  RotateCcw,
  ExternalLink,
  KeyRound,
  Workflow,
} from 'lucide-react'

type SaveState = 'idle' | 'saving' | 'success'

export default function SellerSettingsPage() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const [notifyEmail, setNotifyEmail] = useState(true)
  const [notifySms, setNotifySms] = useState(false)
  const [notifyInApp, setNotifyInApp] = useState(true)
  const [approvalSlaHours, setApprovalSlaHours] = useState('24')
  const [apiWhitelist, setApiWhitelist] = useState('192.168.1.100\n10.0.0.50')
  const [require2fa, setRequire2fa] = useState(true)
  const [defaultLocale, setDefaultLocale] = useState('zh-CN')
  const [timezone, setTimezone] = useState('Asia/Shanghai')
  const [saveState, setSaveState] = useState<SaveState>('idle')

  const whitelistCount = useMemo(
    () => apiWhitelist.split('\n').map((ip) => ip.trim()).filter(Boolean).length,
    [apiWhitelist]
  )

  const handleSave = async (e: FormEvent) => {
    e.preventDefault()
    setSaveState('saving')
    await new Promise((resolve) => setTimeout(resolve, 700))
    setSaveState('success')
    window.setTimeout(() => setSaveState('idle'), 1800)
  }

  const resetSecurity = () => {
    const ok = window.confirm('确认恢复默认安全策略？将重置 IP 白名单与二次验证要求。')
    if (!ok) return
    setApiWhitelist('')
    setRequire2fa(true)
  }

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
        <div className="mb-8 flex items-center justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">设置</h1>
            <p className="text-gray-600">配置通知、安全策略、审批时效与控制台偏好，变更将作用于 seller 全端。</p>
          </div>
          <div className="flex items-center gap-2">
            <Link href="/console/seller/notifications/rules" className="h-10 px-4 border border-gray-300 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center gap-2">
              <Bell className="w-4 h-4" />
              通知规则
            </Link>
            <Link href="/console/seller/requests" className="h-10 px-4 border border-gray-300 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center gap-2">
              <Workflow className="w-4 h-4" />
              申请审批
            </Link>
          </div>
        </div>

        <form onSubmit={handleSave} className="space-y-6">
          <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center gap-2 mb-4"><Bell className="w-5 h-5 text-gray-600" /><h2 className="text-lg font-bold">通知与审批时效</h2></div>
              <div className="space-y-4">
                <label className="flex items-center justify-between gap-4">
                  <span className="text-sm text-gray-700">邮件通知（申请审批、交付异常）</span>
                  <input type="checkbox" checked={notifyEmail} onChange={(e) => setNotifyEmail(e.target.checked)} />
                </label>
                <label className="flex items-center justify-between gap-4">
                  <span className="text-sm text-gray-700">短信通知（高危告警）</span>
                  <input type="checkbox" checked={notifySms} onChange={(e) => setNotifySms(e.target.checked)} />
                </label>
                <label className="flex items-center justify-between gap-4">
                  <span className="text-sm text-gray-700">站内通知（事件中心）</span>
                  <input type="checkbox" checked={notifyInApp} onChange={(e) => setNotifyInApp(e.target.checked)} />
                </label>
                <div>
                  <label className="block text-sm text-gray-700 mb-2">默认审批 SLA（小时）</label>
                  <select className="input" value={approvalSlaHours} onChange={(e) => setApprovalSlaHours(e.target.value)}>
                    <option value="12">12 小时</option>
                    <option value="24">24 小时</option>
                    <option value="48">48 小时</option>
                    <option value="72">72 小时</option>
                  </select>
                </div>
              </div>
            </section>

            <section className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center gap-2 mb-4"><Shield className="w-5 h-5 text-gray-600" /><h2 className="text-lg font-bold">安全策略</h2></div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-700 mb-2">后台 IP 白名单</label>
                  <textarea className="input min-h-[132px]" value={apiWhitelist} onChange={(e) => setApiWhitelist(e.target.value)} />
                  <p className="text-xs text-gray-500 mt-2">每行一个 IP，当前共 {whitelistCount} 条。留空表示仅依赖账户权限控制。</p>
                </div>
                <label className="flex items-center justify-between gap-4">
                  <span className="text-sm text-gray-700 inline-flex items-center gap-2"><KeyRound className="w-4 h-4" />高危操作要求二次验证</span>
                  <input type="checkbox" checked={require2fa} onChange={(e) => setRequire2fa(e.target.checked)} />
                </label>
                <button type="button" onClick={resetSecurity} className="h-10 px-4 border border-red-300 text-red-700 rounded-lg hover:bg-red-50 text-sm font-medium inline-flex items-center gap-2">
                  <RotateCcw className="w-4 h-4" />
                  恢复默认安全策略
                </button>
              </div>
            </section>

            <section className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center gap-2 mb-4"><Globe className="w-5 h-5 text-gray-600" /><h2 className="text-lg font-bold">界面偏好</h2></div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-700 mb-2">默认语言</label>
                  <select className="input" value={defaultLocale} onChange={(e) => setDefaultLocale(e.target.value)}>
                    <option value="zh-CN">简体中文</option>
                    <option value="en-US">English</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm text-gray-700 mb-2">时区</label>
                  <select className="input" value={timezone} onChange={(e) => setTimezone(e.target.value)}>
                    <option value="Asia/Shanghai">Asia/Shanghai</option>
                    <option value="UTC">UTC</option>
                    <option value="America/Los_Angeles">America/Los_Angeles</option>
                  </select>
                </div>
                <div className="rounded-xl border border-gray-200 bg-gray-50 p-4 text-xs text-gray-600 space-y-1">
                  <p>联动入口：</p>
                  <Link href="/console/seller/notifications" className="inline-flex items-center gap-1 text-primary-600 hover:text-primary-700">事件通知中心 <ExternalLink className="w-3 h-3" /></Link>
                  <br />
                  <Link href="/console/seller/analytics" className="inline-flex items-center gap-1 text-primary-600 hover:text-primary-700">调用看板 <ExternalLink className="w-3 h-3" /></Link>
                </div>
              </div>
            </section>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-4 flex items-center justify-between">
            <div className="text-sm text-gray-600 inline-flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 text-amber-500" />
              设置保存后即时生效，建议在低峰时段调整安全策略。
            </div>
            <button type="submit" disabled={saveState === 'saving'} className="h-10 px-6 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50 inline-flex items-center gap-2 text-sm font-medium">
              <Save className="w-4 h-4" />
              {saveState === 'saving' ? '保存中...' : saveState === 'success' ? '保存成功' : '保存设置'}
            </button>
          </div>
        </form>
      </div>

      {saveState === 'success' && (
        <div className="fixed right-6 top-24 z-50 rounded-lg border border-green-200 bg-green-50 px-4 py-3 text-sm text-green-800 shadow-lg inline-flex items-center gap-2">
          <CheckCircle2 className="w-4 h-4" />
          <span>设置已保存并生效</span>
        </div>
      )}
    </>
  )
}
