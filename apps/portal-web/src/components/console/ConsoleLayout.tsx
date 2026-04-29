'use client'

import Link from 'next/link'
import { usePathname, useRouter } from 'next/navigation'
import { 
  LayoutDashboard, 
  Package, 
  FileText, 
  Users, 
  DollarSign, 
  BarChart3,
  FileCheck,
  Settings,
  LogOut,
  Menu,
  X
} from 'lucide-react'
import { useState } from 'react'
import { useAuthStore } from '@/store/useAuthStore'
import { useAdminAuthStore } from '@/store/useAdminAuthStore'

interface NavItem {
  id: string
  label: string
  icon: any
  href: string
  badge?: number
}

interface ConsoleLayoutProps {
  children: React.ReactNode
  role: 'buyer' | 'seller' | 'admin'
}

const SELLER_NAV: NavItem[] = [
  { id: 'dashboard', label: '仪表盘', icon: LayoutDashboard, href: '/console/seller' },
  { id: 'listings', label: '商品管理', icon: Package, href: '/console/seller/listings' },
  { id: 'requests', label: '申请审批', icon: FileText, href: '/console/seller/requests', badge: 3 },
  { id: 'customers', label: '订阅客户', icon: Users, href: '/console/seller/customers' },
  { id: 'revenue', label: '收入看板', icon: DollarSign, href: '/console/seller/revenue' },
  { id: 'analytics', label: '调用看板', icon: BarChart3, href: '/console/seller/analytics' },
  { id: 'contracts', label: '合同发票', icon: FileCheck, href: '/console/seller/contracts' },
  { id: 'settings', label: '设置', icon: Settings, href: '/console/seller/settings' },
]

const BUYER_NAV: NavItem[] = [
  { id: 'dashboard', label: '仪表盘', icon: LayoutDashboard, href: '/console/buyer' },
  { id: 'subscriptions', label: '我的订阅', icon: Package, href: '/console/buyer/subscriptions' },
  { id: 'requests', label: '我的申请', icon: FileText, href: '/console/buyer/requests' },
  { id: 'orders', label: '订单账单', icon: DollarSign, href: '/console/buyer/orders' },
  { id: 'api-keys', label: 'API 密钥', icon: FileCheck, href: '/console/buyer/api-keys' },
  { id: 'usage', label: '使用分析', icon: BarChart3, href: '/console/buyer/usage' },
  { id: 'settings', label: '设置', icon: Settings, href: '/console/buyer/settings' },
]

const ADMIN_NAV: NavItem[] = [
  { id: 'dashboard', label: '仪表盘', icon: LayoutDashboard, href: '/admin/console' },
  { id: 'subjects', label: '主体审核', icon: Users, href: '/admin/console/subjects', badge: 5 },
  { id: 'listings', label: '商品审核', icon: Package, href: '/admin/console/listings', badge: 8 },
  { id: 'audit', label: '风险审计', icon: FileCheck, href: '/admin/console/audit' },
  { id: 'consistency', label: '一致性检查', icon: BarChart3, href: '/admin/console/consistency' },
  { id: 'settings', label: '设置', icon: Settings, href: '/admin/console/settings' },
]

export default function ConsoleLayout({ children, role }: ConsoleLayoutProps) {
  const router = useRouter()
  const pathname = usePathname()
  const [isSidebarOpen, setIsSidebarOpen] = useState(true)
  const { logout: tradeLogout } = useAuthStore()
  const { logout: adminLogout } = useAdminAuthStore()

  const navItems = role === 'seller' ? SELLER_NAV : role === 'buyer' ? BUYER_NAV : ADMIN_NAV

  const handleLogout = () => {
    if (role === 'admin') {
      adminLogout()
      router.push('/admin/login')
      return
    }

    tradeLogout()
    router.push('/login')
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* 顶部导航 */}
      <header className="sticky top-0 z-40 bg-white border-b border-gray-200">
        <div className="flex items-center justify-between h-16 px-6">
          <div className="flex items-center gap-4">
            <button
              onClick={() => setIsSidebarOpen(!isSidebarOpen)}
              className="p-2 text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg"
            >
              {isSidebarOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
            </button>
            
            <Link href="/" className="flex items-center space-x-2">
              <div className="w-8 h-8 bg-primary-600 rounded-lg flex items-center justify-center">
                <span className="text-white font-bold text-lg">D</span>
              </div>
              <span className="text-xl font-bold text-primary-900">数据交易平台</span>
            </Link>

            <div className="ml-4 px-3 py-1 bg-primary-100 text-primary-700 rounded-full text-sm font-medium">
              {role === 'seller' ? '供应商后台' : role === 'buyer' ? '买家控制台' : '平台运营'}
            </div>
          </div>

          <div className="flex items-center gap-4">
            <Link href="/" className="text-sm text-gray-600 hover:text-gray-900">
              返回门户
            </Link>
            <button
              onClick={handleLogout}
              className="flex items-center gap-2 px-4 py-2 text-sm text-gray-700 hover:text-gray-900 hover:bg-gray-100 rounded-lg"
            >
              <LogOut className="w-4 h-4" />
              <span>退出</span>
            </button>
          </div>
        </div>
      </header>

      <div className="flex">
        {/* 左侧导航 */}
        <aside
          className={`fixed left-0 top-16 bottom-0 w-64 bg-white border-r border-gray-200 transition-transform ${
            isSidebarOpen ? 'translate-x-0' : '-translate-x-full'
          }`}
        >
          <nav className="p-4 space-y-1">
            {navItems.map((item) => {
              const Icon = item.icon
              const isActive = pathname === item.href || pathname.startsWith(item.href + '/')
              
              return (
                <Link
                  key={item.id}
                  href={item.href}
                  className={`flex items-center justify-between px-4 py-3 rounded-lg transition-colors ${
                    isActive
                      ? 'bg-primary-50 text-primary-700 font-medium'
                      : 'text-gray-700 hover:bg-gray-50 hover:text-gray-900'
                  }`}
                >
                  <div className="flex items-center gap-3">
                    <Icon className="w-5 h-5" />
                    <span>{item.label}</span>
                  </div>
                  {item.badge && (
                    <span className="px-2 py-0.5 bg-red-500 text-white text-xs font-medium rounded-full">
                      {item.badge}
                    </span>
                  )}
                </Link>
              )
            })}
          </nav>
        </aside>

        {/* 主内容区 */}
        <main
          className={`flex-1 transition-all ${
            isSidebarOpen ? 'ml-64' : 'ml-0'
          }`}
        >
          {children}
        </main>
      </div>
    </div>
  )
}
