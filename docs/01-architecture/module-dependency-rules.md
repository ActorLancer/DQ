# platform-core 模块依赖规则（BOOT-014）

## 依赖方向

`api -> application -> domain -> infrastructure`

## 强制约束

1. 禁止循环依赖。
2. 禁止跨模块直接绕过 application 调用对方 infrastructure。
3. 状态推进必须经模块 application 层编排并落审计/outbox。
4. 共享类型放入 `packages/domain-types`，禁止重复定义同一主对象结构。

## 例外处理

- 例外必须在本文件登记并附任务编号与审批记录。
