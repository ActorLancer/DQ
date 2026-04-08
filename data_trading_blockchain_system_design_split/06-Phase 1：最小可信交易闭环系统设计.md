<!-- Source: data_trading_blockchain_system_design_v1.md | Section: 6. Phase 1：最小可信交易闭环系统设计 -->
[返回总览](./00-README.md)
# 6. Phase 1：最小可信交易闭环系统设计

## 6.1 阶段目标与范围

Phase 1 的唯一目标是构建一个真正可跑通的“最小可信交易闭环”。它必须支持卖方上架、买方下单、链上托管、链下密文交付、链上验真确认、争议处理、结算回退与双方评分。只要这条路径无法闭环，平台就不是数据交易平台，而只是一个目录或文件站。

**Phase 1 必做能力**

| **能力域** | **必须实现内容**                                     | **备注**                     |
|------------|------------------------------------------------------|------------------------------|
| 身份与准入 | 组织注册、用户注册、实名认证、链上身份绑定、角色授权 | 建议接入企业实名或内部 IAM。 |
| 数据目录   | 商品创建、版本管理、标签、样本说明、状态管理、搜索   | 支持审核流但不必过度复杂。   |
| 交易与托管 | 订单创建、价格锁定、买方保证金、卖方保证金、支付托管 | 建议链上托管。               |
| 交付与验真 | 密文上传、一次性下载令牌、Hash 校验、确认/拒收       | 支持下载回执和超时。         |
| 争议与裁决 | 申诉入口、证据上传、自动裁决规则、人工复核入口       | 自动裁决只处理可验证事项。   |
| 信誉与评分 | 交易后互评、加权评分、信誉快照、排序和限权           | 与保证金动态联动。           |
| 审计与报表 | 关键事件日志、订单轨迹、合约事件、运营视图           | 必须具备对账能力。           |

## 6.2 业务流程设计

6.  卖方完成组织认证后创建数据资产与商品，上传链下密文对象或预创建对象存储占位，并提交商品元数据、样本摘要与完整数据 Hash。

7.  平台进行基础审核。审核通过后，商品状态变更为 listed，链上 ProductRegistry 记录商品摘要、卖方 DID、价格、有效期和版本哈希。

8.  买方浏览目录并发起下单。系统冻结买方支付金额与买方保证金，订单进入 buyer_locked。

9.  卖方收到待交付通知后，在交付时限内上传或确认交付对象，并生成一次性下载令牌、密钥封装信息和交付回执；合约记录 delivery_commit_hash。

10. 买方下载密文并本地解密，重新计算 Hash 与链上 full_hash 比对。若一致则确认收货；若不一致或无法解密，则发起拒收或争议。

11. 若买方确认收货，则合约自动结算，货款转卖方，双方保证金按规则释放。若触发拒收，则进入争议流程，由自动裁决和人工复核两级处理。

12. 交易结束后，双方互评；信誉服务据此更新 buyer_score、seller_score 和后续保证金权重。

## 6.3 模块设计

**Phase 1 模块拆分**

| **模块**              | **职责**                                         | **关键输入/输出**                                            |
|-----------------------|--------------------------------------------------|--------------------------------------------------------------|
| IdentityService       | 组织、用户、角色、DID/证书绑定。                 | 输入：注册信息；输出：org_id、user_id、did_id。              |
| CatalogService        | 数据资产、版本、商品、标签、搜索、审核。         | 输入：商品元数据；输出：product_id、status。                 |
| OrderService          | 订单创建、价格快照、状态编排、幂等。             | 输入：product_id、buyer_id；输出：order_id。                 |
| EscrowContractAdapter | 与链上托管合约交互，写入订单、保证金、结算结果。 | 输入：order_id、amount；输出：tx_hash、chain_status。        |
| DeliveryService       | 链下密文上传、令牌签发、下载回执、Hash 验证。    | 输入：object_ref、key_ref；输出：delivery_id、receipt_hash。 |
| DisputeService        | 争议受理、证据归集、自动裁决、人工复核。         | 输入：case request、evidence；输出：decision。               |
| ReputationService     | 评分、信誉快照、黑名单与动态保证金。             | 输入：rating、order result；输出：score delta。              |
| AuditService          | 统一事件日志、追踪链上链下一致性、审计报表。     | 输入：domain events；输出：audit_event。                     |

## 6.4 智能合约设计（Phase 1）

Phase 1 建议至少拆分四类合约或链码：ProductRegistry、OrderEscrow、DisputeRegistry、ReputationRegistry。这样做的原因是产品发布、订单托管、争议处理和信誉更新的生命周期、升级频率和权限边界不同，混在一个合约中会导致升级困难和风险外溢。

**合约职责**

| **合约**           | **核心职责**                                           | **建议写链内容**                                                                  |
|--------------------|--------------------------------------------------------|-----------------------------------------------------------------------------------|
| ProductRegistry    | 维护商品摘要、版本哈希、卖方 DID、状态和有效期。       | product_id、asset_version_hash、price_snapshot、seller_did、status。              |
| OrderEscrow        | 维护订单状态、金额、保证金、交付承诺、验收结果、结算。 | order_id、buyer_did、seller_did、amount、deposits、delivery_commit_hash、status。 |
| DisputeRegistry    | 维护争议对象、证据摘要、裁决结果。                     | case_id、order_id、evidence_hashes、decision_code、penalty。                      |
| ReputationRegistry | 维护评分摘要、信誉增减事件、黑名单标记。               | subject_id、score_before、score_after、reason_code。                              |

## 6.5 订单状态机

**订单状态机**

<table>
<colgroup>
<col style="width: 100%" />
</colgroup>
<thead>
<tr class="header">
<th><p>created</p>
<p>-&gt; buyer_locked</p>
<p>-&gt; seller_delivering</p>
<p>-&gt; delivered</p>
<p>-&gt; accepted -&gt; settled -&gt; closed</p>
<p>-&gt; rejected -&gt; dispute_opened</p>
<p>-&gt; auto_resolved -&gt; settled/closed</p>
<p>-&gt; manual_resolved -&gt; settled/closed</p>
<p>-&gt; expired -&gt; closed</p></th>
</tr>
</thead>
<tbody>
</tbody>
</table>

状态机设计要求：第一，任何状态迁移都必须有明确触发源和时限；第二，同一订单不能出现并发的相互矛盾状态；第三，链下服务可以缓存状态，但以链上状态迁移事件为准；第四，所有迁移都必须具备幂等性，例如重复点击确认收货不能造成重复结算。

## 6.6 保证金、价格与信誉设计

参考第一篇论文的思想，平台应采用双边保证金机制：卖方在商品上架或首次售卖时缴纳卖方保证金，买方在订单创建时缴纳买方保证金。其设计目的不是惩罚本身，而是将作恶成本前置为经济约束，促使诚实履约成为最优策略。

**推荐公式（工程版）**

<table>
<colgroup>
<col style="width: 100%" />
</colgroup>
<thead>
<tr class="header">
<th><p>seller_deposit = price * (base_rate + risk_weight)</p>
<p>risk_weight = 0.02 * price_level + 0.02 * size_level + 0.03 * seller_risk_level</p>
<p>buyer_deposit = max(price * 0.02, price * (0.12 - 0.01 * buyer_credit_level))</p>
<p>reputation_next = alpha * reputation_old</p>
<p>+ beta * weighted_rating</p>
<p>+ gamma * dispute_bonus_or_penalty</p>
<p>+ delta * delivery_timeliness_score</p>
<p>where alpha + beta + gamma + delta = 1.0</p></th>
</tr>
</thead>
<tbody>
</tbody>
</table>

落地时不建议直接照搬论文中的复杂公式，而应采用可解释且易调参的工程公式。推荐先以配置中心管理 base_rate、price_level、size_level 和 buyer_credit_level，对不同业务线、不同商品类别采取不同参数。

**信誉输入项**

| **输入项** | **来源**       | **说明**                                       |
|------------|----------------|------------------------------------------------|
| 交易评价分 | 买方/卖方互评  | 买卖双方都可以给出 1~5 分，但需要权重。        |
| 履约时效分 | 订单时间戳     | 按是否按时交付、按时验收和是否多次催单计算。   |
| 争议结果   | DisputeService | 败诉方扣分，恶意投诉额外扣分。                 |
| 历史稳定性 | 过去 N 笔交易  | 波动小、投诉少、持续正向交易的主体信誉更稳定。 |
| 风控告警   | RiskEngine     | 异常下载、串谋评分、批量拒付等影响信誉上限。   |

## 6.7 链下存储与交付设计

链下交付必须解决四件事：对象如何加密、买方如何获得解密材料、平台如何知道买方是否已获取对象、争议时如何回溯交付事实。建议采用以下机制：卖方上传 AES-GCM 加密后的对象；对象密钥再用买方公钥封装；下载链接使用短期签名令牌；下载完成后由网关生成 receipt_hash 并写入链上或写入审计仓。

**交付关键对象**

| **对象**         | **作用**             | **必须字段**                                                          |
|------------------|----------------------|-----------------------------------------------------------------------|
| storage_object   | 对象存储中的密文文件 | object_id、bucket、path、content_type、size、enc_algo。               |
| delivery_ticket  | 买方下载凭证         | ticket_id、order_id、expire_at、download_limit、buyer_did。           |
| key_envelope     | 密钥封装对象         | envelope_id、order_id、key_cipher、recipient_did。                    |
| delivery_receipt | 下载回执             | receipt_id、order_id、download_at、client_fingerprint、receipt_hash。 |

## 6.8 争议处理设计

争议流程必须分为“自动可裁决争议”和“需要人工复核争议”两类。自动可裁决争议包括：超时未交付、交付 Hash 与链上承诺不一致、重复交付失败、下载令牌签发失败、卖方未按时响应。人工复核争议包括：数据质量与描述不符、业务指标缺失、样本误导、隐性限制未披露等。

**裁决矩阵（简版）**

| **争议类型** | **裁决依据**                      | **建议处理**                                   |
|--------------|-----------------------------------|------------------------------------------------|
| 超时未交付   | 是否超过 seller_delivery_deadline | 卖方败诉，退货款，扣卖方保证金一定比例。       |
| Hash 不一致  | 链上 full_hash 与买方重算哈希     | 卖方败诉，退货款，记录严重信誉扣减。           |
| 买方恶意拒收 | Hash 一致且下载回执存在           | 买方败诉，释放货款，扣买方保证金和信誉。       |
| 无法解密     | 密钥封装错误或对象损坏            | 卖方责任则退货款并扣分；平台责任则免赔并重试。 |
| 质量争议     | 人工审查样本、描述、合同条款      | 人工复核，不应由自动合约直接定性。             |

## 6.9 Phase 1 API 关键集合

**外部 API（摘要）**

| **方法** | **路径**                     | **说明**               |
|----------|------------------------------|------------------------|
| POST     | /api/v1/orgs/register        | 组织注册与资料提交。   |
| POST     | /api/v1/products             | 创建商品。             |
| POST     | /api/v1/products/{id}/submit | 提交审核。             |
| GET      | /api/v1/catalog/search       | 搜索商品目录。         |
| POST     | /api/v1/orders               | 创建订单。             |
| POST     | /api/v1/orders/{id}/lock     | 买方支付与保证金锁定。 |
| POST     | /api/v1/orders/{id}/deliver  | 卖方交付。             |
| POST     | /api/v1/orders/{id}/accept   | 买方确认收货。         |
| POST     | /api/v1/orders/{id}/reject   | 买方拒收。             |
| POST     | /api/v1/cases                | 创建争议。             |
| POST     | /api/v1/ratings              | 提交评分。             |
| GET      | /api/v1/audit/orders/{id}    | 查看订单审计轨迹。     |

## 6.10 Phase 1 交付件与验收

- 一个完整可访问的前端门户和至少一套卖方、买方工作台。

- 一条可验证的演示链路：商品创建 -\> 上架 -\> 下单 -\> 支付托管 -\> 交付 -\> 验真 -\> 结算 -\> 评分。

- 合约事件和后台订单状态一致性对账脚本。

- 至少 20 个核心接口的自动化测试、合约单元测试、对象存储与下载回执测试。

- 基础运营能力：商品审核、订单查询、争议处理、信誉查看、日志检索、报表导出。
