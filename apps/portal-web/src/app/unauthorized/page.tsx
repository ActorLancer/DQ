'use client'

import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { ShieldAlert, ArrowLeft, Home } from 'lucide-react'

export default function UnauthorizedPage() {
  const router = useRouter()

  return (
    <div className="min-h-screen bg-gradient-to-br from-red-50 via-white to-orange-50 flex items-center justify-center p-4">
      <div className="max-w-md w-full text-center">
        {/* 图标 */}
        <div className="w-24 h-24 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-6">
          <ShieldAlert className="w-12 h-12 text-red-600" />
        </div>

        {/* 标题 */}
        <h1 className="text-4xl font-bold text-gray-900 mb-4">403</h1>
        <h2 className="text-2xl font-bold text-gray-900 mb-4">权限不足</h2>
        
        {/* 描述 */}
        <p className="text-gray-600 mb-8">
          抱歉，您没有权限访问此页面。<br />
          您可以切换角色或更换账号后重试。
        </p>

        {/* 操作按钮 */}
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <button
            onClick={() => router.back()}
            className="flex items-center justify-center gap-2 px-6 py-3 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 font-medium"
          >
            <ArrowLeft className="w-4 h-4" />
            <span>返回上一页</span>
          </button>
          
          <Link
            href="/"
            className="flex items-center justify-center gap-2 px-6 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 font-medium"
          >
            <Home className="w-4 h-4" />
            <span>返回首页</span>
          </Link>
        </div>

        {/* 帮助信息 */}
        <div className="mt-12 p-4 bg-blue-50 border border-blue-200 rounded-lg text-left">
          <h3 className="text-sm font-medium text-blue-900 mb-2">需要帮助？</h3>
          <ul className="text-xs text-blue-800 space-y-1">
            <li>• 确认您已使用正确的账号登录</li>
            <li>• 检查您的账号角色和权限设置</li>
            <li>• 联系管理员申请相应的访问权限</li>
          </ul>
        </div>
      </div>
    </div>
  )
}
