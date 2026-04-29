# 认证系统说明

## 🎯 概述

已完成完整的认证和权限控制系统，解决了登录后跳转到 `/unauthorized` 的问题。

## ✅ 问题已解决

### 原问题
用户登录后，访问控制台会被重定向到 `/unauthorized` 页面。

### 根本原因
1. **Zustand persist 状态恢复延迟**: Zustand 的 persist 中间件需要时间从 localStorage 恢复状态
2. **过早的权限检查**: `ProtectedRoute` 在状态恢复完成前就执行了权限检查
3. **partialize 配置问题**: 不必要的 partialize 配置可能导致状态恢复不完整

### 解决方案

#### 1. 修复 `useAuthStore.ts`
- 移除了 `partialize` 配置
- 让 persist 中间件自动处理所有状态
- 简化了 logout 函数（persist 会自动清理）

#### 2. 修复 `ProtectedRoute.tsx`
- 添加 `isHydrated` 状态跟踪
- 等待状态恢复完成后再执行权限检查
- 添加调试日志，方便排查问题
- 改进加载状态显示

## 🚀 快速测试

### 1. 启动开发服务器
```bash
cd apps/portal-web
pnpm dev
```

### 2. 测试买家登录
1. 访问 `http://localhost:3001/login`
2. 选择"买家"角色
3. 输入任意邮箱和密码
4. 点击登录
5. ✅ 应该成功跳转到 `/console/buyer`

### 3. 测试状态持久化
1. 在控制台页面按 F5 刷新
2. ✅ 应该保持登录状态，不会跳转

### 4. 测试角色隔离
1. 以买家身份登录
2. 访问 `/console/seller`
3. ✅ 应该跳转到 `/unauthorized`

## 🔍 调试工具

### 调试页面
访问 `http://localhost:3001/auth-debug` 查看：
- 当前认证状态
- 用户信息（角色、权限）
- 角色检查结果
- LocalStorage 内容

### 浏览器控制台
打开控制台查看调试日志：
```
✅ 权限检查通过: {
  isAuthenticated: true,
  requiredRole: "buyer",
  userRoles: ["buyer"],
  hasRole: true
}
```

## 📁 核心文件

### 状态管理
- `src/store/useAuthStore.ts` - 认证状态管理

### 页面
- `src/app/login/page.tsx` - 登录页面
- `src/app/admin-login/page.tsx` - 管理员登录
- `src/app/unauthorized/page.tsx` - 未授权页面
- `src/app/auth-debug/page.tsx` - 调试页面

### 组件
- `src/components/auth/ProtectedRoute.tsx` - 路由保护
- `src/components/auth/PermissionGate.tsx` - 权限门控
- `src/components/console/SessionIdentityBar.tsx` - 身份栏
- `src/components/console/ConsoleLayout.tsx` - 控制台布局

### 布局（路由保护）
- `src/app/console/buyer/layout.tsx` - 买家控制台
- `src/app/console/seller/layout.tsx` - 供应商后台
- `src/app/console/admin/layout.tsx` - 平台运营

## 📚 文档

### 完整文档
- `AUTH_IMPLEMENTATION_COMPLETE.md` - 完整实现文档
- `AUTH_TESTING_GUIDE.md` - 测试指南
- `AUTH_SYSTEM.md` - 本文档（快速参考）

## 🎨 功能特性

### 认证功能
- ✅ 买家/供应商/管理员登录
- ✅ Mock 登录（任意邮箱密码）
- ✅ 自动跳转到对应控制台
- ✅ returnUrl 支持（登录后返回原页面）
- ✅ 退出登录

### 权限控制
- ✅ 角色检查（buyer/seller/admin）
- ✅ 权限检查（细粒度权限）
- ✅ 路由保护（未登录跳转登录页）
- ✅ 角色隔离（买家无法访问供应商后台）

### 状态管理
- ✅ LocalStorage 持久化
- ✅ 刷新页面保持登录
- ✅ 自动状态恢复
- ✅ Token 管理

### 用户体验
- ✅ 加载状态显示
- ✅ 友好的错误提示
- ✅ 会话倒计时（SessionIdentityBar）
- ✅ 调试工具

## 🔐 安全建议

### 当前实现（Demo）
- 使用 Mock 登录
- Token 存储在 LocalStorage
- 前端权限检查

### 生产环境建议
1. **Token 安全**
   - 使用 HttpOnly Cookie
   - 实现 Token 刷新
   - 设置合理过期时间

2. **密码安全**
   - 密码强度检查
   - 验证码防暴力破解
   - 密码重置功能

3. **会话管理**
   - 会话超时自动登出
   - 多设备登录管理
   - 登录日志记录

4. **权限控制**
   - 后端必须验证所有请求
   - 前端权限仅用于 UI 显示
   - 细粒度权限控制

## 💡 使用示例

### 在组件中使用认证状态
```tsx
import { useAuthStore } from '@/store/useAuthStore'

function MyComponent() {
  const { user, isAuthenticated, hasRole, logout } = useAuthStore()

  if (!isAuthenticated) {
    return <div>请先登录</div>
  }

  return (
    <div>
      <p>欢迎, {user?.name}</p>
      {hasRole('admin') && <AdminPanel />}
      <button onClick={logout}>退出登录</button>
    </div>
  )
}
```

### 保护路由
```tsx
import ProtectedRoute from '@/components/auth/ProtectedRoute'

export default function BuyerPage() {
  return (
    <ProtectedRoute requiredRole="buyer">
      <BuyerContent />
    </ProtectedRoute>
  )
}
```

### 基于权限显示内容
```tsx
import PermissionGate from '@/components/auth/PermissionGate'

function MyComponent() {
  return (
    <div>
      <PermissionGate requiredRole="admin">
        <AdminOnlyButton />
      </PermissionGate>
      
      <PermissionGate requiredPermission="buyer:orders:delete">
        <DeleteOrderButton />
      </PermissionGate>
    </div>
  )
}
```

## 🎯 Demo 账号

任意邮箱和密码都可以登录（Mock 模式）：

```
买家:
邮箱: buyer@test.com
密码: 任意

供应商:
邮箱: seller@test.com
密码: 任意

管理员:
邮箱: admin@test.com
密码: 任意
```

## ✅ 测试清单

- [x] 买家登录成功
- [x] 供应商登录成功
- [x] 管理员登录成功
- [x] 刷新页面保持登录
- [x] 角色隔离正常工作
- [x] 未登录跳转登录页
- [x] 权限不足跳转 /unauthorized
- [x] 退出登录清除状态
- [x] LocalStorage 正确保存
- [x] 调试页面正常工作

## 🐛 问题排查

如果遇到问题：

1. **访问调试页面**: `http://localhost:3001/auth-debug`
2. **查看浏览器控制台**: 检查错误和调试日志
3. **检查 LocalStorage**: Application → Local Storage → auth-storage
4. **清除缓存**: 清除浏览器缓存和 LocalStorage 后重试
5. **查看文档**: 阅读 `AUTH_TESTING_GUIDE.md`

## 📞 支持

详细文档：
- `AUTH_IMPLEMENTATION_COMPLETE.md` - 完整实现说明
- `AUTH_TESTING_GUIDE.md` - 详细测试步骤
- `/auth-debug` - 在线调试工具

---

**状态**: ✅ 已完成并测试通过  
**更新时间**: 2026-04-29
