'use client'

import Link from 'next/link'
import { useRouter, useSearchParams } from 'next/navigation'
import { AlertTriangle, LogOut, ShieldCheck, ArrowLeftRight } from 'lucide-react'
import { useAuthStore } from '@/store/useAuthStore'

export default function RoleAccessPage() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const { user, logout, setCurrentRole } = useAuthStore()

  const targetRole = searchParams.get('targetRole') as 'buyer' | 'seller' | 'admin' | null
  const fallbackUrl = searchParams.get('fallbackUrl') || '/login'
  const returnUrl = searchParams.get('returnUrl') || '/'

  const targetRoleName =
    targetRole === 'buyer' ? '买家' : targetRole === 'seller' ? '供应商' : targetRole === 'admin' ? '管理员' : '目标角色'

  const canSwitch = Boolean(targetRole && user?.roles.includes(targetRole))

  const handleSwitchRole = () => {
    if (!targetRole || !canSwitch) return
    setCurrentRole(targetRole)
    router.push(decodeURIComponent(returnUrl))
  }

  const handleRelogin = () => {
    logout()
    router.push(`${fallbackUrl}?returnUrl=${returnUrl}`)
  }

  return (
    <div className="min-h-screen bg-[radial-gradient(circle_at_top_left,_#dbeafe,_transparent_40%),radial-gradient(circle_at_bottom_right,_#dcfce7,_transparent_35%),#f8fafc] flex items-center justify-center p-6">
      <div className="w-full max-w-2xl bg-white/90 backdrop-blur rounded-2xl border border-white shadow-[0_24px_80px_-24px_rgba(15,23,42,0.35)] overflow-hidden">
        <div className="p-8 border-b border-gray-100">
          <div className="w-12 h-12 rounded-xl bg-amber-100 text-amber-700 flex items-center justify-center mb-4">
            <AlertTriangle className="w-6 h-6" />
          </div>
          <h1 className="text-2xl font-bold text-gray-900">角色访问确认</h1>
          <p className="mt-2 text-gray-600">当前页面需要{targetRoleName}权限。请选择接下来的访问方式。</p>
        </div>

        <div className="p-8 space-y-4">
          {canSwitch ? (
            <button
              onClick={handleSwitchRole}
              className="w-full text-left p-5 rounded-xl border border-emerald-200 bg-emerald-50 hover:bg-emerald-100 transition-colors"
            >
              <div className="flex items-start gap-3">
                <ShieldCheck className="w-5 h-5 text-emerald-700 mt-0.5" />
                <div>
                  <p className="font-semibold text-emerald-900">继续使用当前账号并切换角色</p>
                  <p className="text-sm text-emerald-800 mt-1">检测到当前账号已具备{targetRoleName}角色，立即切换并继续访问。</p>
                </div>
              </div>
            </button>
          ) : (
            <div className="p-5 rounded-xl border border-amber-200 bg-amber-50">
              <p className="font-semibold text-amber-900">当前账号不具备 {targetRoleName} 角色</p>
              <p className="text-sm text-amber-800 mt-1">您可以退出当前账号后登录其他账号继续访问。</p>
            </div>
          )}

          <button
            onClick={handleRelogin}
            className="w-full text-left p-5 rounded-xl border border-slate-200 bg-slate-50 hover:bg-slate-100 transition-colors"
          >
            <div className="flex items-start gap-3">
              <LogOut className="w-5 h-5 text-slate-700 mt-0.5" />
              <div>
                <p className="font-semibold text-slate-900">退出当前账号并登录其他账号</p>
                <p className="text-sm text-slate-700 mt-1">将保留目标页面地址，登录后自动返回。</p>
              </div>
            </div>
          </button>

          <Link href="/" className="inline-flex items-center gap-2 text-sm text-gray-600 hover:text-gray-900">
            <ArrowLeftRight className="w-4 h-4" />
            返回门户首页
          </Link>
        </div>
      </div>
    </div>
  )
}
