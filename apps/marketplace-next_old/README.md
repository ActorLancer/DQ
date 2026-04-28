# marketplace-next

临时迁移前端（独立于 `portal-web` / `console-web`），用于先完成新一代交互与视觉设计，再分阶段迁移业务逻辑。

## Run

```bash
pnpm --filter @datab/marketplace-next dev
```

## Notes

- 对外 API 统一通过 `/api/platform/**` 同源代理到 `platform-core`。
- `NEXT_PUBLIC_MARKETPLACE_LIVE_DATA=1` 时尝试走真实接口；否则使用内置数据集保障 UI 迭代效率。
- 页面默认全路由拆分，不做单页堆叠。
