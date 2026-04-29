# 数据交易平台前端 - 项目总览

## 🎯 项目定位

这是一个**可信数据商品交易平台**的前端 MVP 实现，专注于展示、搜索、筛选、详情查看和申请访问等核心功能。

## ✨ 核心特性

### 1. 可信展示
- 链上存证徽章
- Request ID / Tx Hash 展示
- 链状态和投影状态实时显示
- 合规标签和认证展示

### 2. 高效搜索
- 全局搜索框
- 热门关键词快速跳转
- 多维度筛选（8+ 类筛选项）
- 7 种排序方式

### 3. 详细信息
- 完整的商品详情（Overview / Schema / Sample / Pricing / Docs / Reviews）
- 供应商信息卡
- 质量指标展示
- 授权协议说明

### 4. 流畅申请
- 4 步分步流程
- 表单验证
- 合规确认
- 幂等提交
- 状态透明展示

## 🎨 设计原则

### 克制的设计
- ❌ 花哨插画
- ❌ 弹跳动画
- ❌ 过度装饰
- ✅ 功能性动画（150ms/250ms）
- ✅ 轻微阴影悬浮
- ✅ 充足留白

### 金融科技风格
- 深色背景 (#0F172A)
- 主色蓝 (#2563EB)
- 状态色彩明确（绿/黄/红）
- 等宽字体展示 Hash

### 数据信任
- 链上存证标记
- Request ID 可复制
- Tx Hash 可跳转
- 状态透明展示
- 合规标签突出

## 📊 页面结构

```
首页 (/)
├─ Hero Section (深色渐变)
├─ 全局搜索
├─ 行业分类 (9 个)
├─ 推荐商品 (6 个)
├─ 优质供应商 (4 个)
├─ 可信能力 (6 个)
└─ 标准链路 (3 条)

市场页 (/marketplace)
├─ 顶部搜索栏
├─ 左侧筛选面板 (1/4 宽度)
│  ├─ 行业分类
│  ├─ 交付方式
│  ├─ 授权方式
│  ├─ 价格模式
│  ├─ 质量评分
│  └─ 其他筛选
├─ 右侧结果区 (3/4 宽度)
│  ├─ 排序工具栏
│  ├─ 商品列表 (网格/列表)
│  └─ 分页
└─ URL 状态同步

商品详情页 (/products/[id])
├─ Hero Section (深色)
├─ Sticky Tabs (吸顶)
│  ├─ Overview
│  ├─ Schema
│  ├─ Sample
│  ├─ Pricing
│  ├─ Docs
│  └─ Reviews
├─ 左侧内容区 (2/3 宽度)
│  ├─ Tab 内容
│  ├─ 供应商信息
│  └─ 链上凭证
└─ 右侧 Sticky 面板 (1/3 宽度)
   ├─ 价格摘要
   ├─ 套餐选择
   ├─ 申请访问按钮
   ├─ 联系供应商
   └─ 收藏/对比

申请访问 Drawer (右侧滑出)
├─ Step 1: 选择套餐
├─ Step 2: 填写用途
├─ Step 3: 合规确认
└─ Step 4: 提交审批
   └─ 成功状态展示
      ├─ Request ID
      ├─ 工作流状态
      ├─ 链状态
      └─ 投影状态
```

## 🛠️ 技术栈

| 类别 | 技术 | 用途 |
|------|------|------|
| 框架 | Next.js 14 | SSR/SSG/ISR |
| 语言 | TypeScript | 类型安全 |
| 样式 | Tailwind CSS | 原子化 CSS |
| 动画 | Framer Motion | 克制动画 |
| 状态 | Zustand | 客户端状态 |
| 数据 | TanStack Query | 服务端状态 |
| 图标 | Lucide React | 现代图标 |

## 📦 组件清单

### 布局组件
- `Header` - 顶部导航
- `Footer` - 底部信息

### 首页组件
- `GlobalSearchBar` - 全局搜索
- `IndustryCategoryGrid` - 行业分类
- `ProductCard` - 商品卡片
- `SupplierCard` - 供应商卡片
- `TrustCapabilityCards` - 可信能力
- `StandardFlowEntrance` - 标准链路

### 市场页组件
- `TopSearchBar` - 顶部搜索
- `LeftFilterPanel` - 筛选面板
- `SortToolbar` - 排序工具栏

### 商品详情组件
- `StickyTabs` - 吸顶 Tabs
- `RightStickyApplyPanel` - 申请面板
- `ChainProofCard` - 链上凭证
- `AccessRequestDrawer` - 申请 Drawer

## 🎯 核心交互

### 1. 搜索流程
```
首页搜索框输入 → 按回车 → 跳转市场页 → 带 keyword 参数
```

### 2. 筛选流程
```
勾选筛选项 → 写入 URL → 刷新保持 → 可分享
```

### 3. 详情流程
```
点击商品卡片 → 进入详情页 → 切换 Tabs → 查看内容
```

### 4. 申请流程
```
点击申请访问 → Drawer 滑出 → 4 步流程 → 提交 → 状态展示
```

## 📈 状态管理

### 链状态 (ChainStatus)
- NOT_SUBMITTED - 未提交
- SUBMITTING - 提交中
- CONFIRMED - 已确认 ✅
- FAILED - 失败 ❌

### 投影状态 (ProjectionStatus)
- PENDING - 待投影
- PROJECTED - 已投影 ✅
- OUT_OF_SYNC - 不一致 ⚠️
- REBUILDING - 重建中

### 工作流状态
- PENDING_SUPPLIER_REVIEW - 待供应商审核
- PENDING_PLATFORM_REVIEW - 待平台审核
- APPROVED - 已通过
- REJECTED - 已拒绝

## 🔐 安全特性

### 幂等提交
```typescript
const idempotencyKey = `idem_${Date.now()}_${userId}`
// 重复提交返回相同结果
```

### 敏感信息保护
- API Key 仅创建时显示一次
- 复制操作写入审计日志
- Hash 使用等宽字体展示

### 合规确认
- 5 个必选合规条款
- 全部勾选后才能提交
- 黄色警告提示

## 📱 响应式设计

| 断点 | 宽度 | 布局 |
|------|------|------|
| sm | 640px | 移动端 |
| md | 768px | 平板 |
| lg | 1024px | 桌面 |
| xl | 1280px | 大屏 |
| 2xl | 1536px | 超大屏 |

## 🚀 性能优化

### 渲染策略
- 首页: SSG + ISR (10 分钟)
- 市场页: SSR
- 详情页: SSR / SSG + SWR

### 加载优化
- 骨架屏占位
- 图片懒加载
- 代码分割
- 路由预取

## 📝 数据模型

### 核心类型
- `Listing` - 数据商品
- `Supplier` - 供应商
- `PricingPlan` - 定价计划
- `ChainProof` - 链上凭证
- `AccessRequest` - 访问申请

### 状态类型
- `ChainStatus` - 链状态
- `ProjectionStatus` - 投影状态
- `ListingStatus` - 商品状态
- `AccessRequestStatus` - 申请状态

## 🎨 设计资源

### 色彩
```css
/* 主色 */
--primary-600: #2563EB;
--primary-900: #0F172A;

/* 状态色 */
--success-600: #059669;
--warning-600: #D97706;
--error-600: #DC2626;
```

### 间距
```css
/* 模块间留白 */
py-16  /* 4rem / 64px */
py-24  /* 6rem / 96px */

/* 卡片内边距 */
p-6    /* 1.5rem / 24px */
```

### 圆角
```css
rounded-lg   /* 0.5rem / 8px */
rounded-xl   /* 0.75rem / 12px */
```

## 📚 文档

- `README.md` - 项目说明
- `IMPLEMENTATION.md` - 实现细节
- `QUICKSTART.md` - 快速启动
- `PROJECT_OVERVIEW.md` - 项目总览（本文档）

## 🔄 下一步

### Sprint 2
- [ ] API Client SDK
- [ ] 真实数据接入
- [ ] 错误处理
- [ ] 登录/注册
- [ ] 供应商主页
- [ ] 标准链路页面

### Sprint 3
- [ ] 买家控制台
- [ ] 供应商后台
- [ ] 平台运营后台

## 📞 联系方式

如有问题或建议，请联系项目团队。

---

**版本**: MVP v1.0  
**更新时间**: 2026-04-28  
**状态**: ✅ 前端展示完成，待接入后端
