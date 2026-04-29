# 快速开始指南

## 🚀 5 分钟快速启动

### 1. 启动开发服务器
```bash
cd apps/portal-web
pnpm dev
```

### 2. 访问应用
打开浏览器访问: `http://localhost:3001`

### 3. 测试登录
访问 `http://localhost:3001/login`，使用任意邮箱和密码登录。

---

## 🎯 快速导航

### 公开页面
- **首页**: http://localhost:3001
- **市场**: http://localhost:3001/marketplace
- **商品详情**: http://localhost:3001/products/prod_001

### 登录页面
- **买家/供应商登录**: http://localhost:3001/login
- **管理员登录**: http://localhost:3001/admin-login
- **调试页面**: http://localhost:3001/auth-debug

### 买家控制台
- **Dashboard**: http://localhost:3001/console/buyer
- **我的订阅**: http://localhost:3001/console/buyer/subscriptions
- **我的申请**: http://localhost:3001/console/buyer/requests
- **订单账单**: http://localhost:3001/console/buyer/orders
- **API 密钥**: http://localhost:3001/console/buyer/api-keys
- **使用分析**: http://localhost:3001/console/buyer/usage
- **设置**: http://localhost:3001/console/buyer/settings

### 供应商后台
- **Dashboard**: http://localhost:3001/console/seller
- **商品管理**: http://localhost:3001/console/seller/listings
- **创建商品**: http://localhost:3001/console/seller/listings/create
- **申请审批**: http://localhost:3001/console/seller/requests
- **客户管理**: http://localhost:3001/console/seller/customers
- **收入看板**: http://localhost:3001/console/seller/revenue
- **调用分析**: http://localhost:3001/console/seller/analytics

### 平台运营
- **Dashboard**: http://localhost:3001/console/admin
- **主体审核**: http://localhost:3001/console/admin/subjects
- **商品审核**: http://localhost:3001/console/admin/listings
- **一致性检查**: http://localhost:3001/console/admin/consistency

---

## 🔑 Demo 账号

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

---

## ✅ 快速测试清单

### 基础功能
- [ ] 访问首页，查看 Hero 和搜索
- [ ] 访问市场页面，使用筛选和排序
- [ ] 访问商品详情，查看 Sticky 标签和申请面板

### 买家流程
- [ ] 以买家身份登录
- [ ] 查看 Dashboard 统计
- [ ] 查看我的订阅，检查配额可视化
- [ ] 查看 API 密钥，尝试创建新密钥
- [ ] 刷新页面，确认保持登录状态

### 供应商流程
- [ ] 以供应商身份登录
- [ ] 查看 Dashboard 统计
- [ ] 查看商品管理列表
- [ ] 进入创建商品向导，完成 Step 1
- [ ] 查看申请审批，检查风险等级标签

### 管理员流程
- [ ] 以管理员身份登录
- [ ] 查看 Dashboard 紧急待办
- [ ] 查看主体审核，尝试审核操作
- [ ] 查看一致性检查，检查三列状态指示器

### 认证测试
- [ ] 登录后刷新页面，确认保持登录
- [ ] 以买家身份访问供应商后台，确认跳转到 /unauthorized
- [ ] 访问 /auth-debug，查看认证状态
- [ ] 退出登录，确认清除状态

---

## 🐛 遇到问题？

### 1. 无法登录
- 确认开发服务器正在运行
- 清除浏览器缓存和 LocalStorage
- 访问 `/auth-debug` 查看状态

### 2. 登录后跳转到 /unauthorized
- 访问 `/auth-debug` 查看角色信息
- 查看浏览器控制台日志
- 确认访问的路由与登录角色匹配

### 3. 刷新后跳转到登录页
- 检查 LocalStorage 是否有 `auth-storage` 键
- 查看浏览器控制台是否有错误
- 确认不在隐私/无痕模式

### 4. 端口冲突
如果 3001 端口被占用，修改 `package.json`:
```json
{
  "scripts": {
    "dev": "next dev -p 3002"
  }
}
```

---

## 📚 详细文档

- **完整总结**: `FINAL_SUMMARY.md`
- **认证系统**: `AUTH_SYSTEM.md`
- **测试指南**: `AUTH_TESTING_GUIDE.md`
- **实现细节**: `AUTH_IMPLEMENTATION_COMPLETE.md`
- **控制台进度**: `CONSOLE_PROGRESS.md`

---

## 🎯 核心功能速览

### 认证系统 ✅
- 三种角色登录（买家/供应商/管理员）
- 状态持久化（刷新保持登录）
- 角色隔离（权限控制）
- 路由保护（未登录跳转）

### 买家控制台 ✅
- Dashboard（统计 + 活跃订阅）
- 订阅管理（配额可视化）
- 申请记录（8 种状态）
- API 密钥（完整 CRUD）
- 使用分析（图表占位）
- 设置（5 个标签页）

### 供应商后台 ✅
- Dashboard（统计 + 待处理申请）
- 商品管理（7 种状态）
- 创建商品（6 步向导）
- 申请审批（风险等级）
- 客户管理（统计信息）
- 收入看板（图表占位）
- 调用分析（图表占位）

### 平台运营 ✅
- Dashboard（紧急待办）
- 主体审核（资质文件）
- 商品审核（质量评分）
- 一致性检查（三列状态）⭐

---

## 🎨 设计规范

### 颜色
- 主色: #2563EB
- 深色背景: #0F172A
- 成功: #059669
- 警告: #D97706
- 错误: #DC2626

### 动画
- Fade-in: 150ms
- Slide-in: 250ms
- 无弹跳、上浮动画

### 字体
- 等宽: Request ID, TX Hash, API Key
- 常规: 其他文本

---

## 💡 开发提示

### 添加新页面
1. 在 `src/app/` 下创建新路由文件夹
2. 创建 `page.tsx` 文件
3. 如需保护，使用 `ProtectedRoute` 包裹

### 添加新组件
1. 在 `src/components/` 下创建组件文件
2. 使用 TypeScript 定义 Props
3. 导出组件供页面使用

### 使用认证状态
```tsx
import { useAuthStore } from '@/store/useAuthStore'

function MyComponent() {
  const { user, isAuthenticated, hasRole } = useAuthStore()
  
  if (!isAuthenticated) return <div>请登录</div>
  
  return <div>欢迎, {user?.name}</div>
}
```

### 保护路由
```tsx
import ProtectedRoute from '@/components/auth/ProtectedRoute'

export default function MyPage() {
  return (
    <ProtectedRoute requiredRole="buyer">
      <MyContent />
    </ProtectedRoute>
  )
}
```

---

**状态**: ✅ 全部完成  
**端口**: 3001  
**模式**: Mock 数据

🎉 **开始探索吧！** 🎉
