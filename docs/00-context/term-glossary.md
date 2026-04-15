# 术语表（CTX-016）

## 1. 核心实体术语

- `Tenant`：租户边界，承载组织级配置、隔离与计费边界。
- `Party`：交易主体，通常对应企业、机构或其法定业务实体。
- `Application`：接入平台 API 的客户端应用标识与凭据容器。
- `Connector`：对接外部系统或数据源的适配连接器。
- `ExecutionEnvironment`：任务执行环境（沙箱/受控运行环境）的逻辑标识。
- `DataResource`：原始或加工后可被产品化的数据资源单元。
- `DataProduct`：面向交易的商品化对象，含定价与交付策略。
- `SKU`：标准交易类型与套餐实例的最小交易单元。
- `Authorization`：授权对象，描述可访问/可使用权利及有效期边界。
- `QuerySurface`：可开放查询的数据视图或执行面定义。
- `QueryTemplate`：参数化查询模板，绑定输入约束与输出边界。

## 2. 生命周期术语

- `Order`：订单主对象，承载交易主状态推进。
- `DigitalContract`：数字合同对象，记录条款快照与签约状态。
- `Delivery`：交付对象，记录交付动作、回执与可验证结果。
- `Settlement`：结算对象，记录账务归集与清结算状态。
- `Dispute`：争议对象，记录申诉、证据与裁决结果。
- `BillingEvent`：账单事件对象，记录计费触发事实。

## 3. 一致性与证据术语

- `OutboxEvent`：事务内写入的待发布事件记录，保障主状态与事件一致。
- `ExternalFact`：外部系统回执事实（支付、链回执、存储回执等）。
- `ProofCommit`：链上摘要提交动作与状态。
- `Reconcile`：一致性修复或对账动作。
- `EvidencePackage`：用于审计/监管的证据导出包。
