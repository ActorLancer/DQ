# 策略决策接口与 PDP 设计

- 文档名称：策略决策接口与 PDP 设计
- 适用范围：基于区块链技术的数据交易平台 `V1 / V2 / V3`
- 文档定位：授权决策与策略运行时基线，不等于最终部署方案

---

## 1. 目标

本方案用于解决以下问题：

1. 平台的“最终是否放行”由谁判断
2. `RBAC + Scope + 合同 + Usage Policy + 风控` 怎么组织成统一决策
3. `V1` 是否需要独立策略引擎
4. `V2/V3` 如果引入 `OPA`，怎么接才不破坏现有架构

---

## 2. 当前冻结的策略判断链

现有文档中，后端放行链已经确定为：

1. 身份解析
2. 主体状态
3. 基础权限
4. 作用域
5. 业务对象状态
6. 合同 / `Usage Policy`
7. 环境 / 网络 / 配额
8. 合规 / 风控
9. 高风险附加校验
10. 审计

因此，本平台的策略系统不是单纯“角色判断”，而是：

**混合授权决策系统**

包含：

- `RBAC`
- `ABAC`
- `PBAC`
- `Scope`
- 合同快照
- `Usage Policy`
- 风控阻断
- 高风险 step-up

---

## 3. 基本概念

## 3.1 PEP

`Policy Enforcement Point`

作用：

- 在网关、BFF、业务服务、异步消费者处拦截请求
- 负责把当前请求转成统一输入
- 调用决策逻辑
- 根据结果执行 allow / deny / challenge / mask / dry-run

## 3.2 PDP

`Policy Decision Point`

作用：

- 接收标准化输入
- 综合角色、作用域、策略、风控、对象状态做决策
- 返回决策结果与原因

## 3.3 PIP

`Policy Information Point`

作用：

- 为 PDP 提供决策数据
- 典型数据包括：
  - 用户与主体状态
  - 角色与权限点
  - 作用域
  - 合同
  - `Usage Policy`
  - 风控命中
  - step-up 状态

---

## 4. 推荐路线

## 4.1 V1

`V1` 不建议一开始就引入重型外部策略平台。  
推荐：

- `PEP` 在平台应用层
- `PDP` 先以内嵌服务/共享库实现
- `PIP` 数据来自平台数据库与缓存

即：

**V1 = 应用内 PDP**

优点：

- 与现有数据库模型完全一致
- 不会过早引入第二套策略主数据
- 更容易和合同、Usage Policy、风控、审计联动

## 4.2 V2

`V2` 可以开始外置部分策略运行时，例如：

- 地域限制
- 用途限制
- 输出限制
- 监管例外规则
- 跨平台互认策略

此时可评估：

- `OPA` 作为独立 PDP 运行时

但前提是：

- 业务语义已经先在 `V1` 固化
- 外置的是“决策执行”，不是“重新发明授权模型”

## 4.3 V3

`V3` 再逐步扩展：

- 跨组织 / 跨平台策略联动
- partner-scope 策略
- 更复杂的监管特例
- 更复杂的图风控冻结策略

---

## 5. V1 统一决策输入

建议所有策略判断都收敛为统一输入模型：

```json
{
  "request": {
    "request_id": "uuid",
    "action": "delivery.download",
    "resource_type": "order",
    "resource_id": "uuid",
    "occurred_at": "2026-04-09T12:00:00Z",
    "channel": "web"
  },
  "actor": {
    "actor_type": "user",
    "actor_id": "uuid",
    "tenant_id": "uuid",
    "role_set": ["tenant_admin"],
    "scope_snapshot": {},
    "auth_context_level": "aal2",
    "step_up_valid": true
  },
  "resource": {
    "status": "active",
    "owner_tenant_id": "uuid",
    "risk_status": "normal",
    "data_level": "high"
  },
  "policy": {
    "contract_status": "effective",
    "usage_policy_status": "effective",
    "allowed_regions": ["SG"],
    "allowed_purposes": ["analysis"],
    "exportable": false
  },
  "environment": {
    "source_ip": "x.x.x.x",
    "country_code": "SG",
    "network_zone": "public",
    "app_id": "uuid"
  },
  "risk": {
    "hit_flags": [],
    "subject_frozen": false,
    "product_frozen": false
  }
}
```

---

## 6. V1 统一决策输出

建议 PDP 返回统一结构：

```json
{
  "decision": "allow",
  "effect": "full",
  "reason_codes": [],
  "obligations": [],
  "audit_required": true,
  "step_up_required": false,
  "masking_required": false,
  "dry_run_only": false,
  "decision_version": "v1.0"
}
```

其中：

- `decision`
  - `allow`
  - `deny`
  - `challenge`
  - `allow_with_mask`
  - `dry_run`
- `reason_codes`
  - 如 `PERM_SCOPE_MISMATCH`
  - `POLICY_REGION_BLOCKED`
  - `RISK_SUBJECT_FROZEN`
  - `AUTH_STEP_UP_REQUIRED`
- `obligations`
  - 需要审计
  - 需要只读导出
  - 需要脱敏
  - 需要短时令牌

---

## 7. V1 决策实现方式

## 7.1 应用内 PDP 职责

应用内 `PDP` 负责：

- 聚合请求上下文
- 读取角色与权限点
- 读取作用域快照
- 读取对象状态
- 读取合同与 `Usage Policy`
- 读取风控标记
- 给出最终决策

## 7.2 放行优先级

推荐优先级：

1. 身份无效，直接拒绝
2. 主体冻结，直接拒绝
3. 无基础权限，直接拒绝
4. 作用域不匹配，直接拒绝
5. 合同/策略未生效，直接拒绝
6. 风控阻断，直接拒绝
7. 高风险但未 step-up，返回 `challenge`
8. 满足全部条件，允许

## 7.3 不可下放给 IAM 的策略

以下内容不建议放进 IAM 产品本身：

- 合同状态判断
- Usage Policy 的用途与导出限制
- 数据等级与高敏规则
- 风控冻结
- 结果导出策略
- 支付、托管、结算相关业务限制

原因：

- 这些都是高变化业务规则
- 它们的主数据本来就在平台数据库

---

## 8. V2 引入 OPA 的方式

## 8.1 正确定位

如果 `V2` 引入 `OPA`，其定位应是：

- 独立 `PDP` 运行时
- 策略执行器
- policy-as-code 载体

而不是：

- 业务主数据源
- 合同主库
- 权限主库

## 8.2 推荐接法

推荐模式：

```text
PEP(业务服务)
  -> 组装统一决策输入
  -> 调用 PDP
     -> V1: 本地内嵌 PDP
     -> V2: OPA
  -> 接收 decision
  -> 执行 allow/deny/challenge
  -> 写审计
```

## 8.3 OPA 数据边界

即使使用 `OPA`，下列数据仍来自平台：

- 角色和权限点
- 作用域绑定
- 合同快照
- `Usage Policy`
- 风控状态
- step-up 状态

`OPA` 使用这些输入做决策，不拥有这些数据的来源权威。

## 8.4 外置后的附加要求

如果引入外置 PDP，必须补齐：

- policy input schema
- decision output schema
- policy bundle 版本号
- decision log 审计
- 缓存策略
- 降级策略
- 失败时 fail-closed / fail-open 规则

推荐：

- 高风险与写接口：`fail-closed`
- 低风险只读且可脱敏接口：可按策略允许有限降级

---

## 9. 审计要求

每次 PDP 决策至少应记录：

- `request_id`
- `actor_id`
- `permission_code`
- `scope_snapshot`
- `resource_type`
- `resource_id`
- `decision`
- `reason_codes`
- `policy_version`
- `step_up_required`
- `risk_flags`

如果外置到 `OPA`，还应记录：

- `pdp_engine`
- `bundle_version`
- `decision_latency_ms`

---

## 10. 与现有文档体系的关系

本文件不会改变当前已经冻结的需求，只是把实现路线收紧为：

- `V1`：应用内 PDP
- `V2`：如有必要，再引入 `OPA`

因此它与现有文档的关系是：

- 不改角色模型
- 不改权限点模型
- 不改 Usage Policy 模型
- 不改数据库主数据归属

只补足：

- 决策输入
- 决策输出
- PDP/PEP/PIP 边界
- 后续外置策略引擎的替换方式

---

## 11. 当前结论

当前最合理的路线是：

- `V1` 不上重型外部策略平台
- `V1` 先在应用层完成统一 PDP
- `V2` 再评估 `OPA`
- 即使引入 `OPA`，也只把它当策略执行器，不把它当业务授权主库

这与当前文档体系是兼容且闭环的。

---

## 12. 参考资料

- Open Policy Agent Documentation  
  https://www.openpolicyagent.org/docs/latest/
- OPA Policy Language  
  https://www.openpolicyagent.org/docs/policy-language

