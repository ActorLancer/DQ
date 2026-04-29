'use client'

import { useState } from 'react'
import Link from 'next/link'
import { useRouter, useSearchParams } from 'next/navigation'
import { useAuthStore, UserRole } from '@/store/useAuthStore'
import { 
  Building2, 
  ShoppingCart, 
  Mail, 
  Lock, 
  Eye, 
  EyeOff,
  ArrowLeft
} from 'lucide-react'

export default function LoginPage() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const { login } = useAuthStore()
  
  const [selectedRole, setSelectedRole] = useState<UserRole>('buyer')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState('')
  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setIsLoading(true)

    try {
      // Mock 登录逻辑 - 模拟 API 调用
      await new Promise(resolve => setTimeout(resolve, 1000))

      // Mock 用户数据
      const hasDualRole = email.toLowerCase().includes('dual') || email.toLowerCase().includes('both')
      const availableRoles: UserRole[] = hasDualRole ? ['buyer', 'seller'] : [selectedRole]
      const mockUser = {
        id: `user_${Date.now()}`,
        name: selectedRole === 'buyer' ? '李四' : '张三',
        email: email,
        roles: availableRoles,
        currentRole: selectedRole,
        subjectId: `subject_${selectedRole}_001`,
        subjectName: selectedRole === 'buyer' ? '某某科技有限公司' : '天眼数据科技',
        tenantId: `tenant_${selectedRole}_001`,
        permissions: availableRoles.flatMap((role) => [`${role}:*:read`, `${role}:*:write`]),
      }

      // Mock Token
      const mockToken = `mock_token_${Date.now()}_${selectedRole}`

      // 保存到状态
      login(mockToken, mockUser)

      // 跳转到对应控制台
      const returnUrl = searchParams.get('returnUrl')
      if (returnUrl) {
        router.push(decodeURIComponent(returnUrl))
      } else {
        router.push(`/console/${selectedRole}`)
      }
    } catch (err) {
      setError('登录失败，请重试')
      setIsLoading(false)
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-primary-50 via-white to-blue-50 flex items-center justify-center p-4">
      <div className="w-full max-w-6xl grid grid-cols-1 lg:grid-cols-2 gap-8 items-center">
        {/* 左侧：品牌和说明 */}
        <div className="hidden lg:block">
          <Link href="/" className="inline-flex items-center gap-2 text-primary-600 hover:text-primary-700 mb-8">
            <ArrowLeft className="w-5 h-5" />
            <span>返回首页</span>
          </Link>
          
          <h1 className="text-4xl font-bold text-gray-900 mb-4">
            欢迎来到<br />数据交易平台
          </h1>
          <p className="text-lg text-gray-600 mb-8">
            安全、透明、高效的数据交易服务
          </p>

          <div className="space-y-4">
            <div className="flex items-start gap-3">
              <div className="w-10 h-10 bg-primary-100 rounded-lg flex items-center justify-center flex-shrink-0">
                <ShoppingCart className="w-5 h-5 text-primary-600" />
              </div>
              <div>
                <h3 className="font-medium text-gray-900 mb-1">买家</h3>
                <p className="text-sm text-gray-600">发现优质数据产品，快速接入 API，助力业务增长</p>
              </div>
            </div>

            <div className="flex items-start gap-3">
              <div className="w-10 h-10 bg-green-100 rounded-lg flex items-center justify-center flex-shrink-0">
                <Building2 className="w-5 h-5 text-green-600" />
              </div>
              <div>
                <h3 className="font-medium text-gray-900 mb-1">供应商</h3>
                <p className="text-sm text-gray-600">发布数据产品，触达海量客户，实现数据价值变现</p>
              </div>
            </div>
          </div>
        </div>

        {/* 右侧：登录表单 */}
        <div className="bg-white rounded-2xl shadow-xl p-8 lg:p-12">
          <div className="lg:hidden mb-6">
            <Link href="/" className="inline-flex items-center gap-2 text-primary-600 hover:text-primary-700 text-sm">
              <ArrowLeft className="w-4 h-4" />
              <span>返回首页</span>
            </Link>
          </div>

          <h2 className="text-2xl font-bold text-gray-900 mb-2">登录账号</h2>
          <p className="text-gray-600 mb-8">选择您的角色并登录</p>

          {/* 角色选择 */}
          <div className="grid grid-cols-2 gap-4 mb-8">
            <button
              type="button"
              onClick={() => setSelectedRole('buyer')}
              className={`p-4 rounded-xl border-2 transition-all ${
                selectedRole === 'buyer'
                  ? 'border-primary-500 bg-primary-50'
                  : 'border-gray-200 hover:border-gray-300'
              }`}
            >
              <ShoppingCart className={`w-6 h-6 mx-auto mb-2 ${
                selectedRole === 'buyer' ? 'text-primary-600' : 'text-gray-400'
              }`} />
              <div className={`text-sm font-medium ${
                selectedRole === 'buyer' ? 'text-primary-900' : 'text-gray-600'
              }`}>
                买家
              </div>
            </button>

            <button
              type="button"
              onClick={() => setSelectedRole('seller')}
              className={`p-4 rounded-xl border-2 transition-all ${
                selectedRole === 'seller'
                  ? 'border-green-500 bg-green-50'
                  : 'border-gray-200 hover:border-gray-300'
              }`}
            >
              <Building2 className={`w-6 h-6 mx-auto mb-2 ${
                selectedRole === 'seller' ? 'text-green-600' : 'text-gray-400'
              }`} />
              <div className={`text-sm font-medium ${
                selectedRole === 'seller' ? 'text-green-900' : 'text-gray-600'
              }`}>
                供应商
              </div>
            </button>
          </div>

          {/* 登录表单 */}
          <form onSubmit={handleLogin} className="space-y-6">
            {/* 错误提示 */}
            {error && (
              <div className="p-3 bg-red-50 border border-red-200 rounded-lg">
                <p className="text-sm text-red-800">{error}</p>
              </div>
            )}

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                邮箱地址
              </label>
              <div className="relative">
                <Mail className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="your@email.com"
                  className="w-full pl-10 pr-4 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
                  required
                />
              </div>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                密码
              </label>
              <div className="relative">
                <Lock className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
                <input
                  type={showPassword ? 'text' : 'password'}
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="••••••••"
                  className="w-full pl-10 pr-12 py-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
                  required
                />
                <button
                  type="button"
                  onClick={() => setShowPassword(!showPassword)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
                >
                  {showPassword ? (
                    <EyeOff className="w-5 h-5" />
                  ) : (
                    <Eye className="w-5 h-5" />
                  )}
                </button>
              </div>
            </div>

            <div className="flex items-center justify-between">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  className="w-4 h-4 text-primary-600 rounded focus:ring-2 focus:ring-primary-500"
                />
                <span className="text-sm text-gray-600">记住我</span>
              </label>
              <a href="#" className="text-sm text-primary-600 hover:text-primary-700">
                忘记密码？
              </a>
            </div>

            <button
              type="submit"
              disabled={isLoading}
              className={`w-full py-3 rounded-lg font-medium transition-colors ${
                selectedRole === 'buyer'
                  ? 'bg-primary-600 hover:bg-primary-700 text-white'
                  : 'bg-green-600 hover:bg-green-700 text-white'
              } disabled:opacity-50 disabled:cursor-not-allowed`}
            >
              {isLoading ? '登录中...' : '登录'}
            </button>
          </form>

          {/* 注册链接 */}
          <div className="mt-6 text-center">
            <p className="text-sm text-gray-600">
              还没有账号？{' '}
              <Link 
                href={selectedRole === 'buyer' ? '/register/buyer' : '/register/seller'}
                className={selectedRole === 'buyer' ? 'text-primary-600 hover:text-primary-700 font-medium' : 'text-green-600 hover:text-green-700 font-medium'}
              >
                立即注册
              </Link>
            </p>
          </div>

          {/* Demo 提示 */}
          <div className="mt-8 p-4 bg-blue-50 border border-blue-200 rounded-lg">
            <p className="text-xs text-blue-800 text-center">
              <strong>Demo 提示：</strong> 输入任意邮箱和密码即可登录。邮箱包含 dual 或 both 可体验买卖双角色账号。
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
