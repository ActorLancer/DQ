# 数据交易平台 - Portal Web

基于 Next.js 14 的数据交易平台门户网站，包含首页、市场页和商品详情页。

## 技术栈

- **框架**: Next.js 14 (App Router)
- **语言**: TypeScript
- **样式**: Tailwind CSS
- **动画**: Framer Motion
- **状态管理**: Zustand + TanStack Query
- **图标**: Lucide React

## 功能特性

### 已实现

1. **首页 (Home)**
   - Hero Section 带全局搜索
   - 行业分类网格
   - 推荐数据商品
   - 优质供应商展示
   - 可信能力卡片
   - 标准链路入口

2. **市场页 (Marketplace)**
   - 左侧多维筛选面板
   - 顶部搜索栏
   - 排序工具栏
   - 网格/列表视图切换
   - URL 状态同步（筛选条件写入 URL）
   - 商品卡片展示
   - 分页

3. **商品详情页 (Product Detail)**
   - 深色 Hero Section
   - Sticky Tabs (Overview / Schema / Sample / Pricing / Docs / Reviews)
   - 右侧 Sticky 申请面板
   - 链上凭证卡片
   - 供应商信息卡
   - 数据字段表格
   - 脱敏样例展示
   - API 文档

4. **申请访问流程 (Access Request)**
   - 右侧滑出 Drawer
   - 4 步分步流程
     - Step 1: 选择套餐
     - Step 2: 填写用途
     - Step 3: 合规确认
     - Step 4: 提交审批
   - 幂等提交状态展示
   - Request ID / Chain Status / Projection Status

5. **核心组件**
   - Header / Footer
   - GlobalSearchBar
   - ProductCard
   - SupplierCard
   - ChainProofCard
   - LeftFilterPanel
   - SortToolbar
   - StickyTabs
   - RightStickyApplyPanel
   - AccessRequestDrawer

## 设计原则

### UI/UX

- **克制的动画**: 仅使用功能性动画（Fade-in 150ms, Slide-in 250ms）
- **深色金融科技风格**: 主色 #0F172A (Slate 900) + #2563EB (Blue 600)
- **状态色彩明确**: 成功(深绿)、警告(琥珀黄)、失败(砖红)
- **充足留白**: 模块间 py-16 或 py-24
- **悬浮效果**: 商品卡片仅增加轻微阴影，禁止整体上浮
- **按钮反馈**: 所有按钮必须有 active 状态

### 技术实现

- **SSG + ISR**: 首页采用静态生成 + 增量再生（10 分钟）
- **SSR**: 市场页和详情页采用服务端渲染
- **URL 驱动**: 筛选状态写入 URL，支持分享和刷新保持
- **骨架屏**: 数据加载使用骨架屏，避免全局 Loading
- **等宽字体**: Hash 和 Request ID 使用 font-mono

## 开发指南

### 安装依赖

```bash
cd apps/portal-web
pnpm install
```

### 启动开发服务器

```bash
pnpm dev
```

访问 http://localhost:3000

### 构建生产版本

```bash
pnpm build
pnpm start
```

### 类型检查

```bash
pnpm type-check
```

### Lint

```bash
pnpm lint
```

## 项目结构

```
apps/portal-web/
├── src/
│   ├── app/                    # Next.js App Router
│   │   ├── page.tsx           # 首页
│   │   ├── marketplace/       # 市场页
│   │   ├── products/[id]/     # 商品详情页
│   │   ├── layout.tsx         # 根布局
│   │   └── globals.css        # 全局样式
│   ├── components/            # 组件
│   │   ├── layout/           # 布局组件
│   │   ├── home/             # 首页组件
│   │   ├── marketplace/      # 市场页组件
│   │   └── product/          # 商品详情组件
│   ├── types/                # TypeScript 类型定义
│   └── lib/                  # 工具函数
├── public/                   # 静态资源
├── tailwind.config.ts       # Tailwind 配置
├── tsconfig.json            # TypeScript 配置
├── next.config.js           # Next.js 配置
└── package.json
```

## 核心页面路由

- `/` - 首页
- `/marketplace` - 数据市场
- `/marketplace?keyword=xxx&industry=finance` - 带筛选的市场页
- `/products/[id]` - 商品详情页
- `/suppliers/[id]` - 供应商主页（待实现）
- `/standard-flow` - 标准链路（待实现）
- `/trust-center` - 可信能力（待实现）

## Mock 数据

当前所有数据均为 Mock 数据，用于前端展示和交互验证。后续需要：

1. 创建 API Client SDK
2. 接入真实后端接口
3. 实现 TanStack Query 数据获取
4. 添加错误处理和重试逻辑

## 待实现功能

### Sprint 1 (当前)
- [x] 门户首页
- [x] 全局搜索
- [x] 行业分类
- [x] 推荐商品
- [x] 优质供应商
- [x] Marketplace 筛选
- [x] 商品详情页
- [x] 申请访问流程

### Sprint 2
- [ ] 供应商主页
- [ ] 标准链路页面
- [ ] 可信能力页面
- [ ] 帮助中心
- [ ] 登录/注册页面

### Sprint 3
- [ ] 买家控制台
- [ ] 供应商后台
- [ ] 平台运营后台

## 注意事项

1. **不要使用 AI 味的设计**: 避免花哨插画、弹跳动画、过度装饰
2. **专注数据信任**: 强调合规、脱敏、授权、审计、链上存证
3. **高效操作**: 减少点击次数，关键信息一目了然
4. **状态透明**: 链状态、投影状态、工作流状态必须清晰展示
5. **幂等安全**: 提交操作必须支持幂等，避免重复提交

## 贡献指南

1. 遵循现有代码风格
2. 组件必须有 TypeScript 类型
3. 使用 Tailwind CSS 工具类
4. 避免内联样式
5. 保持组件单一职责
6. 添加必要的注释

## License

Private - 数据交易平台项目
