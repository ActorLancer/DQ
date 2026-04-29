'use client'

import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { Search, User, Menu, LogOut, ChevronDown } from 'lucide-react'
import { useAuthStore } from '@/store/useAuthStore'
import { useAdminAuthStore } from '@/store/useAdminAuthStore'
import { useState } from 'react'

export default function Header() {
  const router = useRouter()
  const { isAuthenticated, user, logout, setCurrentRole } = useAuthStore()
  const {
    isAuthenticated: isAdminAuthenticated,
    user: adminUser,
    logout: adminLogout,
  } = useAdminAuthStore()
  const [showUserMenu, setShowUserMenu] = useState(false)

  const handleTradeLogout = () => {
    logout()
    router.push('/')
  }

  const handleAdminLogout = () => {
    adminLogout()
    router.push('/')
  }

  const handleSwitchRole = (role: 'buyer' | 'seller') => {
    setCurrentRole(role)
    router.push(`/console/${role}`)
    setShowUserMenu(false)
  }

  return (
    <header className="sticky top-0 z-50 bg-white border-b border-gray-200">
      <div className="w-full px-1.5 sm:px-2 lg:px-3">
        <div className="flex items-center h-[72px]">
          <div className="flex flex-1 items-center justify-start min-w-0">
            <Link href="/" className="flex items-center space-x-3">
              <div className="w-9 h-9 bg-primary-600 rounded-lg flex items-center justify-center">
                <span className="text-white font-bold text-xl">D</span>
              </div>
              <span className="text-[23px] font-bold text-primary-900 tracking-tight">数据交易平台</span>
            </Link>
          </div>

          <nav className="hidden xl:flex items-center gap-9 px-6">
            <Link href="/marketplace" className="text-[16px] text-gray-700 hover:text-primary-600 font-semibold transition-colors">数据市场</Link>
            <Link href="/suppliers" className="text-[16px] text-gray-700 hover:text-primary-600 font-semibold transition-colors">优质供应商</Link>
            <Link href="/standard-flow" className="text-[16px] text-gray-700 hover:text-primary-600 font-semibold transition-colors">标准链路</Link>
            <Link href="/trust-center" className="text-[16px] text-gray-700 hover:text-primary-600 font-semibold transition-colors">可信能力</Link>
            <Link href="/docs" className="text-[16px] text-gray-700 hover:text-primary-600 font-semibold transition-colors">帮助中心</Link>
          </nav>

          <div className="flex flex-1 items-center justify-end space-x-4">
            <button className="p-2 text-gray-600 hover:text-primary-600 transition-colors">
              <Search className="w-5 h-5" />
            </button>

            {isAuthenticated && user ? (
              <div className="relative">
                <button
                  onClick={() => setShowUserMenu(!showUserMenu)}
                  className="flex items-center gap-2.5 px-4 py-2 text-gray-700 hover:bg-gray-50 rounded-lg transition-colors"
                >
                  <div className="w-9 h-9 bg-primary-100 rounded-full flex items-center justify-center">
                    <User className="w-4 h-4 text-primary-600" />
                  </div>
                  <span className="hidden md:inline text-[16px] font-semibold">{user.name}</span>
                  <ChevronDown className="w-4 h-4" />
                </button>

                {showUserMenu && (
                  <>
                    <div className="fixed inset-0 z-10" onClick={() => setShowUserMenu(false)} />
                    <div className="absolute right-0 mt-2 w-80 bg-white rounded-xl shadow-xl border border-gray-200 py-2 z-20">
                      <div className="px-5 py-4 border-b border-gray-100">
                        <p className="text-base font-semibold text-gray-900">{user.name}</p>
                        <p className="text-sm text-gray-500 mt-0.5">{user.email}</p>
                      </div>

                      <div className="px-5 py-3">
                        <p className="text-sm text-gray-500 mb-2.5">控制台入口</p>
                        <div className="flex gap-2.5">
                          {user.roles.includes('buyer') && (
                            <button
                              onClick={() => handleSwitchRole('buyer')}
                              className="px-3.5 py-2 text-sm rounded-lg border border-primary-200 text-primary-700 hover:bg-primary-50"
                            >
                              买家控制台
                            </button>
                          )}
                          {user.roles.includes('seller') && (
                            <button
                              onClick={() => handleSwitchRole('seller')}
                              className="px-3.5 py-2 text-sm rounded-lg border border-green-200 text-green-700 hover:bg-green-50"
                            >
                              供应商后台
                            </button>
                          )}
                        </div>
                      </div>

                      <button
                        onClick={handleTradeLogout}
                        className="w-full flex items-center gap-2 px-5 py-3 text-sm text-red-600 hover:bg-red-50"
                      >
                        <LogOut className="w-4 h-4" />
                        <span>退出交易账号</span>
                      </button>
                    </div>
                  </>
                )}
              </div>
            ) : (
              <>
                <Link href="/login" className="hidden md:inline-flex items-center gap-2 px-4 py-2 text-[15px] text-gray-700 hover:text-primary-600 font-semibold transition-colors">
                  <User className="w-4 h-4" />
                  <span>登录</span>
                </Link>
                <Link href="/login" className="btn-primary">开始使用</Link>
              </>
            )}

            {isAdminAuthenticated && adminUser && (
              <div className="hidden md:flex items-center gap-2 pl-3 border-l border-gray-200">
                <Link href="/admin/console" className="px-3 py-2 text-sm rounded-lg border border-gray-300 hover:bg-gray-50">
                  管理端控制台
                </Link>
                <button onClick={handleAdminLogout} className="px-3 py-2 text-sm text-red-600 rounded-lg hover:bg-red-50">
                  退出管理端
                </button>
              </div>
            )}

            <button className="xl:hidden p-2 text-gray-600">
              <Menu className="w-6 h-6" />
            </button>
          </div>
        </div>
      </div>
    </header>
  )
}
