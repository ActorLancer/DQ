# fixtures 目录校准（BOOT-036）

`fixtures/` 用于统一演示数据、联调样本和测试夹具落位。

当前结构：

- `fixtures/demo/`
- `fixtures/local/`

说明：

- `fixtures/demo/`：`TEST-001` 起的正式 demo 数据包入口，承接五条标准链路、8 个标准 SKU 的演示主体、官方展示商品、订单蓝图、交付/账单/审计样例与校验入口。
- `fixtures/local/`：ENV 阶段遗留的本地 bootstrap / smoke / manifest 样例，不再等同于完整 demo 数据包。

后续可按任务扩展 `staging/demo` 子目录。
