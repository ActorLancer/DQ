# 认证系统测试指南

## 🎯 快速测试步骤

### 第一步：启动开发服务器

```bash
cd apps/portal-web
pnpm dev
```

访问: `http://localhost:3001`

### 第二步：测试买家登录

1. 访问 `http://localhost:3001/login`
2. 选择"买家"角色（左侧按钮）
3. 输入任意邮箱（如 `buyer@test.com`）
4. 输入任意密码（如 `123456`）
5. 点击"登录"按钮
6. ✅ 应该跳转到 `/console/buyer` 买家仪表盘

### 第三步：测试状态持久化

1. 在买家控制台页面，按 `F5` 刷新页面
2. ✅ 应该保持登录状态，不会跳转到登录页
3. ✅ 页面应该正常显示买家控制台内容

### 第四步：测试角色隔离

1. 保持买家登录状态
2. 手动访问 `http://localhost:3001/console/seller`
3. ✅ 应该跳转到 `/unauthorized` 页面
4. ✅ 显示"权限不足"提示

### 第五步：测试供应商登录

1. 点击"退出登录"或访问 `http://localhost:3001/login`
2. 选择"供应商"角色（右侧按钮）
3. 输入任意邮箱（如 `seller@test.com`）
4. 输入任意密码（如 `123456`）
5. 点击"登录"按钮
6. ✅ 应该跳转到 `/console/seller` 供应商仪表盘

### 第六步：测试管理员登录

1. 访问 `http://localhost:3001/admin-login`
2. 输入任意邮箱（如 `admin@test.com`）
3. 输入任意密码（如 `123456`）
4. 点击"登录"按钮
5. ✅ 应该跳转到 `/console/admin` 平台运营仪表盘

### 第七步：使用调试页面

1. 访问 `http://localhost:3001/auth-debug`
2. ✅ 查看当前认证状态
3. ✅ 查看用户信息（姓名、邮箱、角色、权限）
4. ✅ 查看角色检查结果
5. ✅ 查看 LocalStorage 内容

## 🔍 问题排查

### 问题 1: 登录后跳转到 /unauthorized

**检查步骤**:

1. 打开 `/auth-debug` 页面
2. 查看"认证状态"部分:
   - `isAuthenticated` 应该是 `✓ 已认证`
   - `Token` 应该有值
3. 查看"用户信息"部分:
   - `角色列表` 应该包含你选择的角色
4. 查看"角色检查"部分:
   - 对应角色的 `hasRole()` 应该是 `✓ 有权限`

**可能原因**:
- 状态未正确保存
- 角色未正确设置
- LocalStorage 被清除

**解决方案**:
1. 清除浏览器缓存和 LocalStorage
2. 重新登录
3. 查看浏览器控制台是否有错误

### 问题 2: 刷新后跳转到登录页

**检查步骤**:

1. 登录后，打开浏览器控制台（F12）
2. 切换到 "Application" 或 "存储" 标签
3. 查看 "Local Storage" → `http://localhost:3001`
4. 应该看到 `auth-storage` 键

**可能原因**:
- LocalStorage 未正确保存
- 浏览器隐私模式
- 浏览器扩展干扰

**解决方案**:
1. 确保不在隐私/无痕模式
2. 禁用可能干扰的浏览器扩展
3. 检查浏览器控制台的错误信息

### 问题 3: 无法访问控制台

**检查步骤**:

1. 打开浏览器控制台（F12）
2. 查看 Console 标签
3. 应该看到类似的日志:
   ```
   ✅ 权限检查通过: {
     isAuthenticated: true,
     requiredRole: "buyer",
     userRoles: ["buyer"],
     hasRole: true
   }
   ```

**可能原因**:
- 角色不匹配
- 权限检查失败

**解决方案**:
1. 确认登录时选择的角色
2. 确认访问的控制台路径
3. 重新登录

## 🧪 完整测试场景

### 场景 1: 买家完整流程

```
1. 访问首页 → 点击"登录" → 选择"买家" → 输入凭证 → 登录成功
2. 跳转到买家仪表盘 → 查看统计卡片
3. 点击左侧"订阅管理" → 查看订阅列表
4. 点击左侧"申请记录" → 查看申请列表
5. 点击左侧"API 密钥" → 查看密钥列表
6. 刷新页面 → 保持登录状态
7. 尝试访问 /console/seller → 跳转到 /unauthorized
8. 点击"退出登录" → 跳转到首页
```

### 场景 2: 供应商完整流程

```
1. 访问首页 → 点击"登录" → 选择"供应商" → 输入凭证 → 登录成功
2. 跳转到供应商仪表盘 → 查看统计卡片
3. 点击左侧"商品管理" → 查看商品列表
4. 点击"创建商品" → 进入创建向导
5. 点击左侧"客户管理" → 查看客户列表
6. 点击左侧"收入看板" → 查看收入统计
7. 刷新页面 → 保持登录状态
8. 尝试访问 /console/buyer → 跳转到 /unauthorized
9. 点击"退出登录" → 跳转到首页
```

### 场景 3: 管理员完整流程

```
1. 访问 /admin-login → 输入凭证 → 登录成功
2. 跳转到平台运营仪表盘 → 查看紧急统计
3. 点击左侧"主体审核" → 查看待审核主体
4. 点击左侧"商品审核" → 查看待审核商品
5. 点击左侧"一致性检查" → 查看系统状态
6. 刷新页面 → 保持登录状态
7. 尝试访问 /console/buyer → 跳转到 /unauthorized
8. 点击"退出登录" → 跳转到首页
```

## 📊 测试检查清单

### 基础功能
- [ ] 买家登录成功
- [ ] 供应商登录成功
- [ ] 管理员登录成功
- [ ] 登录后跳转正确
- [ ] 退出登录成功

### 状态持久化
- [ ] 刷新页面保持登录
- [ ] 关闭标签页重新打开保持登录
- [ ] LocalStorage 正确保存

### 权限控制
- [ ] 买家无法访问供应商后台
- [ ] 供应商无法访问买家控制台
- [ ] 买家/供应商无法访问管理员后台
- [ ] 未登录无法访问任何控制台

### 路由保护
- [ ] 未登录访问控制台跳转登录页
- [ ] 登录后自动返回原页面（returnUrl）
- [ ] 权限不足跳转 /unauthorized

### UI 显示
- [ ] SessionIdentityBar 显示正确信息
- [ ] 左侧导航显示对应角色的菜单
- [ ] 用户头像和名称显示正确
- [ ] 会话倒计时正常工作

## 🎯 性能测试

### 1. 登录速度
- Mock 登录应该在 1 秒内完成
- 页面跳转应该流畅无卡顿

### 2. 状态恢复速度
- 刷新页面后状态恢复应该在 100ms 内完成
- 不应该出现闪烁或跳转

### 3. 权限检查速度
- 权限检查应该是同步的，无延迟
- 不应该出现"加载中"状态

## 🔧 开发者工具

### 1. React DevTools

安装 React DevTools 扩展，查看组件状态:

```
Components → ProtectedRoute → hooks → useAuthStore
```

### 2. Redux DevTools (Zustand)

安装 Redux DevTools 扩展，查看状态变化:

```
State → auth-storage
```

### 3. 浏览器控制台

查看调试日志:

```javascript
// 查看认证状态
console.log(useAuthStore.getState())

// 手动登录
useAuthStore.getState().login('mock_token', {
  id: 'test_user',
  name: '测试用户',
  email: 'test@example.com',
  roles: ['buyer'],
  currentRole: 'buyer',
  subjectId: 'subject_001',
  subjectName: '测试公司',
  tenantId: 'tenant_001',
  permissions: ['buyer:*:read', 'buyer:*:write']
})

// 手动登出
useAuthStore.getState().logout()
```

## 📝 测试报告模板

```markdown
## 测试日期: YYYY-MM-DD
## 测试人员: [姓名]
## 浏览器: [Chrome/Firefox/Safari] [版本号]

### 测试结果

#### 基础功能
- [x] 买家登录: ✅ 通过
- [x] 供应商登录: ✅ 通过
- [x] 管理员登录: ✅ 通过

#### 状态持久化
- [x] 刷新保持登录: ✅ 通过
- [x] LocalStorage 保存: ✅ 通过

#### 权限控制
- [x] 角色隔离: ✅ 通过
- [x] 路由保护: ✅ 通过

### 发现的问题

1. [问题描述]
   - 重现步骤: ...
   - 预期结果: ...
   - 实际结果: ...
   - 截图: ...

### 建议

1. [改进建议]
```

## ✅ 测试通过标准

所有以下条件都满足才算测试通过:

1. ✅ 三种角色都能正常登录
2. ✅ 登录后跳转到正确的控制台
3. ✅ 刷新页面保持登录状态
4. ✅ 角色隔离正常工作
5. ✅ 未登录无法访问控制台
6. ✅ 权限不足跳转到 /unauthorized
7. ✅ 退出登录清除状态
8. ✅ LocalStorage 正确保存和恢复
9. ✅ 无控制台错误
10. ✅ UI 显示正确

## 🚀 自动化测试（未来）

可以使用 Playwright 或 Cypress 编写自动化测试:

```typescript
// 示例: Playwright 测试
test('buyer login flow', async ({ page }) => {
  await page.goto('http://localhost:3001/login')
  await page.click('[data-role="buyer"]')
  await page.fill('[name="email"]', 'buyer@test.com')
  await page.fill('[name="password"]', '123456')
  await page.click('button[type="submit"]')
  await expect(page).toHaveURL('/console/buyer')
})
```

## 📞 需要帮助？

如果测试过程中遇到问题:

1. 查看 `/auth-debug` 页面
2. 查看浏览器控制台日志
3. 查看 `AUTH_IMPLEMENTATION_COMPLETE.md` 文档
4. 清除浏览器缓存和 LocalStorage 后重试
