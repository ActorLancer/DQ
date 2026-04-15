<!-- Source: data_trading_blockchain_system_design_v1.md | Section: 12. API 设计、事件模型与消息总线 -->
[返回总览](./00-README.md)
# 12. API 设计、事件模型与消息总线

## 12.1 API 设计规范

- 所有写接口必须支持幂等键 Idempotency-Key。

- 所有敏感读接口必须记录审计事件和调用主体。

- 下载类接口不得直接暴露真实对象路径，必须通过短时签名令牌或网关代理下载。

- 合约相关接口必须返回 tx_hash、request_id 和业务状态映射，便于排障。

## 12.2 事件模型

**建议的领域事件**

| **事件名**             | **触发时机**     | **消费者**                                |
|------------------------|------------------|-------------------------------------------|
| ProductCreated         | 商品创建成功     | CatalogProjection, AuditService           |
| ProductListed          | 商品审核通过上架 | SearchIndexer, NotificationService        |
| OrderCreated           | 订单链上创建成功 | OrderProjection, AuditService             |
| BuyerLocked            | 买方托管成功     | SettlementProjection, NotificationService |
| DeliveryCommitted      | 卖方交付承诺成功 | DeliveryProjection, AuditService          |
| OrderAccepted          | 买方确认收货     | SettlementService, ReputationService      |
| OrderRejected          | 买方拒收         | DisputeService                            |
| CaseOpened             | 争议受理         | CaseProjection, NotificationService       |
| CaseResolved           | 争议裁决完成     | SettlementService, ReputationService      |
| TrainingRoundCommitted | 训练轮次提交     | ContributionEngine, AuditService          |
| CrossChainAcked        | 跨链接收回执     | CrossChainProjection, RiskEngine          |
| RiskAlertRaised        | 风险告警生成     | RiskConsole, SupervisorView               |

## 12.3 消息总线与异步任务

系统需要消息总线来解耦链事件投影、通知、索引、风控和报表任务。推荐至少规划以下主题：domain-events、chain-events、audit-events、risk-events、training-events、cross-chain-events。消息消费必须具备重试、死信队列和告警。

**事件消息样例**

<table>
<colgroup>
<col style="width: 100%" />
</colgroup>
<thead>
<tr class="header">
<th><p>{</p>
<p>"event_id": "evt-20260407-00001",</p>
<p>"event_type": "OrderAccepted",</p>
<p>"source": "order-service",</p>
<p>"ref_id": "ORD-2026-00088",</p>
<p>"actor_id": "USR-1001",</p>
<p>"tx_hash": "0xabc123",</p>
<p>"occurred_at": "2026-04-07T10:00:00Z",</p>
<p>"payload": {</p>
<p>"buyer_org_id": "ORG-B",</p>
<p>"seller_org_id": "ORG-S",</p>
<p>"amount": 12000</p>
<p>}</p>
<p>}</p></th>
</tr>
</thead>
<tbody>
</tbody>
</table>
