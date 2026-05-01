'use client'

import { FormEvent, useState } from 'react'
import Link from 'next/link'
import SessionIdentityBar from '@/components/console/SessionIdentityBar'
import {
  AlertTriangle,
  Bell,
  CheckCircle2,
  Globe,
  Save,
  Shield,
  SlidersHorizontal,
  RotateCcw,
} from 'lucide-react'

type SaveState = 'idle' | 'saving' | 'success'

export default function AdminSettingsPage() {
  const sessionExpiresAt = new Date(Date.now() + 30 * 60 * 1000).toISOString()

  const [emailNotify, setEmailNotify] = useState(true)
  const [smsNotify, setSmsNotify] = useState(true)
  const [inAppNotify, setInAppNotify] = useState(true)
  const [riskThreshold, setRiskThreshold] = useState('medium')
  const [auditRetentionDays, setAuditRetentionDays] = useState('180')
  const [requireMfa, setRequireMfa] = useState(true)
  const [defaultLocale, setDefaultLocale] = useState('zh-CN')
  const [timezone, setTimezone] = useState('Asia/Shanghai')
  const [saveState, setSaveState] = useState<SaveState>('idle')

  const handleSave = async (e: FormEvent) => {
    e.preventDefault()
    setSaveState('saving')
    await new Promise((resolve) => setTimeout(resolve, 700))
    setSaveState('success')
    window.setTimeout(() => setSaveState('idle'), 1800)
  }

  const handleReset = () => {
    const ok = window.confirm('确认恢复平台设置默认值？此操作仅影响前端模拟配置。')
    if (!ok) return
    setEmailNotify(true)
    setSmsNotify(true)
    setInAppNotify(true)
    setRiskThreshold('medium')
    setAuditRetentionDays('180')
    setRequireMfa(true)
    setDefaultLocale('zh-CN')
    setTimezone('Asia/Shanghai')
  }

  return (
    <>
      <SessionIdentityBar
        subjectName="数据交易平台运营中心"
        roleName="平台管理员"
        tenantId="tenant_platform_001"
        scope="admin:settings:write"
        sessionExpiresAt={sessionExpiresAt}
        userName="管理员"
      />

      <div className="p-8">
        <div className="mb-8 flex items-center justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 mb-2">平台设置</h1>
            <p className="text-gray-600">统一配置通知、风控阈值与审计留存策略，作用于 admin 控制台与平台运营流程。</p>
          </div>
          <Link href="/admin/console/notifications/rules" className="h-10 px-4 border border-gray-300 rounded-lg hover:bg-gray-50 text-sm font-medium inline-flex items-center gap-2">
            <Bell className="w-4 h-4" />
            通知规则
          </Link>
        </div>

        <form onSubmit={handleSave} className="space-y-6">
          <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
            <section className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center gap-2 mb-4">
                <Bell className="w-5 h-5 text-gray-600" />
                <h2 className="text-lg font-bold">通知与告警</h2>
              </div>
              <div className="space-y-4">
                <label className="flex items-center justify-between gap-4">
                  <span className="text-sm text-gray-700">邮件通知（审核、风控、链异常）</span>
                  <input type="checkbox" checked={emailNotify} onChange={(e) => setEmailNotify(e.target.checked)} />
                </label>
                <label className="flex items-center justify-between gap-4">
                  <span className="text-sm text-gray-700">短信通知（高优先级告警）</span>
                  <input type="checkbox" checked={smsNotify} onChange={(e) => setSmsNotify(e.target.checked)} />
                </label>
                <label className="flex items-center justify-between gap-4">
                  <span className="text-sm text-gray-700">站内通知（事件通知中心）</span>
                  <input type="checkbox" checked={inAppNotify} onChange={(e) => setInAppNotify(e.target.checked)} />
                </label>
              </div>
            </section>

            <section className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center gap-2 mb-4">
                <Shield className="w-5 h-5 text-gray-600" />
                <h2 className="text-lg font-bold">风控与审计</h2>
              </div>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-700 mb-2">默认风控阈值</label>
                  <select className="input" value={riskThreshold} onChange={(e) => setRiskThreshold(e.target.value)}>
                    <option value="low">低风险优先放行</option>
                    <option value="medium">中风险人工复核</option>
                    <option value="high">高风险强制拦截</option>
                  </select>
                </div>
                <div>
                  <label className="block text-sm text-gray-700 mb-2">审计日志留存天数</label>
                  <select className="input" value={auditRetentionDays} onChange={(e) => setAuditRetentionDays(e.target.value)}>
                    <option value="90">90 天</option>
                    <option value="180">180 天</option>
                    <option value="365">365 天</option>
                  </select>
                </div>
                <label className="flex items-center justify-between gap-4">
                  <span className="text-sm text-gray-700">高危操作强制 MFA</span>
                  <input type="checkbox" checked={requireMfa} onChange={(e) => setRequireMfa(e.target.checked)} />
                </label>
              </div>
            </section>

            <section className="bg-white rounded-xl border border-gray-200 p-6">
              <div className="flex items-center gap-2 mb-4">
                <Globe className="w-5 h-5 text-gray-600" />
                <h2 className="text-lg font-bold">控制台偏好</h2>
              </div>
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
                <button type="button" onClick={handleReset} className="h-10 px-4 border border-red-300 text-red-700 rounded-lg hover:bg-red-50 text-sm font-medium inline-flex items-center gap-2">
                  <RotateCcw className="w-4 h-4" />
                  恢复默认配置
                </button>
              </div>
            </section>
          </div>

          <div className="bg-white rounded-xl border border-gray-200 p-4 flex items-center justify-between">
            <div className="text-sm text-gray-600 inline-flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 text-amber-500" />
              平台配置保存后即时生效，建议先在低峰时段验证通知链路。
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
          <span>平台设置已保存</span>
        </div>
      )}
    </>
  )
}
