'use client'

import { useState, useEffect } from 'react'
import { Building2, User, Clock, Shield, ChevronDown } from 'lucide-react'

interface SessionIdentityBarProps {
  subjectName: string
  roleName: string
  tenantId: string
  scope: string
  sessionExpiresAt: string
  userName?: string
}

export default function SessionIdentityBar({
  subjectName,
  roleName,
  tenantId,
  scope,
  sessionExpiresAt,
  userName = '张三',
}: SessionIdentityBarProps) {
  const [timeRemaining, setTimeRemaining] = useState('')
  const [isWarning, setIsWarning] = useState(false)

  useEffect(() => {
    const updateTimer = () => {
      const now = new Date()
      const expires = new Date(sessionExpiresAt)
      const diff = expires.getTime() - now.getTime()
      
      if (diff <= 0) {
        setTimeRemaining('已过期')
        setIsWarning(true)
        return
      }

      const minutes = Math.floor(diff / 60000)
      const seconds = Math.floor((diff % 60000) / 1000)
      
      setTimeRemaining(`${minutes}:${seconds.toString().padStart(2, '0')}`)
      setIsWarning(minutes < 5)
    }

    updateTimer()
    const interval = setInterval(updateTimer, 1000)
    return () => clearInterval(interval)
  }, [sessionExpiresAt])

  return (
    <div className={`sticky top-16 z-30 border-b transition-colors ${
      isWarning ? 'bg-red-50 border-red-200' : 'bg-primary-50 border-primary-200'
    }`}>
      <div className="container-custom">
        <div className="flex items-center justify-between h-10 text-sm">
          {/* 左侧：主体和角色 */}
          <div className="flex items-center gap-6">
            <div className="flex items-center gap-2">
              <Building2 className="w-4 h-4 text-primary-600" />
              <span className="text-gray-600">主体:</span>
              <button className="font-medium text-primary-900 hover:text-primary-600 flex items-center gap-1">
                {subjectName}
                <ChevronDown className="w-3 h-3" />
              </button>
            </div>
            
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4 text-primary-600" />
              <span className="text-gray-600">角色:</span>
              <button className="font-medium text-primary-900 hover:text-primary-600 flex items-center gap-1">
                {roleName}
                <ChevronDown className="w-3 h-3" />
              </button>
            </div>

            <div className="flex items-center gap-2">
              <span className="text-gray-600">租户:</span>
              <code className="font-mono text-xs text-primary-900 bg-white px-2 py-0.5 rounded">
                {tenantId}
              </code>
            </div>

            <div className="flex items-center gap-2">
              <span className="text-gray-600">作用域:</span>
              <code className="font-mono text-xs text-primary-900 bg-white px-2 py-0.5 rounded">
                {scope}
              </code>
            </div>
          </div>

          {/* 右侧：用户和会话 */}
          <div className="flex items-center gap-6">
            <div className="flex items-center gap-2">
              <User className="w-4 h-4 text-gray-600" />
              <span className="text-gray-900">{userName}</span>
            </div>

            <div className={`flex items-center gap-2 ${isWarning ? 'text-red-600 animate-pulse' : 'text-gray-600'}`}>
              <Clock className="w-4 h-4" />
              <span className="text-xs">会话有效期:</span>
              <span className="font-mono text-xs font-medium">{timeRemaining}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
