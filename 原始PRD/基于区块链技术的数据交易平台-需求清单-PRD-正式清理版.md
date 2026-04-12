# 基于区块链技术的数据交易平台需求清单（PRD-R4）

- 文档版本：R4
- 文档日期：2026-04-11
- 文档定位：用于产品范围确认、后续 PRD/原型/技术方案/合规设计的上位输入
- 参考输入：用户提供《核心交易链路》、国内数据要素与可信数据空间政策、上海/深圳数据交易监管规则、欧盟数据空间与数据流通规则、Ocean / IDS / Gaia-X / Catena-X 等行业实践，以及本轮补充的 Solana 公链接入展示需求
- 关联专题稿：
  - `原始PRD/数据本体存储与信任边界设计.md`
  - `原始PRD/数据商品存储与分层存储设计.md`
  - `原始PRD/数据商品查询与执行面设计.md`
  - `原始PRD/敏感数据处理与受控交付设计.md`
  - `原始PRD/数据对象产品族与交付模式增强设计.md`
  - `原始PRD/数据商品元信息与数据契约设计.md`
  - `原始PRD/数据原样处理与产品化加工流程设计.md`
  - `原始PRD/策略决策接口与 PDP 设计.md`
  - `原始PRD/合规审查与风控规则详细稿.md`
  - `原始PRD/交易链监控、公平性与信任安全设计.md`
  - `原始PRD/角色与权限矩阵详细稿.md`
  - `原始PRD/链上链下技术架构与能力边界稿.md`
  - `原始PRD/日志、可观测性与告警设计.md`
  - `原始PRD/商品搜索、排序与索引同步设计.md`
  - `原始PRD/商品推荐与个性化发现设计.md`
  - `原始PRD/身份认证、注册登录与会话管理设计.md`
  - `原始PRD/审计、证据链与回放设计.md`
  - `原始PRD/双层权威模型与链上链下一致性设计.md`
  - `原始PRD/支付、资金流与轻结算设计.md`
  - `原始PRD/IAM 技术接入方案.md`

---

## 1. 文档目标

本文件用于回答五个问题：

1. 这个平台到底要做成什么。
2. 它解决的核心问题是什么。
3. 它为什么必须依托区块链，而不是普通数据中台或普通电商平台。
4. 第一阶段应该做哪些能力，哪些先不做。
5. 为了不阻塞后续设计与开发，需求阶段必须提前明确哪些规则、边界、实体、权限和交易模式。

> 本文件是“需求确定阶段”的完整清单，不是 UI 级详细交互稿，也不是数据库表设计或微服务拆分方案。

---

## 2. 产品一句话定义

**该平台不是“把原始数据搬到链上卖”的网站，而是一套以联盟链为可信交易底座、以 Solana 等公链为公开可验证展示与外部增信层，以数字身份、数字合约、合规审查、受控交付、隐私计算与全程审计为核心能力的“可信数据流通与交易基础设施”。**

平台的本质是把“数据资源”转化为：

- 可登记
- 可发现
- 可定价
- 可授权
- 可交付
- 可计算
- 可结算
- 可追责
- 可监管

的数据产品、数据服务与数据使用权。

---

## 3. 产品背景

### 3.1 市场与制度背景

当前数据流通的主要矛盾不是“有没有数据”，而是：

- 权利边界不清，谁可以卖、卖什么、卖到什么程度不明确
- 买方难以判断数据质量、真实性和可用性
- 交易一旦交付副本，后续用途难控制
- 涉及个人信息、商业秘密、重要数据时，合规与安全成本极高
- 跨组织、跨行业、跨地域的互信不足
- 缺少全流程证据链，争议处理成本高
- 多方参与场景下分账、审计、监管协同复杂

在国内制度层面，平台设计必须面向“数据产权、流通交易、收益分配、安全治理”四大制度方向；同时要适配可信数据空间、数据流通交易标准示范合同、场内外结合的数据要素流通体系等政策方向。
在国际参考层面，成熟路线也不是“原始数据上链”，而是“链上身份/授权/计费/审计，链下数据/连接器/安全空间/隐私计算”；如需引入公链，更适合作为公开验证、品牌展示、生态接口和轻量凭证层，而不是承载核心敏感交易数据。

### 3.2 为什么要用区块链

区块链不是为了“存大数据”，而是为了解决以下可信问题：

1. **多方不信任**
   - 交易双方、平台、监管、第三方服务机构之间需要共同认可的事件记录。
2. **授权难执行**
   - 需要把用途、期限、地域、可导出性、可训练性、可转授权性等约束写成数字合约。
3. **审计难留痕**
   - 需要对关键交易事件做不可抵赖、可核验、可追溯的存证。
4. **结算与分润复杂**
   - 需要在多方参与的持续型交易中自动触发分账、对账和结算。
5. **争议难举证**
   - 需要保留合同版本、授权版本、交付回执、调用摘要、计算结果摘要、验收记录等证据链。

### 3.3 为什么不能把所有东西都上链

平台必须采用**“联盟链主链路 + 公链展示锚点 + 链下数据 + 连接器/沙箱/隐私计算”**的混合架构，原因如下：

- 原始数据量大，链上存储成本和性能不可接受
- 原始数据上链不利于隐私与商业秘密保护
- 法律合规要求下，许多数据只能受控使用，不能直接转移副本
- 大多数高价值场景需要“数据不出域、算法进场、结果受控返回”

---

## 4. 产品定位与目标

### 4.1 产品定位

平台优先定位为：

**面向 B2B / B2G / G2B 的可信数据交易与数据协作平台**

而不是：

- 面向普通消费者的“个人信息买卖平台”
- 面向公开链代币投机的“数据资产上币平台”
- 只做文件下载分发的简单目录网站

### 4.2 核心产品目标

#### 目标一：让交易成立
平台必须完成完整交易闭环：

- 上架
- 搜索/撮合
- 询价/竞价
- 审核/审批
- 合同签署
- 受控交付或受控计算
- 结算分账
- 售后争议
- 全程审计

#### 目标二：让交易可信
平台必须解决：

- 主体可信
- 数据来源可信
- 授权可信
- 使用可信
- 日志可信
- 结算可信
- 审计可信

#### 目标三：让交易可持续规模化
平台必须具备：

- 数据标准化
- 目录化与产品化
- 质量评估
- 合规评估
- 权利模型
- 使用控制
- 风险控制
- 监管接口
- 生态服务

#### 目标四：让“数据不出库但能用”成为主流交易方式
平台不能只支持卖数据副本，还必须支持：

- API 访问与订阅式交付
- 查询权与结果获取权
- 数据空间内受控共享
- 模型调用与结果服务
- `V2/V3` 的计算权、联邦协作、安全多方计算、`ZKP` 证明服务

### 4.3 北极星指标（需求阶段建议）

- 有效上架数据产品数
- 完成合规审查的数据产品占比
- 成交转化率
- “非副本交付”交易占比（API / 只读共享 / 模板查询 / 查询沙箱 / 结果产品 / V2 受控计算）
- 争议率与争议解决时长
- 高风险交易人工复核命中率
- 审计追溯完成率
- 买方复购率 / 续费率
- 数据供方留存率
- 多方分账自动完成率

---

## 5. 范围定义

本节用于把“战略全景”收敛为“研发可拆解范围”。本项目后续一切版本规划、架构评审、团队配置、排期预算，均以本节为优先边界。

## 5.1 V1 范围锁定原则

1. **先做可规模复制的标准交易，不先做最复杂的隐私计算交易。**
2. **先锁定 2 个行业、5 条标准链路，不做跨行业泛化平台。**
3. **先把文件快照/版本订阅、只读共享、API/服务、模板查询 lite、查询沙箱、结果产品跑通，不把 C2D/FL/MPC/TEE 作为 V1 交付承诺。**
4. **Solana 公链能力在 V1 仅允许作为可关闭的技术展示/内测能力，不进入生产主链路依赖。**
5. **自然人供给、跨境自动流转、金融/医疗高敏副本交易，不进入 V1。**

## 5.2 V1 Must / Should / Won't 锁定表

| 分类 | Must（V1 必做） | Should（V1 可选，默认关） | Won't（V1 不做） |
|---|---|---|---|
| 主体与账户 | 租户、企业主体、部门、用户、应用、连接器、执行环境建模；KYC/KYB；黑白灰名单；角色权限；企业 OIDC SSO；MFA；会话与设备治理；Fabric 身份绑定与证书治理 | `SAML 2.0`、`SCIM`、二维码授权、企业外部 IdP 深度联邦增强 | 自然人自由供给市场 |
| 行业与场景 | 锁定工业制造/供应链、零售/本地生活两大行业；5 条标准链路 | 第三行业试点 | 泛行业开放平台 |
| 产品类型 | 文件快照/版本订阅、只读共享、API/服务、模板查询 lite、查询沙箱、固定报告/结果产品 | 组合包销售、卖方自持共享增强 | C2D、FL、MPC、TEE、ZKP 正式交付 |
| 权利模型 | 使用权、访问权、查询权、结果获取权、有限内部共享权 5 类；标准 SKU 模板；合同模板/授权模板/验收模板/退款模板/计费模板绑定 | 行业定制附加条款模板 | 所有权转让、无限期转授权 |
| 交易流程 | 上架、报价、合同、支付锁定、交付、验收、结算、争议、审计 | RFQ 批量询价 | 拍卖、撮合撮单算法市场 |
| 交付控制 | 文件下载令牌、水印、API Key、配额、沙箱账号、到期断权 | 行级/列级细粒度脱敏策略扩展 | 通用计算入域平台 |
| 链与存证 | 联盟链登记、合同摘要、交付回执、账单摘要、审计摘要 | 公链锚定内测 | 以公链作为核心订单状态机 |
| 结算财务 | 一次性结算、周期账单、用量账单、发票申请、人工结算/半自动分账 | 预付+后付混合账期、自动分润 | 数字资产/代币化清结算 |
| 合规与风控 | 自动规则初审 + 人工复核；高风险阻断；证据包导出 | 风险评分模型 | 无规则的纯人工审核 |
| 公链展示 | 无 | 联盟链事件批次哈希同步到 Solana 测试网/演示环境 | 生产环境默认公开展示交易摘要、NFT 大规模签发 |

## 5.2A 架构图与模块清单的生命周期标注规则

为避免“架构图画出来了就被误认为 V1 必做”，自本版起所有架构图、模块清单、页面规划和技术方案都必须显式标注以下状态之一：

- `[V1 Active]`：V1 必须建设并可上线验收的能力
- `[V2 Reserved]`：V2 预留，占位但不构成 V1 交付承诺
- `[V3 Reserved]`：V3 预留，占位但不构成 V1/V2 交付承诺

最低要求：

- 文件快照、版本订阅、只读共享、API、模板查询 lite、查询沙箱、结果产品、审计、支付、认证、搜索、推荐、双层权威一致性、交易链监控必须标为 `[V1 Active]`
- `C2D / FL / MPC / TEE / ZKP / 完整 clean room / 复杂共享连接器框架` 必须标为 `[V2 Reserved]` 或 `[V3 Reserved]`
- 预留模块允许出现在总览图中，但不得据此要求 V1 一次性完成服务拆分、数据建模、前端页面和联调验收

## 5.3 首批行业与标准交易场景锁定

### 5.3.1 首批行业

**行业 A：工业制造 / 供应链协同**
- 选择原因：数据价值明确、企业主体清晰、以 API/报表/查询类产品为主，合规复杂度相对可控。
- 典型供方：设备厂商、工业 SaaS、MES/ERP 服务商、物流协同方。
- 典型买方：制造企业、供应链协同企业、设备运维服务商。

**行业 B：零售 / 本地生活经营分析**
- 选择原因：交易需求高频、产品标准化程度较高，适合先做目录、报价、订阅和查询沙箱。
- 典型供方：零售数据服务商、商圈分析服务商、平台型服务商。
- 典型买方：连锁品牌、代理运营商、选址与经营分析团队。

### 5.3.2 首批 5 条标准链路

1. **工业设备运行指标 API 订阅**
   卖方提供设备运行状态/稼动率/能耗 API，买方按月订阅并按调用量追加计费。
2. **工业质量与产线日报文件包交付**
   卖方按周/月交付低敏质量汇总文件，买方一次性采购并下载验收。
3. **供应链协同查询沙箱**
   上下游企业在平台沙箱中查询订单履约、库存周转、补货建议，不导出原始明细。
4. **零售门店经营分析 API / 报告订阅**
   提供门店客流、转化、销售结构等聚合指标，支持 API 或月报服务。
5. **商圈/门店选址查询服务**
   买方按次发起查询，在沙箱中查看候选区域画像与评分，导出受限结果。

### 5.3.3 场景排除项
- 医疗明细、金融明细、涉及敏感个人信息原始副本交易。
- 跨境自动交付。
- 多机构联邦训练/多方安全计算正式生产交付。

## 5.4 V2 范围（扩展能力层）

V2 在 V1 跑稳后进入，重点补齐“可用不可见”能力：
- C2D / 入域计算
- 联邦学习任务管理
- MPC 任务编排
- TEE 远程证明接入
- ZKP 结果证明接入
- 结果级授权与贡献度分润
- Solana 公链生产级锚定与公开验证页
- 不可转让交易凭证 / 供应商认证凭证 / 数据产品护照 NFT

V2 还必须把 V1 已冻结但仅按最小路径落地的数据对象能力补齐为正式能力：
- 只读共享从“单后端 / 单协议 / 单连接器试点”升级为“可治理的共享连接器框架”
- 模板查询从 `lite` 升级为完整 `clean room / analysis rule / TVF / output policy`
- 版本订阅从固定周期订阅升级为更完整的 revision 发布、历史版本授权和周期治理
- 文件快照、只读共享、API、模板查询、结果产品的差异化授权/验收/退款/赔付规则，从最小模板升级为完整规则体系

## 5.5 V3 范围（可信数据空间与生态扩展）

- 跨平台连接器互联
- 跨区域/跨联盟链协同
- 数据空间互认
- 跨境受控流通工作台
- 数据资产登记、融资、质押辅助服务

V3 必须彻底完成前两阶段未完全收口的跨平台能力：
- 跨平台只读共享与 revoke/回执/证据联动
- 跨平台 revision 订阅、对象授权和责任边界快照
- 跨平台 clean room / 受控执行 / 结果证明协同
- 跨域争议、审计回放和责任链穿透

## 5.6 暂不建议进入前两阶段的范围

- 面向大众自然人的 C2C 个人数据买卖
- 公链代币化发行作为核心业务模式
- 原始高敏数据副本公开交易
- 以公链钱包地址替代企业实名与审计链路


## 6. 关键产品原则

1. **不以副本转移为唯一交易方式**
2. **链上不放原始数据，只放凭证、状态与摘要**
3. **先合规，再交易，再使用，再持续监管**

### 6.1 数据本体存储与信任边界要求

#### 6.1.1 总体原则

1. 平台交易的是“可验证的使用权、访问权、查询权、结果获取权，以及 `V2/V3` 的计算权扩展”，不等于默认交易明文副本。
2. 平台必须支持 `平台托管`、`卖方自持`、`受控执行`、`买方拉取`、`第三方可信存储` 五类存储/交付模式。
3. 平台必须统一治理元数据、版本、哈希、授权、交付、调用、审计和生命周期，但不要求默认掌握原始明文。
4. 原始数据、密文大对象、模型权重、训练语料、证据原文默认不直接落 PostgreSQL，数据库保存元数据、路径、哈希、状态、索引和审计引用。
5. 平台信任的核心来自最小明文接触、密钥分离、受控交付、受控执行和全量证据链，而不是单靠链上或 ZKP。

#### 6.1.2 必须支持的存储模式

- 平台托管：平台保存密文对象或受控对象，交付通过票据、令牌和密钥封装完成。
- 卖方自持：平台仅保存元数据、哈希、连接方式和策略，交易后由卖方侧连接器/API 网关完成交付。
- 受控执行：数据保留在卖方或专用执行环境，买方获取的是结果、查询或模型输出，而不是原始底层数据。
- 买方拉取：平台签发访问权、配额和有效期，买方按授权从卖方侧或平台代理层拉取数据。
- 第三方可信存储：平台记录第三方托管关系、责任边界和信任证据。

#### 6.1.3 平台必须显式记录的字段

每个资产或资产版本至少记录：

- `storage_mode`
- `payload_location_type`
- `custody_mode`
- `key_control_mode`
- `platform_plaintext_access`
- `platform_unilateral_decrypt_allowed`
- `default_delivery_route`
- `requires_controlled_execution`
- `retention_policy`
- `destroy_policy`

每个订单和交付至少记录：

- `storage_mode_snapshot`
- `delivery_route_snapshot`
- `trust_boundary_snapshot`
- `executor_type`
- `source_binding_id`
- `receipt_hash`

#### 6.1.4 平台信任机制

1. 卖方不信任平台时，平台必须允许“卖方自持 + 平台只管授权与审计”的正式模式，而不是把它当例外。
2. 平台托管时，数据与密钥必须分离，平台不应同时独占数据对象和长期有效解密能力。
3. 高敏资产默认优先走 API、沙箱、受控查询、受控执行或结果交付，不鼓励明文副本下载。
4. 所有下载、解密、导出、调用、结果交付、吊销、销毁必须进入证据链并支持监管与争议回放。
5. ZKP 可用于局部断言证明，但不能作为“平台从未查看/使用过数据”的唯一信任基础。

#### 6.1.5 生命周期治理要求

平台追踪的重点是：

- 资产生命周期
- 版本生命周期
- 权利生命周期
- 交付生命周期
- 凭证生命周期
- 风险生命周期
- 审计生命周期
- 存储生命周期

生命周期治理的核心不是无限期托管明文，而是保证：

- 状态可追踪
- 授权可撤销
- 交付可验证
- 调用可审计
- 过期可断权
- 保留与销毁可证明

### 6.2 支付、资金流与轻结算要求

完整设计见：[《支付、资金流与轻结算设计》](./支付、资金流与轻结算设计.md)

平台必须新增独立的支付、资金流与清结算子系统，至少遵循以下原则：

- 交易域与支付域解耦，订单模块不得直接绑定具体支付渠道 SDK。
- 平台维护业务主状态、账务镜像、结算指令和审计证据；真实资金优先由外部支付/银行/托管体系承接。
- 平台起步司法辖区固定为新加坡；`V1` 生产只开放新加坡白名单内的真实支付与真实结算走廊。
- 平台通过“司法辖区配置 + 走廊策略 + 结算路由 + 持牌伙伴接入”实现全球扩展，不以技术中间件替代牌照要求。
- 平台必须支持支付宝、微信支付、银联、PayPal、线下对公转账和 `MockPaymentProvider`。
- 平台收费规则必须版本化、参数化，支持平台服务费、渠道手续费、保证金规则、增值服务费和分润规则。
- 货款、保证金、退款、赔付、打款、分润、税票等资金流必须结构化建模。
- 支付成功、退款成功、打款成功、对账差异处理等动作必须全量审计、可追责、可重放。
- `V1` 必须支持真实支付适配层抽象、支付意图、回调幂等、基础对账与 Mock 支付；`V2/V3` 再扩展自动分润、多币种与数字资产结算。

### 6.3 全局产品原则补充

1. 先做 `B2B / B2G` 的高价值、可落地场景。
2. 高敏感数据默认采用“可用不可见”。
3. 角色权限不是静态的，必须与身份、数据级别、场景绑定。
4. 平台不直接替代法律判断，但必须把法律规则前置成系统规则。
5. 默认保留完整证据链。
6. 默认支持平台外部监管与第三方审计。
7. 先联盟链、后跨链；先受许可网络、后开放生态。

---

## 7. 产品要解决的核心问题

## 7.1 交易怎么完成
- 上架
- 撮合
- 交易
- 交付
- 结算
- 风控
- 存证

## 7.2 交易为什么能成立
- 数据治理
- 确权与授权管理
- 目录与标准化
- 质量评估
- 合规审查
- 合同管理
- 使用控制
- 争议处理
- 监管协同
- 生态服务

## 7.3 平台的核心卖点

### 卖点 1：可验证授权
把用途、期限、部门、地域、算法类型、输出限制等写成数字合约，并在链上保留签署与执行证据。

### 卖点 2：可用不可见
通过沙箱、连接器、MPC、TEE、FL、ZKP 等技术，让买方获得“使用能力”而不是“无限复制权”。

### 卖点 3：全流程可信追踪
从注册、上架、审核、签约、交付、调用、结算到争议，全程可追溯、可审计、可复核。

### 卖点 4：多形态产品交易
既支持数据副本、也支持 API、查询、订阅、模型、算法，以及 `V2/V3` 的计算权和联合任务。

### 卖点 5：合规即产品能力
合规不是线下补材料，而是平台内建的规则引擎、审查流、使用控制和证据链。

---

## 8. 总体架构原则（链上链下分工）

```text
┌──────────────────────────────────────────────┐
│         应用层 / 门户层 [V1 Active + V2 Reserved] │
│ [V1 Active] 目录搜索 | 上架中心 | 询报价 | 订单 | 合同 | 工单 │
│ [V2 Reserved] 公开验证页 | 凭证展示页 | 数据产品护照页       │
└──────────────────────────────────────────────┘
                    │
┌──────────────────────────────────────────────┐
│      交易控制层 / 规则执行层 [V1 Active + V2 Reserved] │
│ [V1 Active] 身份认证 | 权限策略 | 合规审查 | 风控 | 定价 | 结算 │
│ [V1 Active] 数字合约 | 审批流 | 使用控制 | 审计编排 | 争议处理 │
│ [V2 Reserved] 公链披露策略 | 公示脱敏规则 | 凭证发行编排       │
└──────────────────────────────────────────────┘
                    │
┌──────────────────────────────────────────────┐
│      执行层 / 数据与计算交付层 [V1 Active + V2 Reserved] │
│ [V1 Active] 文件交付 | API 网关 | 查询沙箱 | 连接器网络 │
│ [V2 Reserved] 安全空间 | C2D | MPC | FL | ZKP 校验 | TEE 证明 │
│ [V1 Active] 版本管理 | 数据血缘 | POC 测试环境 | SLA 监测   │
└──────────────────────────────────────────────┘
                    │
┌──────────────────────────────────────────────┐
│      信任层 / 联盟链 / 存证与清算层 [V1 Active]      │
│ DID/VC | 合约哈希 | 授权状态 | 订单状态 | 回执 |   │
│ 账单摘要 | 分润结果 | 证据摘要 | 监管观察节点     │
│ 冻结标记 | 下架标记 | 审批结果 | 日志批次         │
└──────────────────────────────────────────────┘
                    │
┌──────────────────────────────────────────────┐
│    公链展示层 / Solana 公示锚点与凭证层 [V2 Reserved] │
│ 事件批次哈希 | 公开时间戳 | 供应商认证凭证      │
│ 不可转让交易凭证 | 数据产品护照 NFT | 展示索引  │
└──────────────────────────────────────────────┘
                    │
┌──────────────────────────────────────────────┐
│       外部服务 / 预言机 / 生态接口 [V1 Active + V2 Reserved] │
│ [V1 Active] KYC/KYB | CA/电子签章 | 支付/发票 | 评级 | 时间戳 │
│ [V2 Reserved] TEE 远程证明 | 安全检测      │
│ [V1 Active] KMS/HSM | SIEM/SOC | 消息桥/同步服务           │
└──────────────────────────────────────────────┘
```

---

## 9. 用户角色分类与权限模型

## 9.1 主体身份分类（一级分类）

### 1）自然人
分为：
- 个人买方
- 个人开发者 / 算法提交者
- 个人数据贡献者（仅受托/合作模式）
- 研究人员 / 独立顾问

**原则：**
- V1 不支持普通自然人自由上架数据副本。
- 自然人如参与供给，优先通过“受托方 / 数据合作社 / 授权委托”的方式接入。
- 自然人买方仅可购买低风险或聚合结果类产品。

### 2）企业
分为：
- 数据供方
- 数据买方
- 双边参与方
- 数据加工服务方
- 模型/算法服务方
- 节点参与方
- 生态合作伙伴

### 3）政府/公共机构
分为：
- 公共数据提供方
- 授权运营方
- 监管方
- 联合治理方

### 4）事业单位 / 医疗机构 / 教育机构 / 科研机构 / 行业协会
分为：
- 数据供方
- 联合研究参与方
- 受控计算参与方
- 标准/认证机构

### 5）境外主体
分为：
- 境外企业
- 境外研究机构
- 境外服务商

**原则：**
- 默认不开放高敏/重要数据自由访问。
- 必须触发跨境合规审查与特殊审批流。
- 优先支持“境内受控计算 + 结果出境”的模式。

## 9.2 企业级主体与账户对象模型（新增，研发必须据此建模）

平台至少要区分以下对象，不能再只用单一 `Party` 承担全部职责：

1. **Tenant / Organization（租户/组织工作空间）**：平台中的业务边界单位，承载部门、用户、应用、连接器、订单和账单。
2. **Party（法律主体）**：合同和责任归属单位，负责签约、付款、审计与追责。一个 Tenant 必须绑定且仅绑定一个主 Party，但一个 Party 可拥有多个 Tenant（如多品牌、多地区分公司）。
3. **Department（部门）**：组织内部的权限与成本中心，如采购部、法务部、数据治理部。
4. **User（用户）**：平台具体操作者，隶属于某 Tenant，可兼任多个角色。
5. **Application（应用）**：由 Tenant 创建的系统身份，用于 API 调用、批处理任务或第三方系统接入。
6. **Connector（连接器）**：连接链下数据源、沙箱或外部执行环境的受控接入点。
7. **Execution Environment（执行环境）**：文件下载环境、API 服务环境、沙箱、V2 计算环境等。

## 9.3 平台内部角色

- 平台超级管理员
- 运营管理员
- 主体审核员
- 产品审核员
- 合规审核员
- 风控专员
- 法务专员
- 争议处理专员
- 财务结算专员
- 审计管理员
- 监管观察员
- 节点运维管理员
- 连接器/执行环境运维方

## 9.4 角色模型设计原则

平台权限必须采用 **RBAC + ABAC + PBAC + Scope** 混合模型：
- **RBAC**：按角色授予基础能力。
- **ABAC**：按主体类型、行业、地区、信用、KYC 等动态判定。
- **PBAC**：按合约与策略做最终放行或阻断。
- **Scope**：权限必须绑定到 Tenant / Department / Product SKU / Application / Connector / Environment 具体范围。

## 9.5 权限判断链路

任一访问请求都必须按如下顺序判断：
1. 租户是否有效、是否被冻结。
2. 法律主体是否通过认证，是否在黑白灰名单中。
3. 用户/应用是否属于该租户与部门范围。
4. 角色是否允许该类动作。
5. 产品 SKU 是否允许该类权利出售。
6. 合同是否已生效，订单是否已支付/锁定。
7. 所属连接器/执行环境是否满足策略要求。
8. 当前请求是否命中频控、额度、时间窗、地域限制。
9. 是否命中风控和合规阻断规则。

## 9.6 角色权限示例矩阵（摘要）

| 对象 | 典型角色 | 可执行动作 | 默认禁止 |
|---|---|---|---|
| Tenant 管理员 | 企业管理员 | 用户/部门/应用管理、账单查看、审批发起 | 越过法务/合规角色直接放行高风险交易 |
| Department 审批人 | 部门负责人 | 部门级订单审批、预算确认 | 修改合同模板 |
| User-采购角色 | 采购经理 | RFQ、下单、验收、续费 | 创建供方产品 |
| User-法务角色 | 法务 | 合同审阅、条款审批 | 修改技术策略 |
| User-数据治理角色 | 数据管理员 | 产品上架、元数据维护、样例上传 | 直接确认付款 |
| Application | 系统账号 | API 调用、批量查询 | 登录控制台、转授权 |
| Connector 运维 | 连接器管理员 | 健康检查、策略下发 | 查看订单财务信息 |
| 监管观察员 | 监管节点/监管账号 | 查看监管视图、导出证据包 | 介入业务配置 |

---

## 10. 平台核心实体设计（需求基线版）

本节明确研发需落地的对象模型。后续数据库、权限引擎、流程引擎、SDK、审计模型都应以此为基线。

## 10.1 租户实体（Tenant / Organization）

### 属性
- tenant_id
- tenant_name
- bind_party_id
- tenant_status（active / suspended / closed）
- industry_primary
- region
- billing_plan
- default_contract_template_set
- default_approval_flow_set
- default_public_chain_flag
- created_at / updated_at

## 10.2 主体实体（Party）

### 基本属性
- party_id
- party_type（自然人/企业/政府/事业单位/科研/境外主体）
- legal_name
- unified_code / id_no / org_code
- registration_region
- industry_classification
- beneficial_owner_info（企业）
- kyc_kyb_status
- risk_level
- credit_score
- certification_level
- blacklist_status
- role_set

### 扩展属性
- data_processing_capability_level
- security_capability_level
- allowed_data_level
- cross_border_flag
- node_member_flag
- controlled_env_capability_flag
- connector_capability_flag

## 10.3 部门实体（Department）
- department_id
- tenant_id
- department_name
- parent_department_id
- cost_center_code
- approver_user_ids
- allowed_budget_limit
- data_scope_tags
- department_status

## 10.4 用户实体（User）
- user_id
- tenant_id
- department_id
- bind_party_id
- name
- email / phone
- role_set
- identity_level
- mfa_status
- employment_status
- last_login_at
- user_status

## 10.5 应用实体（Application）
- app_id
- tenant_id
- app_name
- app_type（internal system / external partner / automation）
- owner_user_id
- app_secret_status
- allowed_product_scope
- allowed_connector_scope
- allowed_environment_scope
- rate_limit_profile
- app_status

## 10.6 连接器实体（Connector）
- connector_id
- tenant_id
- connector_type（DB / object storage / API proxy / sandbox bridge / V2 compute connector）
- owner_party_id
- bind_data_resource_ids
- network_zone
- auth_mode
- health_status
- last_attested_at
- supported_actions
- connector_status

## 10.7 执行环境实体（Execution Environment）
- env_id
- env_type（download / api / sandbox / V2 compute）
- owner_party_id
- isolation_level
- export_policy
- audit_policy
- trusted_attestation_flag
- supported_product_types
- current_capacity
- environment_status

## 10.8 数据资源实体（Data Resource）
- resource_id
- resource_name
- owner_party_id
- source_type
- source_basis
- data_domain
- time_range / geo_range
- update_frequency
- storage_location
- data_scale
- format_type
- contains_pi_flag
- contains_spi_flag
- trade_secret_flag
- important_data_flag
- security_level
- rights_status
- transformation_capability
- non_trade_reason

## 10.9 数据产品实体（Data Product）
- product_id
- product_name
- product_type
- owner_tenant_id
- owner_party_id
- bind_resource_ids
- delivery_mode
- usage_mode
- sku_template_id
- quality_score
- compliance_risk_level
- preview_uri
- update_commitment
- service_sla_level
- current_version
- product_status

字段归属规则：

- `product_type` 只表达目录与展示分类。
- `delivery_mode` 表达该产品默认展示或优先推荐的交付路径，可作为 SKU 初始化默认值或目录聚合展示值，但不替代 SKU 级事实源。
- `usage_mode` 表达推荐使用形态或默认使用姿态，用于详情展示、审核提示和模板选型，不等同于正式权利集合。
- `sku_template_id` 表达该产品默认绑定的 SKU 模板族或模板初始化策略，用于批量生成 SKU 或为新 SKU 提供默认模板，不等同于单个 SKU 的最终模板快照。
- `Data Product` 是产品级目录事实源；进入交易、授权、验收和结算时，必须以下游 `Product SKU` 快照为准。

## 10.10 产品 SKU 实体（Product SKU）

用于把“卖什么权利”落为可下单对象。一个数据产品可对应多个 SKU。

- sku_id
- product_id
- sku_type（`FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD`）
- sku_name
- rights_profile_code（由正式 `rights_type` 组合生成，仅用于模板、展示与检索，不构成独立权利枚举）
- allowed_rights（仅允许填写正式 `rights_type` 枚举）
- forbidden_rights
- delivery_mode
- pricing_mode
- contract_template_id
- default_usage_policy_id
- acceptance_template_id
- refund_template_id
- settlement_rule_id
- public_chain_display_flag
- sku_status

建模要求：

- `sku_type` 是标准 SKU 主轴。
- `allowed_rights / forbidden_rights` 是权利主轴，只能引用正式 `rights_type` 枚举。
- `delivery_mode` 表达交付路径，`pricing_mode` 表达计费路径，两者都不得替代权利枚举。
- `rights_profile_code` 仅作为派生组合标签使用，不得再以 `rights_bundle_type` 一类混合字段充当权利真值源。
- `Product SKU` 是下单、合同、授权、验收、账单和结算的直接事实源。
- 若 `Product` 与 `SKU` 在 `delivery_mode`、模板选择或使用姿态上存在差异，均以 `SKU` 快照为准；`Product` 仅保留目录默认值与聚合展示值。
- `usage_mode` 与 `allowed_rights` 不得混用：前者表达推荐使用姿态，后者表达正式权利边界。
- `contract_template_id / default_usage_policy_id / acceptance_template_id / refund_template_id` 为 SKU 级最终模板绑定；不得再回写覆盖产品级默认模板族。

## 10.11 数字合约实体（Digital Contract）
- contract_id
- contract_template_id
- buyer_party_id / seller_party_id
- bind_sku_id
- authority_model
- proof_commit_policy
- proof_commit_state
- external_fact_status
- reconcile_status
- idempotency_key
- request_id / trace_id
- status_version
- use_purpose
- use_scope
- time_limit
- region_limit
- environment_requirement
- export_limit
- sublicense_flag
- model_training_flag
- commercial_use_flag
- derivative_flag
- approval_flow_id
- sign_status
- hash_digest
- effective_status

## 10.12 授权策略实体（Usage Policy）
- policy_id
- bind_sku_id / bind_contract_id
- allow_subject_scope
- allow_action_set
- allow_time_window
- allow_quota
- allow_field_scope
- allow_record_scope
- algorithm_whitelist
- output_masking_rule
- watermark_rule
- breach_trigger_rule
- auto_revoke_rule

## 10.12A 授权实例实体（Authorization）

- authorization_id
- bind_contract_id / bind_sku_id / bind_policy_id
- subject_type / subject_id
- authority_model
- proof_commit_policy
- proof_commit_state
- external_fact_status
- reconcile_status
- idempotency_key
- request_id / trace_id
- status_version
- grant_status
- effective_from / effective_to
- revoke_reason_code
- last_reconciled_at

## 10.12B 交付实体（Delivery）

- delivery_id
- order_id / sku_id / contract_id
- delivery_mode
- authority_model
- proof_commit_policy
- proof_commit_state
- external_fact_status
- reconcile_status
- idempotency_key
- request_id / trace_id
- status_version
- delivery_status
- delivery_receipt_ref
- delivered_at
- last_reconciled_at

## 10.12C 结算实体（Settlement）

- settlement_id
- order_id / bill_event_id
- settlement_cycle
- authority_model
- proof_commit_policy
- proof_commit_state
- external_fact_status
- reconcile_status
- idempotency_key
- request_id / trace_id
- status_version
- settlement_status
- payable_amount / receivable_amount
- settled_at
- last_reconciled_at

## 10.12D 争议实体（Dispute）

- dispute_id
- order_id / contract_id / delivery_id / settlement_id
- dispute_type
- authority_model
- proof_commit_policy
- proof_commit_state
- external_fact_status
- reconcile_status
- idempotency_key
- request_id / trace_id
- status_version
- dispute_status
- dispute_reason_code
- resolution_type
- resolved_at

## 10.13 交易订单实体（Order）
- order_id
- buyer_tenant_id / seller_tenant_id
- buyer_party_id / seller_party_id
- bind_product_id / bind_sku_id
- quote_mode
- idempotency_key
- request_id / trace_id
- order_amount
- deposit_amount
- contract_id
- authority_model
- proof_commit_policy
- proof_commit_state
- external_fact_status
- reconcile_status
- payment_status
- delivery_status
- acceptance_status
- settlement_status
- dispute_status
- invoice_status
- current_state
- status_version
- last_reconciled_at
- last_external_fact_at

### 10.13.1 Order 状态字段工程分层

- `current_state`：订单流程推进主状态，也是默认展示主状态。
- `status_version`：订单状态写入版本号，只用于并发控制、重放保护和顺序校验，不表达业务阶段。
- `payment_status / delivery_status / acceptance_status / settlement_status / dispute_status / invoice_status`：业务域子状态，由各自领域服务维护。
- `proof_commit_state / external_fact_status / reconcile_status`：技术与审计镜像状态，用于一致性、外部事实和存证链路校验，不单独替代业务主状态。

### 10.13.2 Order 单一状态源与优先级规则

1. `current_state` 是订单流程推进唯一主状态；任何服务不得仅凭子状态直接视订单“已完成”。
2. 订单页面默认主徽标只展示 `current_state`；其他状态以副徽标、阻断提示或一致性提示呈现。
3. `payment_status / delivery_status / acceptance_status / settlement_status` 的更新只能由对应领域服务写入；若触发流程推进，必须由订单编排服务在同一事务或同一 outbox 流中同步推进 `current_state`。
4. `dispute_status` 可阻断交付、结算、开票和断权，但不直接覆盖 `current_state`；展示层可额外显示“争议中”高优先级横幅。
5. `external_fact_status / reconcile_status` 只影响自动结算、自动放行、自动锚定等自动动作，不回写成新的业务阶段。
6. `proof_commit_state` 只表示链上或证据提交进度，不得替代支付成功、交付成功、验收成功等业务事实。
7. 状态冲突时按以下顺序处理：
   - 先校验 `status_version` 是否允许本次写入；
   - 再校验 `current_state` 与各业务域子状态是否满足状态映射；
   - 最后校验 `external_fact_status / reconcile_status / proof_commit_state` 是否允许自动动作继续执行。
8. 任何回退都必须走显式补偿或人工覆写流程，并生成审计事件，不允许静默把 `current_state` 倒退到更早阶段。

## 10.14 账单事件实体（Billing Event）

用于把结算从“原则”落到“可计算事件源”。

- bill_event_id
- order_id
- sku_id
- authority_model
- proof_commit_policy
- proof_commit_state
- external_fact_status
- reconcile_status
- idempotency_key
- request_id / trace_id
- status_version
- event_type（一次性交付 / 月度订阅 / API 调用 / 沙箱席位 / 逾期赔付 / 退款 / 分账）
- event_time
- metered_quantity
- billing_unit
- unit_price
- amount
- settlement_cycle
- source_system
- evidence_ref
- bill_status

## 10.15 审计事件实体（Audit Event）
- event_id
- event_type
- operator_type（user / app / connector / system）
- operator_id
- related_object_type / related_object_id
- timestamp
- source_ip / device_id / connector_id
- precondition_snapshot
- execution_result
- risk_tag
- onchain_hash
- raw_log_ref
- evidence_level

## 10.16 计算任务实体（Compute Task）

适用于 V2 的 C2D / MPC / FL / TEE / ZKP 场景。

- task_id
- task_type
- initiator_party_id
- provider_party_ids
- algorithm_id
- execution_env_id
- input_rule
- output_rule
- resource_consumption
- approval_status
- execution_status
- result_summary
- proof_material_ref
- charging_rule

## 10.17 公链事件锚定批次实体（Public Anchor Batch）
- batch_id
- source_chain
- target_chain
- event_scope
- merkle_root / hash_root
- event_count
- anchor_tx_hash
- anchor_status
- anchor_time
- public_disclosure_level
- verification_url
- retry_count
- operator_id
- exception_reason

## 10.18 凭证类 Token / NFT 实体（Credential Token）
- credential_id
- credential_type（交易凭证 / 供应商认证 / 数据产品护照）
- token_standard
- mint_address / token_id
- owner_subject_id
- bind_subject_type
- bind_product_id
- bind_order_id
- metadata_hash
- display_metadata_uri
- revocation_status
- valid_from / valid_to
- issuer_org_id
- issue_reason
- public_display_flag

## 10.19 数据产品护照实体（Data Product Passport）
- passport_id
- product_id
- current_version
- catalog_hash
- schema_hash
- quality_report_hash
- compliance_report_hash
- authorization_template_hash
- latest_audit_batch_id
- service_sla_level
- public_summary_uri
- update_cycle
- last_anchor_time

## 10.20 对象模型工程冻结规则

### 10.20.1 命名规则

- 对象名统一使用英文单数聚合名；中文只作为章节标题和解释文本。
- 字段名统一使用英文 `snake_case`，不得在同一对象字段清单中继续混用中文业务名。
- 页面展示名、合同描述名、审计展示名都必须映射到同一工程字段名，不允许各系统自创近义字段。

### 10.20.2 字段分层规则

- 业务字段：表达交易、授权、计费、交付、验收、结算事实，例如 `order_amount`、`pricing_mode`、`delivery_mode`。
- 控制字段：表达流程控制和放行条件，例如 `status`、`current_state`、`approval_status`、`status_version`。
- 审计字段：表达留痕、回放、对账、外部事实和证据引用，例如 `request_id`、`trace_id`、`evidence_ref`、`proof_commit_state`。
- 集成字段：表达外部系统映射、连接器、网关、链和支付回执，例如 `connector_id`、`anchor_tx_hash`、`external_fact_status`。

### 10.20.3 关联、基数与唯一约束规则

- `Tenant 1:N Department / User / Application / Connector`
- `Party 1:N DataResource / DataProduct`
- `DataProduct 1:N ProductSKU`
- `Order 1:N BillingEvent / AuditEvent`
- `UsagePolicy` 与 `DigitalContract` 必须绑定到明确的 `SKU` 或合同快照，不允许悬空存在
- 任何 `bind_*_ids / allowed_* / supported_*` 在 PRD 中只表示聚合视图；工程落库时应优先拆成关系表或快照表，数组字段只能作为缓存或投影，不得作为唯一事实源
- 所有主键统一全局唯一；自然键必须明确作用域，例如“商品内唯一”“租户内唯一”“主体内唯一”

### 10.20.4 版本与快照规则

- `current_version` 只表示当前指针，不表示可变版本正文；正式版本内容必须落入不可变版本表或快照表
- 合同、策略、模板、商品元数据、质量报告、授权规则只要进入交易或审核，就必须固化快照
- `status_version` 只服务状态并发控制，不得与商品版本、合同版本、策略版本混用


## 11. 交易对象与数据产品分类设计

## 11.1 交易对象不应只定义为“数据集”

平台应以五类基础对象 family 组织交易标的，并在其下扩展高级能力：

### 1）文件 / 对象包产品
- 形式：CSV/Parquet/图片/音视频/压缩包/结构化快照/合成数据包/特征文件包
- 适用：静态快照、离线导出、标签包、衍生结果包
- 要求：版本号、哈希、水印、导出控制、用途限制

### 2）表 / 库 / 视图 / 数据共享产品
- 形式：只读表、只读视图、secure view、linked dataset、delta share、只读 schema
- 适用：持续可查、零拷贝、可撤权的数据共享
- 要求：recipient 绑定、只读共享、到期撤权、共享协议快照

### 3）API / 服务产品
- 形式：REST / GraphQL / MQ / WebSocket / 评分服务 / 检索服务
- 适用：实时数据、持续更新数据、标准化查询与评分能力
- 优势：更容易计费、断权和用途控制

### 4）V1 查询类对象与 V2 受控执行扩展
- `V1` 形式：查询沙箱、SQL 模板、TVF 模板、clean room lite
- `V2` 扩展：受控任务、C2D 执行、其他受控执行能力
- 适用：买方不拿原始明细，只执行被批准的查询或任务
- `V1` 要求：模板白名单、输出边界、导出限制、审计留痕
- `V2` 要求：任务审批、执行环境约束、算法白名单、执行证明

### 5）结果 / 合成 / 特征 / 模型衍生产品
- 形式：固定报告、聚合结果、合成数据、特征结果、模型 API、模型工件
- 适用：买方只要结果不要原始明文，或购买模型能力而非原始数据
- 要求：明确与原始数据关系、可逆性风险、导出与商用边界

在这五类基础 family 之上，平台继续支持高级扩展：

- 计算权产品（Compute-to-Data）
- 联邦学习产品
- MPC 结果产品
- 模型 / 算法服务产品
- ZKP 证明产品

## 11.2 不同数据类型的推荐交易方式

| 数据类型 | 推荐交易方式 | 不推荐方式 |
|---|---|---|
| 公开数据/开放数据 | 下载/API | 无 |
| 低敏企业经营数据 | 下载/API/订阅 | 无控制买断 |
| 高频实时 IoT 数据 | 流式订阅/API | 文件快照为主 |
| 敏感个人信息相关数据 | 查询/计算权/联邦/MPC | 原始副本交付 |
| 医疗/金融高敏数据 | C2D/FL/MPC/沙箱 | 副本下载 |
| 政务授权运营数据 | API/沙箱/受控结果 | 无限复制分发 |
| 重要数据相关产品 | 受控协作/结果输出 | 下载型交易 |
| 模型训练能力 | 联邦/计算权/受控训练 | 无约束下载训练集 |
| 资格证明类数据 | ZKP / 证明服务 | 明文全量披露 |

### 11.2.1 平台标准交易方式清单（增强）

平台统一支持以下交易方式：

1. 一次性文件快照出售
2. 版本订阅 / 周期更新
3. 零拷贝只读共享
4. API / 服务按调用或按周期计费
5. 模板查询 / clean room lite / 受限结果交付
6. 联邦学习 / MPC / 受控协作任务
7. 结果产品 / 合成数据 / 特征结果 / 模型结果交付

其中：

- `V1` 正式支持：1 / 2 / 3 / 4 / 5（lite）/ 7（结果类）
- `V2` 正式支持：完整 5 / 6 / 模型服务
- `V1` 只要求最小可交付路径，不要求一次支持全部共享协议、完整 clean room 或复杂 revision 策略
- `V2/V3` 必须把 `V1` 的对象骨架补齐为完整协作能力，而不是长期停留在试点实现

---



## 11.3 权利模型与产品类型矩阵

### 11.3.1 权利集合定义
- **使用权**：在约定业务场景中内部使用数据或结果。
- **访问权**：通过 API、共享接口或受控服务访问数据能力。
- **查询权**：在受控环境内执行限定查询。
- **计算权**：提交算法或任务到数据侧执行。
- **结果获取权**：获得符合输出规则的结果集、报告或聚合指标。
- **有限内部共享权**：允许在买方同一 Tenant 内的指定部门、用户或应用范围内共享。
- **受控模型训练权**：允许在受控条件下将产品用于模型训练或微调。
- **结果级商业输出权**：允许将结果用于约定范围内的对外商业服务。
- **转授权权**：允许再授权给第三方。

### 11.3.2 V1 权利边界
- V1 仅正式支持：**使用权、访问权、查询权、结果获取权、有限内部共享权**。
- V1 默认禁止：**所有权转让、无限期模型训练权、转授权权、开放再分发权**。
- V2 再引入：**计算权、受控模型训练权、结果级商业输出权**。

### 11.3.2A 建模主轴与派生轴

- `rights_type` 是权利体系唯一枚举源。
- `sku_type` 是标准可售对象主轴，用于定义 8 个标准 SKU。
- `delivery_mode` 只表达交付路径，不表达权利边界。
- `pricing_mode` 只表达计费路径，不表达权利边界。
- `allowed_rights / forbidden_rights` 是基于 `rights_type` 的正式权利集合。
- `rights_profile_code` 仅用于模板映射、展示和检索，不得单独扩展出新的权利枚举。
- `product_type / 商品族` 用于目录与产品分类，不得替代 `rights_type` 或 `sku_type`。

### 11.3.3 产品类型 × 可售权利 × 默认限制矩阵

| 产品类型 | V1/V2 | 可售权利 | 默认禁止 | 默认合同模板 | 默认计费 | 默认交付 |
|---|---|---|---|---|---|---|
| 文件快照 / 版本订阅 | V1 | 使用权、有限内部共享权 | 转售、转授权、默认训练、对外公开分发 | T-FILE-STD | 一次性/分批/周期 | 文件令牌下载 / 版本推送 |
| 只读共享产品 | V1 | 访问权、查询权、结果获取权 | 写入共享源、转授权、subscriber 越权扩散 | T-SHARE-RO | 周期费/订阅费 | 只读 share grant / linked dataset / datashare |
| API / 服务产品 | V1 | 访问权、结果获取权 | 凭证共享、绕过配额、批量导出明细、再分发 | T-API-SUB | 月订阅 + 调用量 | API Key / OAuth |
| 模板查询 lite | V1 | 查询权、结果获取权 | 自由 SQL、越权参数、原始明细导出、结果逆向识别 | T-QUERY-LITE | 按次/套餐/周期 | template grant / 白名单模板执行 |
| 查询沙箱 | V1 | 查询权、结果获取权、有限内部共享权（可选） | 原始明细导出、脚本逃逸、任意容器执行、结果逆向识别 | T-SBX-STD | 席位/月费/按次 | 沙箱账号 / 项目空间 / 受限导出 |
| 固定报告 / 结果产品 | V1 | 结果获取权、使用权、有限内部共享权（可选） | 反向恢复原始明细、默认再分发、默认训练 | T-RESULT-STD | 一次性/按批次 | 结果包下载 / 报告交付 |
| C2D 计算权产品 | V2 | 计算权、结果获取权 | 原始数据访问、未审算法执行 | T-C2D-CTRL | 任务费/算力费 | 任务编排 |
| 联邦学习产品 | V2 | 计算权、结果获取权、受控模型训练权 | 拉取他方样本、导出中间参数（默认） | T-FL-JOINT | 项目费/阶段费 | 联合训练任务 |
| MPC 结果产品 | V2 | 结果获取权 | 查看他方输入、下载中间明细 | T-MPC-RES | 按任务/按结果 | 多方计算 |
| 模型/算法产品 | V2 | 访问权、结果获取权、结果级商业输出权 | 反编译、权重复制（默认） | T-MODEL-SVC | 调用量/服务期 | 模型 API |
| ZKP 证明产品 | V2 | 访问权、结果获取权 | 获取底层证明材料明文 | T-ZKP-PROOF | 按次 | 证明回执 |

### 11.3.4 数据对象交易能力当前完成度与分期收口

当前已设计完成并冻结的内容：

- 五类基础对象 family 与七类标准交易方式
- `V1` 正式商品化范围：文件快照/版本订阅、只读共享、API/服务、模板查询 lite、查询沙箱、固定报告/结果产品
- 权利边界、默认禁止动作、标准 SKU、标准状态机与基础交付链路
- `asset_object_binding`、`data_share_grant`、`revision_subscription`、`template_query_grant` 等数据库骨架

当前尚未彻底完成、但已明确必须在后续阶段补齐的内容：

- `V1` 只读共享仅承诺最小可交付实现，不承诺一次支持 `Snowflake / BigQuery / Delta Sharing` 等完整异构协议矩阵
- `V1` 模板查询仅承诺白名单模板与受限结果交付，不承诺自由 SQL 或完整 clean room
- `V1` 版本订阅仅承诺固定周期和标准 revision 发布，不承诺复杂历史授权、动态窗口和跨平台游标协同
- `V1` 文件快照、共享、API、模板查询、结果产品虽已区分对象类型，但完整的差异化授权/验收/退款/赔付规则体系仍需在 `V2/V3` 持续补齐

阶段收口要求：

- `V2`：完成共享连接器框架、完整 clean room、完整 revision 治理、按交付模式分流的规则体系
- `V3`：完成跨平台共享、跨平台订阅、跨域受控执行、跨平台证据与责任链贯通

### 11.3.5 数据商品元信息与数据契约最小要求

`V1` 起，每个可售数据商品都必须同时具备：

- 元信息档案：覆盖业务描述、结构描述、质量描述、安全与合规描述、交付描述、版本描述、验收描述、授权描述、责任描述。
- 字段结构说明：至少说明字段名、字段类型、主键/时间字段、编码规则和结构摘要。
- 质量报告摘要：至少说明覆盖范围、刷新频率、采样方式、缺失率、异常率和质量评分。
- 加工责任链摘要：至少说明原始提供者、加工者、平台、第三方评估方及其责任边界。
- 数据契约：至少固化交付义务、验收标准、授权边界、责任边界和争议口径。

`V2/V3` 可以继续扩展到模型、计算和跨平台对象，但不允许回退到“只有标题和价格”的商品定义方式。

### 11.3.6 数据原样处理与产品化加工最小要求

平台必须采用“原始格式保留 + 类内标准化 + 统一治理层”的处理模型，不允许把所有原始数据粗暴转换成单一格式后再交易。

`V1` 起必须明确六区模型：

- 原始接入区
- 分类识别区
- 类内标准化区
- 加工处理区
- 产品包装区
- 交付运营区

`V1` 起每个可售数据商品至少应完成：

- 原始接入登记
- 格式识别与对象分类
- 基础类内标准化
- 加工责任链记录
- 样例或预览生成
- 数据契约与质量报告绑定

`V2/V3` 继续补齐模型制品、复杂 extraction、跨平台加工责任链与外部质量证明互认。

### 11.3.7 数据商品存储与分层存储最小要求

平台必须采用四层分工模型：

- 控制面：PostgreSQL
- 数据面：MinIO / S3 兼容对象存储
- 查询/执行面：受控查询与执行环境
- 信任面：Fabric 联盟链

`V1` 起必须明确最小分层存储区：

- `raw`
- `curated`
- `product`
- `preview`
- `delivery`
- `archive`
- `evidence`

并满足以下要求：

- PostgreSQL 保存目录、状态、契约、对象绑定、哈希和生命周期信息，不保存大规模原始数据本体。
- 对象存储保存原始文件、加工后产品包、预览文件、临时交付对象、证据原文和模型制品。
- 查询/执行面只读取 `curated / product` 等受控区域，不绕过治理直接暴露 `raw` 区。
- Fabric 只锚定对象摘要、版本摘要、订单摘要、验收摘要和证据批次根。
- `raw` 区对象不得直接对外售卖，商品目录只能引用正式 `product` 区对象或受控执行入口。

### 11.3.8 数据商品查询与执行面最小要求

平台必须把“查询/执行面”定义为正式交付能力，而不是数据库账号、临时脚本或人工运维动作。

`V1` 起至少正式支持三类查询面：

- `template_query_lite`
- `sandbox_query`
- `report_result`

`V1` 起必须满足以下要求：

- 查询面只允许读取经过治理的 `curated / product / result` 区，不得把 `raw` 区作为默认对外查询面。
- 查询权必须作为独立权利表达，与下载权、API 访问权、结果获取权分开建模。
- 模板查询必须通过白名单模板、参数 schema 校验、输出边界校验和导出限制校验后才能执行。
- 沙箱查询必须绑定正式执行环境、会话时效、导出策略和审计留痕，不得直接下发自由数据库账号。
- 每次查询执行都必须形成正式运行记录，记录请求参数摘要、命中模板、输出摘要、计费单位和审计引用。
- `V1` 不承诺自由 SQL、完整 clean room、任意自定义分析规则；这些能力在 `V2/V3` 继续完整补齐。

### 11.3.9 敏感数据处理与受控交付最小要求

平台必须把敏感数据交易设计为“受控使用权交易”，而不是默认的原始副本交易。

`V1` 起必须满足以下要求：

- 含敏感个人信息、高敏行业明细、重要数据相关商品默认不得走原始副本下载交付。
- 敏感商品必须根据分类分级自动收口到 `report_result / template_query_lite / sandbox_query / seller_hosted_api / 受控执行入口` 等正式路径。
- 每个敏感商品都必须同时具备：处理依据摘要、结构摘要、质量摘要、安全预览、输出边界、验收口径和撤权口径。
- 卖方不信任平台或平台不持有明文时，仍必须形成版本承诺、数据契约、可交付性证明和试算/预览证据。
- 敏感执行动作必须强化审计，记录 `actor / step_up / query_run or task / result_object / masked_level / export_scope / approval_ticket`。
- 链上只锚定敏感交易摘要，不得锚定原始敏感正文、实名身份、合同正文和敏感日志。
- `V1` 正式承诺高敏场景的最小闭环为：受限预览、模板查询 lite、受控沙箱、结果产品与卖方自持 API；完整 clean room / TEE / FL / MPC 在 `V2/V3` 正式补齐。

## 11.4 V1 标准 SKU 与模板编码

本节只保留正式版标准 SKU 真值，不再保留旧的 `SKU-A / SKU-B / SKU-C / SKU-D` 过渡口径。

### 11.4.1 V1 八个标准 SKU

| 标准 SKU | 商品族 | 默认权利 | 默认交付 | 默认模板族 |
|---|---|---|---|---|
| `FILE_STD` | 文件快照 | 使用权 + 有限内部共享权（可选） | 文件令牌下载 / 限次下载 | `CONTRACT_FILE_V1 / LICENSE_FILE_USE_V1 / DELIVERY_FILE_V1 / ACCEPT_FILE_V1 / REFUND_FILE_V1 / BILL_FILE_ONCE_V1` |
| `FILE_SUB` | 版本订阅 | 使用权 + 有限内部共享权（可选） | 周期版本推送 / 周期交付 | `CONTRACT_FILE_SUB_V1 / LICENSE_FILE_USE_V1 / DELIVERY_FILE_SUB_V1 / ACCEPT_FILE_SUB_V1 / REFUND_FILE_SUB_V1 / BILL_FILE_SUB_V1` |
| `SHARE_RO` | 只读共享 | 访问权 + 查询权 + 结果获取权 | `share grant / linked dataset / datashare` | `CONTRACT_SHARE_RO_V1 / LICENSE_SHARE_RO_V1 / DELIVERY_SHARE_RO_V1 / ACCEPT_SHARE_RO_V1 / REFUND_SHARE_RO_V1 / BILL_SHARE_RO_V1` |
| `API_SUB` | API / 服务 | 访问权 + 结果获取权 | `Application + Key / OAuth` | `CONTRACT_API_SUB_V1 / LICENSE_API_SUB_V1 / DELIVERY_API_SUB_V1 / ACCEPT_API_SUB_V1 / REFUND_API_SUB_V1 / BILL_API_SUB_V1` |
| `API_PPU` | API / 服务 | 访问权 + 结果获取权 | `Application + Key / OAuth` | `CONTRACT_API_PPU_V1 / LICENSE_API_PPU_V1 / DELIVERY_API_PPU_V1 / ACCEPT_API_PPU_V1 / REFUND_API_PPU_V1 / BILL_API_PPU_V1` |
| `QRY_LITE` | 模板查询 lite | 查询权 + 结果获取权 | `template grant / 白名单模板执行` | `CONTRACT_QUERY_LITE_V1 / LICENSE_QUERY_LITE_V1 / DELIVERY_QUERY_LITE_V1 / ACCEPT_QUERY_LITE_V1 / REFUND_QUERY_LITE_V1 / BILL_QUERY_LITE_V1` |
| `SBX_STD` | 查询沙箱 | 查询权 + 结果获取权（必要时可附有限内部共享权） | `沙箱账号 / 项目空间 / 受限导出` | `CONTRACT_SANDBOX_V1 / LICENSE_SANDBOX_USE_V1 / DELIVERY_SANDBOX_V1 / ACCEPT_SANDBOX_V1 / REFUND_SANDBOX_V1 / BILL_SANDBOX_V1` |
| `RPT_STD` | 固定报告 / 结果产品 | 结果获取权 + 使用权 + 有限内部共享权（可选） | `报告交付 / 结果包下载` | `CONTRACT_REPORT_V1 / LICENSE_RESULT_USE_V1 / DELIVERY_REPORT_V1 / ACCEPT_REPORT_V1 / REFUND_REPORT_V1 / BILL_REPORT_V1` |

### 11.4.2 设计约束

- `V1` 的标准 SKU 真值源只有以上 8 个编码
- 预置商业套餐、报价包、促销包、席位包、项目包都只能作为标准 SKU 下的实现型套餐，不得反向替代标准 SKU 编码
- `QRY_LITE` 与 `SBX_STD` 必须分开建模、分开授权、分开验收
- `SHARE_RO` 必须拥有独立模板族，不得并入文件包或 API 类模板
- `API_SUB` 与 `API_PPU` 的默认权利统一为“访问权 + 结果获取权”，默认模板命名统一按 `SUB / PPU` 细分，不得再回退为未区分计费模式的通用 `API` 模板名

### 11.4.3 V2 预留标准 SKU

以下编码只允许作为 `V2/V3` 预留，不进入 `V1` 正式销售和验收范围：

- `C2D_CTRL`
- `FL_JOINT`
- `MPC_RES`
- `MODEL_SVC`
- `ZKP_PROOF`

## 12. 数据分类分级与主体适配规则

## 12.1 数据分类维度

### 按来源分
- 个人数据
- 企业数据
- 公共数据
- 工业设备数据
- 互联网行为数据
- 科研实验数据
- 模型生成数据
- 第三方加工衍生数据

### 按敏感程度分
- 公开
- 内部
- 受限
- 敏感
- 高敏/监管重点
- 禁止市场化流通

### 按法律属性分
- 不含个人信息
- 含个人信息
- 含敏感个人信息
- 涉商业秘密
- 涉重要数据
- 涉国家核心数据相关范围（如适用场景命中）

### 按使用形态分
- 可下载
- 可 API 调用
- 可查询
- 可计算
- 可联邦协作
- 仅可证明
- 不可交易

## 12.2 主体-数据适配原则

### 自然人主体
- 可作为买方购买低风险产品
- 可作为开发者提交算法
- 不可自由售卖敏感个人信息
- 高风险场景必须走受托模式

### 企业主体
- 是平台主要交易主体
- 可根据资质购买不同级别产品
- 能否下载/导出取决于行业、用途、合约、受控环境

### 政府/公共机构
- 更适合作为公共数据提供方、监管方或可信节点
- 对公共数据需执行授权运营、场景审批和结果治理

### 科研机构
- 更适合获取脱敏结果、受控计算权、联合建模权
- 伦理和研究目的声明必须进入合约与审查链路

### 境外主体
- 原则上优先支持“境内计算、受控输出”
- 涉出境必须单独审查

---

## 13. 注册前审计与准入要求

## 13.1 为什么注册前就要审计

这个平台不是普通 SaaS 注册页。
对于数据供方、敏感数据买方、算法服务方、隐私计算服务方，**注册前审计**是平台信任基础的一部分。

## 13.2 主体准入层级

### L0：游客
- 仅浏览公开介绍和开放目录
- 不可查看详细样例
- 不可发起交易

### L1：基础实名用户
- 可查看一般产品详情
- 可收藏、询价、申请试用
- 不可访问受限产品

### L2：认证企业/机构
- 完成 KYB / 资质验证
- 可购买标准产品
- 可参与标准交易

### L3：增强认证主体
- 完成法务、风控、安全能力评估
- 可参与受限产品交易
- 可发起受控计算任务

### L4：高风险/重点行业主体
- 增强认证 + 专项审查 + 审批白名单
- 方可访问高敏或特殊行业能力

## 13.3 供方注册前审查项

- 主体真实性
- 法定代表/授权代表真实性
- 数据来源合法性声明
- 数据采集/处理依据
- 数据处理规则与隐私政策
- 是否涉及第三方授权
- 是否含个人信息/敏感个人信息
- 是否涉及商业秘密、重要数据
- 数据质量与更新能力证明
- 安全能力与事故记录
- 历史违规记录
- 所属行业限制

## 13.4 买方注册前审查项

- 主体真实性
- 使用场景说明
- 行业背景
- 是否存在高风险用途
- 数据处理能力
- 安全保障能力
- 是否具备受控环境
- 历史违规/侵权/失信记录
- 是否涉及跨境使用

## 13.5 算法/隐私计算服务方审查项

- 算法来源
- 开发者身份
- 算法安全扫描结果
- 是否存在数据外传风险
- 容器镜像签名
- 模型/算法许可证
- 是否符合可执行白名单

---

## 14. 核心功能架构草图（需求阶段）

## 14.1 门户与交易前模块
- [V1 Active] 首页与场景导航
- [V1 Active] 数据目录中心
- [V1 Active] 行业专题页
- [V1 Active] 搜索与推荐
- [V1 Active] 上架中心
- [V1 Active] 产品编辑器
- [V1 Active] 样例/沙箱预览
- [V1 Active] 需求大厅（求购单/招标单）
- [V1 Active] 询价/报价/竞价
- [V1 Active] 咨询与 POC 申请

## 14.2 审查与风控模块
- [V1 Active] 主体审核台
- [V1 Active] 产品审核台
- [V1 Active] 合规审查台
- [V1 Active] 风险评分引擎
- [V1 Active] 黑灰名单
- [V1 Active] 人工复核工作台
- [V2 Reserved] 跨境流转审查工作台
- [V1 Active] 敏感字段识别
- [V1 Active] 权利证明校验

## 14.3 合同与授权模块
- [V1 Active] 标准合同模板库
- [V1 Active] 场景化数字合约模板
- [V1 Active] 条款选择器
- [V1 Active] 权限策略引擎
- [V1 Active] 电子签章
- [V1 Active] 合同版本管理
- [V1 Active] 授权生效/变更/撤销
- [V1 Active] 自动断权规则

## 14.4 交付与执行模块
- [V1 Active] 文件交付中心
- [V1 Active] API 网关
- [V1 Active] Token / Key 管理
- [V1 Active] 数据清洗/脱敏流水线
- [V1 Active] 查询沙箱
- [V2 Reserved] 安全空间
- [V2 Reserved] C2D 任务中心
- [V2 Reserved] MPC 任务调度
- [V2 Reserved] FL 任务中心
- [V2 Reserved] TEE 证明校验
- [V2 Reserved] ZKP 校验服务

## 14.5 结算与财务模块
- [V1 Active] 订单结算
- [V1 Active] 账单中心
- [V1 Active] 调用量计费
- [V2 Reserved] 分润规则引擎
- [V1 Active] 第三方支付接口
- [V1 Active] 发票管理
- [V1 Active] 退款/赔付处理

## 14.6 审计与监管模块
- [V1 Active] 存证上链
- [V1 Active] 审计日志中心
- [V1 Active] 操作追踪
- [V1 Active] 证据包导出
- [V1 Active] 监管视图
- [V1 Active] 重点交易监控
- [V1 Active] 风险事件上报
- [V1 Active] 合规台账

## 14.7 运营与生态模块
- [V1 Active] 用户与角色管理
- [V1 Active] 供需运营
- [V1 Active] 评价与信用体系
- [V3 Reserved] 数商与第三方服务市场
- [V2 Reserved] 培训认证
- [V1 Active] 案例中心
- [V2 Reserved] 服务商接入
- [V1 Active] 平台公告与规则中心

---

## 14.8 公链展示与公开验证模块
- [V2 Reserved] Solana 展示事件上链服务
- [V2 Reserved] 联盟链事件批量哈希锚定
- [V2 Reserved] 公链披露策略引擎
- [V2 Reserved] 公链回执查询
- [V2 Reserved] 公开验证页 / 浏览器跳转
- [V2 Reserved] 供应商认证凭证签发
- [V2 Reserved] 不可转让交易凭证签发
- [V2 Reserved] 数据产品护照 NFT 生成与更新
- [V2 Reserved] 凭证撤销 / 到期 / 冻结同步
- [V2 Reserved] 对外品牌展示与生态演示页

## 14.9 开发者平台与集成模块
- [V1 Active] API 门户与开发者中心
- [V1 Active] SDK / OpenAPI / 示例代码
- [V1 Active] 测试沙箱 / Mock 数据
- [V1 Active] 应用凭证与密钥管理
- [V1 Active] 调用分析与配额管理
- [V1 Active] Webhook / 事件订阅
- [V1 Active] 数据连接器接入规范
- [V2 Reserved] 第三方应用审批与上架
- [V2 Reserved] 合作伙伴集成控制台
- [V1 Active] 外部 SIEM / BI / ERP / CRM 对接接口

## 14.10 V1 开发清单（从总图中剥离）

以下模块属于 `V1` 实际开发、联调、验收清单；未进入本清单的模块即使出现在总图中，也不构成 `V1` 开发承诺：

- 门户与交易前：首页与场景导航、数据目录中心、行业专题页、搜索与推荐、上架中心、产品编辑器、样例/沙箱预览、需求大厅、询价/报价/竞价、咨询与 POC 申请
- 审查与风控：主体审核台、产品审核台、合规审查台、风险评分引擎、黑灰名单、人工复核工作台、敏感字段识别、权利证明校验
- 合同与授权：标准合同模板库、数字合约模板、条款选择器、权限策略引擎、电子签章、合同版本管理、授权生效/变更/撤销、自动断权规则
- 交付与执行：文件交付中心、API 网关、Token / Key 管理、数据清洗/脱敏流水线、查询沙箱
- 结算与财务：订单结算、账单中心、调用量计费、第三方支付接口、发票管理、退款/赔付处理
- 审计与监管：存证上链、审计日志中心、操作追踪、证据包导出、监管视图、重点交易监控、风险事件上报、合规台账
- 运营与生态：用户与角色管理、供需运营、评价与信用体系、案例中心、平台公告与规则中心
- 开发者平台与集成：API 门户与开发者中心、SDK / OpenAPI / 示例代码、测试沙箱 / Mock 数据、应用凭证与密钥管理、调用分析与配额管理、Webhook / 事件订阅、数据连接器接入规范、外部 SIEM / BI / ERP / CRM 对接接口

### 14.10.1 V1-Core（首笔标准交易闭环必需）

- 主体与账户：租户、企业主体、部门、用户、应用、连接器、执行环境、KYC/KYB、黑白灰名单、RBAC、基础 OIDC、MFA、会话治理
- 商品与交易：目录中心、上架中心、八个标准 SKU、询报价、合同、授权、订单、支付锁定、交付、验收、结算、争议
- 交付与控制：文件交付、API 网关、模板查询 lite、查询沙箱、只读共享开通、基础断权
- 审计与一致性：联盟链存证、证据包导出、审计日志、双层权威一致性、基础对账、回放
- 支付与财务：支付意图、基础退款、人工打款、账单中心、发票申请、Mock PaymentProvider

### 14.10.2 V1-Extended（首批客户需要，可在首个商用客户前补齐）

- 搜索与推荐增强
- 企业 OIDC SSO 深化、设备信任治理
- 开发者平台、SDK、Webhook、外部 SIEM / BI / ERP / CRM 对接
- 监管视图、重点交易监控、风险事件上报
- 半自动分账、周期账单增强、发票流转增强

### 14.10.3 V1-Reserved（仍属 V1 架构范围，但默认后置实施）

- Solana 演示锚定、公开验证页、凭证展示页
- 合作伙伴控制台、第三方应用审批与上架
- 复杂共享连接器增强、卖方自持共享增强
- 二维码授权、SCIM、SAML 2.0 深度联邦

## 15. 核心交易链路设计（完整闭环）

## 15.1 统一主链路（适用于全部产品）

### 步骤 1：主体注册与审核
- 创建 Tenant、绑定 Party、建立 Department / User / Application
- 完成 KYC/KYB、资质校验、黑白灰名单校验
- 根据行业和能力确定可交易产品范围

### 步骤 2：数据资源治理与产品化
- 录入 Data Resource
- 形成 Data Product 与 SKU
- 绑定合同模板、计费模板、验收模板、结算模板
- 产出样例、预览、质量报告、合规标签

### 步骤 3：产品审核与挂牌
- 系统自动规则初审
- 高风险项人工复核
- 审核通过后上架目录并生成产品版本

### 步骤 4：发现与撮合
- 买方搜索、筛选、发起询价或直接下单
- 平台支持标准报价、阶梯价格、订阅价格
- 对大额/高风险订单进入审批流

### 步骤 5：合同与授权
- 选择 SKU -> 自动匹配合同模板
- 双方协商条款（如有）
- 电子签章
- 生成 Usage Policy

### 步骤 6：付款与锁定
- 预付款/全款/保证金锁定
- 订单状态从 `contract_effective` 进入 `payment_locked`
- 锁定后才允许交付令牌、API Key、沙箱账号发放

### 步骤 7：交付或执行
- 文件型：签发一次性或限次下载令牌、写入水印
- API 型：签发应用级访问凭证、额度配置、回源策略
- 沙箱型：开通用户/部门席位、限制模板与导出规则
- V2 计算型：创建 Compute Task、绑定执行环境

### 步骤 8：验收
- 按产品类型执行自动验收或人工验收
- 不同产品类型有独立拒收条件和补救路径

### 步骤 9：结算与账单归档
- 根据 Billing Event 汇总账单
- 释放托管资金
- V1 采用人工结算或半自动分账
- 自动分润能力作为 V2 扩展
- 发票申请、账单归档、审计留痕

### 步骤 10：持续使用监管与售后
- 监控配额、到期、异常访问
- 续费、扩容、重新授权
- 争议受理、证据导出、责任认定

## 15.2 V1 首批标准交易链路（可直接进入拆解）

### 链路 A：文件副本一次性交易
挂牌 -> 审核 -> 下单 -> 合同签署 -> 全款锁定 -> 下载令牌签发 -> 买方下载并校验 -> 自动/人工验收 -> 结算 -> 审计归档

### 链路 B：API 订阅交易
挂牌 -> 审核 -> 下单 -> 合同签署 -> 首期费用锁定 -> 应用绑定 -> API 凭证签发 -> 健康检查 -> 首次成功调用 -> 周期账单 -> 续费/停用

### 链路 C：沙箱查询交易
挂牌 -> 审核 -> 下单 -> 合同签署 -> 支付锁定 -> 沙箱席位开通 -> 模板查询 -> 结果下载（受限） -> 验收 -> 账单结算

### 链路 D：定制报告服务
需求提交 -> 人工报价 -> 合同 -> 付款锁定 -> 供方生成报告 -> 平台交付 -> 买方验收 -> 结算

### 链路 E：查询+订阅混合包
签约 -> API Key + 沙箱席位同时开通 -> 账单按“基础订阅 + 超额用量”计算 -> 到期统一断权

## 15.3 订单-验收-结算状态机（按产品类型拆分）

### 15.3.1 通用订单状态
`draft -> quoted -> approval_pending -> contract_pending -> contract_effective -> payment_locked -> delivery_in_progress -> acceptance_pending -> accepted / rejected / disputed -> settled -> closed`

### 15.3.2 文件副本产品状态机（V1）
- **交付完成条件**：下载令牌签发成功，文件包生成完成，可校验哈希。
- **自动验收条件**：买方成功下载 + 平台校验文件哈希一致；或交付后 3 个工作日未提出异议。
- **拒收条件**：文件损坏、字段与样例/合同不一致、缺失约定批次、加密包无法解密。
- **失败重试**：允许补发下载令牌 2 次；超过 2 次需人工介入。
- **退款/赔付触发点**：无法交付、重复交付损坏、字段缺失严重。
- **账单事件源**：文件包数量、版本号、下载成功回执。

### 15.3.3 API / 订阅产品状态机（V1）
- **交付完成条件**：应用绑定成功、API Key 生效、健康检查通过。
- **自动验收条件**：首次成功调用返回 2xx/业务成功；或开通后 5 个工作日无异议。
- **拒收条件**：接口不可用、字段契约不符、鉴权失败、SLA 未达标。
- **失败重试**：凭证重签发、限流配置重下发、路由切换。
- **退款/赔付触发点**：上线失败、连续 SLA 违约、账单误计费。
- **账单事件源**：订阅周期、调用量、超额调用、峰值并发。

### 15.3.4 只读共享状态机（SHARE_RO）
- **交付完成条件**：共享对象开通、读权限可验证、样例查询返回结果。
- **自动验收条件**：买方完成首个只读查询；或 5 个工作日未异议。
- **拒收条件**：共享未开通、对象不可见、授权范围与合同不符、样例查询失败。
- **失败重试**：重下发共享授权、重建共享对象、修复只读策略。
- **退款/赔付触发点**：共享长期不可用、授权范围错误、SLA 连续违约。
- **账单事件源**：共享席位周期、共享对象数量、共享可用天数。

### 15.3.5 模板查询 lite 状态机（QRY_LITE）
- **交付完成条件**：模板授权开通、白名单模板可执行、结果边界校验通过。
- **自动验收条件**：买方首次执行模板成功；或 5 个工作日未异议。
- **拒收条件**：模板未开通、模板执行失败、输出边界与合同不符、结果异常。
- **失败重试**：补开模板授权、修复模板参数、重放模板执行。
- **退款/赔付触发点**：模板长期不可用、模板授权错误、结果持续不符合约定。
- **账单事件源**：模板执行次数、结果集导出次数、超额执行次数。

### 15.3.6 查询沙箱状态机（SBX_STD）
- **交付完成条件**：账号/席位开通、项目空间创建、样例查询通过。
- **自动验收条件**：首次登录且样例查询成功；或 5 个工作日未异议。
- **拒收条件**：账号未开通、席位不可用、导出规则与合同不符、沙箱环境异常。
- **失败重试**：补开席位、重置权限、修复环境、替换项目空间。
- **退款/赔付触发点**：席位无法使用、环境长期不可用、导出能力错误配置。
- **账单事件源**：席位天数、席位数、项目空间天数、导出次数。

### 15.3.7 固定报告 / 结果产品状态机（RPT_STD）
- **交付完成条件**：报告或结果包生成完成、交付回执生成、下载或签收路径可验证。
- **自动验收条件**：买方下载或签收结果包；或 5 个工作日未异议。
- **拒收条件**：报告未交付、结果结构与合同不符、关键指标缺失、交付件损坏。
- **失败重试**：重生成报告、补发结果包、重签交付回执。
- **退款/赔付触发点**：未按期交付、内容明显不符、交付件反复损坏。
- **账单事件源**：报告份数、结果包数量、加急交付次数。

### 15.3.8 C2D / 受控计算产品状态机（V2 预留）
- **交付完成条件**：任务创建、算法审核通过、任务在指定环境完成。
- **自动验收条件**：结果包生成并满足输出模板；证明材料齐全。
- **拒收条件**：任务失败、输出越界、算法被拒、证明缺失。
- **失败重试**：重新调度、替换算力、二次审核。
- **退款/赔付触发点**：平台原因导致任务反复失败、结果不可用。
- **账单事件源**：任务数、CPU/GPU 时长、结果等级、人工审核次数。

## 15.4 账单事件源与结算口径

### 15.4.1 统一账单事件源
所有结算必须由 `Billing Event` 驱动，禁止直接按订单状态拍脑袋结算。

### 15.4.2 V1 支持的计费口径
- 文件产品：文件包数、版本数、分批交付次数
- 只读共享：共享周期、共享对象数量、共享可用天数
- API 产品：订阅周期、调用次数、超额调用量、峰值并发档位
- 模板查询 lite：模板执行次数、结果导出次数、超额执行量
- 查询沙箱：席位数、席位天数、项目空间天数、导出次数
- 固定报告/结果产品：报告份数、结果包数量、加急交付次数

### 15.4.3 分账顺序
买方付款 -> 平台托管 -> 验收成功或自动验收 -> 平台服务费扣除 -> 供方分账 -> 税务/发票状态更新 -> 账单归档

### 15.4.4 退款与赔付顺序
争议成立 -> 冻结待结算金额 -> 按责任比例退款/赔付 -> 更新信用记录 -> 输出审计包

### 15.4.5 V1 结算实现边界
- V1 必须支持账单事件归集、账单生成、应结金额计算、人工结算指令、结算结果留痕。
- V1 可支持半自动分账，即系统计算分配结果、人工确认执行。
- V1 不要求上线全自动分润引擎。
- 自动分润、复杂多方收益拆分、贡献度驱动分润统一放入 V2。
---

## 16. 业务规则（收敛版）

## 16.1 上架规则
1. 未完成 Tenant + Party 绑定和 KYC/KYB，不得上架。
2. 每个上架产品必须绑定至少一个 SKU。
3. 每个 SKU 必须绑定合同模板、验收模板、退款模板、计费模板。
4. V1 只允许上架 `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD` 八个标准 SKU 对应的商品。
5. 重大变更（字段、交付方式、权利、价格模型）必须重新审核并生成新版本。

## 16.2 权利与 SKU 规则
1. 平台销售对象是 `SKU + 权利边界`，不是抽象“数据”。
2. 默认不销售所有权、不销售无限期转授权权。
3. 受控模型训练权在 V1 默认禁止；如确有业务需求，需走 V2 受控计算路径。
4. 有限内部共享权必须限定到 Tenant 内指定 Department / User / Application。
5. 每个 SKU 必须标明：允许动作、禁止动作、交付方式、验收条件、账单口径、赔付方式。

## 16.3 审批与阻断规则
1. 高风险交易必须进入人工审批，不得系统自动放行。
2. 以下任一命中直接阻断：
   - 权利证明缺失
   - 原始敏感个人信息副本交付
   - 境外主体申请原始副本下载
   - 未声明用途却申请模型训练
   - 合同模板与 SKU 类型不匹配
3. 以下命中进入增强审批：
   - 政府/公共数据授权运营
   - 金额超过阈值
   - 含个人信息但已有合法处理基础
   - 对外商业输出权申请

## 16.4 交付规则
1. 文件产品交付单位是“文件包/版本包”，不是原始库表全量访问。
2. API 产品交付单位是“应用级访问权”，不是“用户共享口令”。
3. 沙箱产品交付单位是“席位 + 模板 + 结果导出限制”。
4. V2 计算产品交付单位是“任务执行权 + 结果获取权”，不是数据副本。
5. 到期自动断权必须覆盖用户、应用、连接器和环境四层。

## 16.5 验收规则
1. 不同产品类型必须有不同验收模板，禁止统一一个“已收到”即验收。
2. 自动验收计时器必须在“交付完成”后启动，而不是支付后启动。
3. 验收失败后仅允许进入：补交付、部分退款、全额退款、争议处理四种路径。

## 16.6 结算规则
1. 结算必须由 Billing Event 汇总触发。
2. 没有账单事件，不得结算。
3. 账单争议期间，待争议部分金额冻结，其他无争议部分允许先行结算。
4. 分账规则必须在合同生效前锁定，避免事后改分账。

## 16.7 争议规则
争议至少覆盖：
- 数据与描述不符
- 文件损坏/字段缺失
- API 不可用/SLA 不达标
- 沙箱模板不可用
- 越权使用
- 误计费/重复计费
- 恶意退款
- 非法训练/违规导出

平台必须具备：争议受理、证据调取、日志核验、责任判定、赔付执行、信用惩戒。

## 16.8 公链披露与展示规则
1. V1 生产环境默认关闭 Solana 公链展示。
2. V1 如因 KPI 需要演示，只允许同步 **联盟链事件批次哈希** 到测试网或演示环境。
3. V2 才允许在生产环境开放对外公开验证页和 NFT/凭证签发。
4. 公链只展示摘要、批次、公开元数据，不展示原始数据、实名身份、合同正文、敏感日志。
5. 公链展示功能必须可按 Tenant / Product / Environment 级别关闭。

## 16.9 应急冻结、下架与撤销规则
1. 主体违规、产品违规、合规结论变更时，平台可冻结 SKU、Order、Application、Connector。
2. 冻结顺序：先停新购 -> 再停续费 -> 再停交付 -> 最后停存量访问（按合同与监管要求执行）。
3. 撤销或下架不影响审计日志、账单和证据链保留义务。


## 17. 权限逻辑设计

## 17.1 授权最小单元

平台权限不能只做到“产品级”，必须支持至少以下粒度：

- 产品级
- 接口级
- 字段级
- 记录级
- 时间窗口级
- 调用频率级
- 任务级
- 算法级
- 输出结果级
- 用户组/部门级

## 17.2 权限动作集合

- 浏览元数据
- 预览样本
- 询价
- 下单
- 下载
- API 调用
- 沙箱查询
- 提交算法
- 发起联邦任务
- 发起 MPC 任务
- 获取结果
- 导出结果
- 续费续权
- 转授予内部子账号
- 申请扩权

## 17.3 扩权流程

买方如需扩大权限，必须发起扩权申请，平台重新审查以下内容：

- 新用途
- 新部门
- 新地域
- 新算法
- 新导出需求
- 新训练需求
- 新共享对象

经审批通过后，生成新版本授权策略与合约补充条款。

## 17.4 违规触发自动处置

- 连续异常调用 -> 自动限流
- 批量导出尝试 -> 自动冻结导出
- 可疑跨账号共享 -> 自动锁定凭证
- 异常区域访问 -> 二次认证/阻断
- 算法输出越界 -> 自动中止任务
- 到期未续费 -> 自动断权

---

## 18. 数据规则

## 18.1 链上数据规则

链上只允许存以下信息：
- 主体 DID / 凭证摘要
- 产品登记摘要
- 合同哈希
- 授权状态
- 订单状态
- 支付与分账摘要
- 交付回执摘要
- 审计摘要
- 争议证据索引
- 时间戳与签名

**严禁**将原始明细数据、敏感字段、模型权重全文、密钥材料直接上链。

## 18.2 链下数据规则

链下存储：
- 原始数据
- 样本明细
- 模型文件
- 算法容器
- 全量日志
- 敏感配置
- 密钥管理材料
- 证据原件

要求：
- 加密存储
- 分级保护
- 密钥隔离
- 水印/指纹
- 版本留存
- 可删除/可冻结/可归档

## 18.3 元数据规则

每个可交易产品必须有标准元数据：
- 名称
- 描述
- 来源
- 范围
- 时间范围
- 更新频率
- 字段说明
- 样例
- 适用场景
- 禁用场景
- 风险等级
- 交付方式
- 权利类型
- 价格模型
- SLA

## 18.4 质量规则

质量评估至少覆盖：
- 完整性
- 准确性
- 一致性
- 时效性
- 覆盖度
- 连续性
- 稳定性
- 可解释性

平台必须支持：
- 自动质量检测
- 手工质量说明
- 第三方质量报告
- 历史质量趋势
- 质量争议反馈

## 18.5 数据清洗与脱敏规则

平台需提供：
- 标准化清洗
- 缺失值处理
- 去重
- 字段映射
- 脱敏模板
- 匿名化/去标识化
- 假名化
- 规则审计
- 再识别风险评估

## 18.6 输出结果规则

在受控计算场景下，结果输出必须可配置：
- 只返回统计汇总
- 只返回模型指标
- 返回脱敏样本
- 返回受限报告
- 返回证明而非原值
- 结果经过最小化审查
- 结果留痕并可复核

---

## 18.7 数据血缘、版本与撤回规则
- 平台必须记录数据产品从来源、加工、脱敏、打包、上架到交付的完整血缘链
- 数据集、API、模型、标签包、报告都必须支持版本号与版本差异说明
- 当上游原始数据发生失效、误采集、违规采集、质量缺陷时，应支持下游影响分析
- 数据产品护照应展示最近一次版本、质量评估、合规状态、适用范围和 SLA 摘要
- 平台应支持“版本冻结”“强制下架”“向买方推送撤回通知”三类动作

## 18.8 开放接口与集成规则
- 所有可交易 API 应具备统一鉴权、统一审计、统一计费、统一限流能力
- 平台应提供 OpenAPI/SDK/回调事件规范，降低第三方接入成本
- 对接外部数据连接器、数据空间连接器时，必须校验连接器身份、版本和策略兼容性
- 外部应用接入需经过应用审查、最小权限授权和周期性复核
- 平台应支持导出标准化账单、审计摘要和履约状态给 ERP/财务/监管系统

## 19. 合规与安全要求

## 19.1 国内基础合规框架

平台设计必须适配以下规则方向：
- 数据安全
- 个人信息保护
- 数据跨境流动
- 数据交易合规评估
- 公共数据授权运营
- 可信数据空间
- 电子签名/电子合同
- 网络与系统安全

## 19.2 合规审查输入字段（新增，系统必须结构化采集）

每个主体、产品、订单在进入审核时，至少要采集以下字段：
- 主体类型 / 地区 / 行业 / 认证等级
- 数据分类分级 / 是否含 PI / SPI / 重要数据
- 数据来源证明、权利证明、处理依据
- 产品类型 / SKU 类型 / 交付方式
- 使用目的 / 是否训练模型 / 是否商业输出 / 是否再分发
- 是否涉及境外主体 / 是否存在出境路径
- 执行环境类型 / 是否具备受控环境
- 合同模板编号 / 审批流编号
- 风险标签 / 金额等级 / 历史违规记录

## 19.3 合规审查决策表（新增，研发按规则引擎实现）

| 规则编号 | 输入条件 | 系统动作 | 审批级别 | 阻断点 | 证据要求 | 时效要求 |
|---|---|---|---|---|---|---|
| C-001 | 缺少权利证明或来源证明 | 自动驳回 | 无 | 上架前阻断 | 证明材料上传 | 实时 |
| C-002 | 原始敏感个人信息 + 文件副本交付 | 自动阻断 | 无 | 下单前阻断 | 分类分级结果 | 实时 |
| C-003 | 含个人信息但未填写处理依据 | 自动驳回 | 无 | 上架前阻断 | 同意/授权/法定义务证明 | 实时 |
| C-004 | 境外主体申请下载副本 | 自动阻断 | 无 | 交易协商前阻断 | 主体信息、交付方式 | 实时 |
| C-005 | 模型训练用途 + V1 文件/API/沙箱产品 | 自动阻断 | 无 | 合同生效前 | 训练用途说明、环境说明 | 实时 |
| C-006 | 公共数据授权运营产品 | 转专项审批 | L3 + L4 | 上架前 | 授权文件、运营范围 | 3 个工作日 |
| C-007 | 重要数据 / 高风险行业数据 | 转人工增强审批 | L4 | 合同生效前 | 分类证明、安全评估 | 3 个工作日 |
| C-008 | 申请商业输出权 | 转人工审批 | L3 | 合同生效前 | 业务场景说明、输出边界 | 2 个工作日 |
| C-009 | 自然人作为供方申请上架副本 | 自动阻断 | 无 | 注册前/上架前 | 主体类型判断 | 实时 |
| C-010 | 合同模板与 SKU 类型不匹配 | 自动驳回 | 无 | 合同签署前 | 模板与 SKU 校验记录 | 实时 |

## 19.4 风控决策表（新增，交易中/交易后）

| 风控编号 | 命中条件 | 自动动作 | 人工复核角色 | 处置时限 |
|---|---|---|---|---|
| R-001 | 短时高频下载/超限调用 | 限流 + 告警 | 风控专员 | 30 分钟 |
| R-002 | 共享 API 凭证/IP 异常漂移 | 临时冻结应用 | 安全运营 | 30 分钟 |
| R-003 | 沙箱导出异常增加 | 暂停导出能力 | 风控 + 合规 | 2 小时 |
| R-004 | 账单异常飙升 | 进入人工复核，不自动扣费 | 财务结算 | 1 个工作日 |
| R-005 | 连续 SLA 违约 | 自动生成赔付工单 | 运营 + 财务 | 1 个工作日 |
| R-006 | 已签约产品被判定违规 | 冻结新购/续费，评估存量访问 | 合规 + 法务 | 4 小时 |

## 19.5 平台必须内建的合规能力
- 自动规则初审
- 人工复核工作台
- 证据材料上传与留存
- 审批流配置
- 高风险清单管理
- 删除/撤回/限制处理工单
- 跨境用途审查
- 自动化决策风险提示
- 审查日志归档

## 19.6 高风险清单机制
平台必须维护“禁止 / 限制 / 需审批”三类高风险清单，并支持版本化生效。至少包括：
- 原始敏感个人信息副本交易
- 医疗/金融高敏明细直接下载
- 重要数据无审批交易
- 未说明用途的模型训练采购
- 跨境自由导出
- 可能导致歧视性自动化决策的数据用途
- 画像营销且处理依据不足的交易

## 19.7 系统安全要求（可验收版）
- 全链路 TLS 加密，敏感字段静态加密
- 高风险操作二次认证，关键审批双人复核
- KMS/HSM 管理核心密钥，密钥轮换周期可配置
- 用户、应用、连接器均纳入零信任访问控制
- 审计日志与业务日志分级存储，证据日志防篡改
- 容器/连接器/应用支持镜像或包级安全扫描
- 安全事件进入 SOC/SIEM，形成工单闭环
- 灾备、备份、恢复演练纳入季度计划


## 19.8 威胁模型与安全控制基线

注意：当前阶段允许以可审计的软件密钥管理替代，需要在实现阶段添加TODO注释后期完成完整实现，如：企业 IAM / OIDC / OAuth2.1、KMS/HSM、双人审批等。

平台在需求阶段就必须显式定义威胁模型，不能只写“注意安全”。至少要覆盖以下七类威胁：
- 恶意卖方：数据描述失真、版本不一致、拖延交付、伪造样本
- 恶意买方：下载后拒付、恶意拒收、虚假投诉、刷差评
- 平台内部滥用：越权查看、擅改状态、泄露密钥、绕过审批
- 链下文件泄露：下载链接转发、对象存储暴露、密文离线破解尝试
- 模型/算法投毒：联邦任务上传恶意更新、受控计算任务植入外传逻辑
- 跨链与预言机攻击：重放、伪造回执、中间人、桥接节点串谋
- 风控绕过：串谋评分、洗订单、异常下载、凭证共享、违规转售

对应的系统级最低控制要求如下：
- 高风险操作必须启用多因素认证、二次确认或双人审批
- 密钥、令牌、证书、下载授权必须全程留痕并支持吊销
- 审计日志与业务日志必须隔离存储，关键证据日志应具备防篡改保全能力
- 对象下载、API 调用、沙箱访问、任务执行必须形成可回溯主体与设备证据
- 风控命中后必须支持冻结主体、冻结 SKU、冻结任务、冻结令牌四类动作
- 安全控制不能仅覆盖交易期，还必须覆盖续费、扩权、撤权、导出、失效与吊销阶段

## 19.9 身份、密钥与证书治理基线

注意：当前阶段允许以可审计的软件密钥管理替代，需要在实现阶段添加TODO注释后期完
成完整实现，如：企业 IAM / OIDC / OAuth2.1、KMS/HSM、双人审批等。

平台身份体系必须至少区分以下四层，不得用单一账号模型混用：
- 组织身份：用于合同、账单、责任归属与审计主体
- 用户身份：用于登录、审批、人工操作与责任追踪
- 链上身份：用于链上签名、成员准入、节点治理与凭证签发
- 服务身份：用于微服务、网关、连接器、预言机之间的机器鉴权

最低治理要求：
- 登录层优先采用企业 IAM / OIDC / OAuth2.1 等统一身份体系，链上身份不直接替代最终用户登录
- 高风险审批、冻结、解冻、吊销、签发等动作必须绑定 MFA 与强审计
- 浏览器门户与开放 API 的会话体系必须分层设计：门户优先服务端会话，开放 API 使用短期令牌与可轮换刷新令牌
- 组织成员邀请、企业 SSO 首次建档、设备信任、会话撤销与 step-up 认证必须纳入正式对象模型
- 对象或文件加密必须支持 DEK/KEK 分层治理，DEK 不得长期复用
- 证书、密钥、下载令牌、服务签名密钥必须支持轮换、吊销、过期与异常停用
- 连接器、执行环境、跨链网关、预言机都必须具备独立服务身份，不得复用平台管理员口令
- 密钥托管、封装与调用必须进入 KMS/HSM 或等价可审计治理能力

## 20. 隐私保护计算与先进能力要求

## 20.1 多方安全计算（MPC）
适用：
- 联合风控
- 联合营销
- 风险评分
- 交集求解
- 不共享原始数据的联合统计

需求：
- 任务编排
- 各方授权
- 输入输出约束
- 结果审查
- 任务审计
- 成本计费

## 20.2 联邦学习（FL）
适用：
- 多机构联合建模
- 模型训练协作
- 医疗/金融/工业协作

需求：
- 参与方管理
- 模型版本管理
- 聚合策略管理
- 训练日志
- 贡献度评估
- 训练权授权

## 20.3 零知识证明（ZKP）
适用：
- 只证明资格、属性或事实
- 不披露底层原始数据

需求：
- 证明模板
- 证明验证接口
- 证明有效期
- 证明存证
- 证明撤销机制

## 20.4 可信执行环境（TEE）
适用：
- 高敏数据受控分析
- 算法运行可信证明

需求：
- 远程证明接入
- 环境白名单
- 运行镜像签名
- 执行回执
- 结果摘要上链

## 20.5 Compute-to-Data
适用：
- 数据不出域
- 买方只拿结果或模型效果

需求：
- 算法上传审核
- 运行环境审批
- 输出白名单
- 算法网络访问限制
- 任务级计费
- 完整执行日志与证明

---

## 21. 智能合约、预言机、联盟链与公链接入需求

## 21.1 联盟链要求

优先使用**许可型联盟链**，原因：
- 可控身份
- 可治理
- 可监管接入
- 更适合隐私与审计
- 性能更适合企业级业务

### 联盟链节点建议角色
- 平台主运营节点
- 监管观察节点
- 核心机构节点
- 公证/存证节点
- 结算/清分协同节点（如需要）

## 21.2 智能合约职责

智能合约应负责：
- 主体登记状态同步
- 产品登记摘要
- 合同状态机
- 授权生效/变更/撤销
- 订单状态流转
- 保证金与结算触发
- 分润规则执行
- 违约触发规则
- 审计摘要写入
- 争议冻结标记

**不建议**智能合约承担复杂全文检索、原始数据计算、大文件存储。

## 21.3 预言机职责

需要预言机接入的外部信息包括：
- KYC / KYB 校验结果
- CA/电子签章结果
- 支付成功结果
- 发票系统回执
- TEE 远程证明结果
- 第三方审计结论
- 合规审批结果
- 外部时间戳/可信时间源
- 信用评级或黑名单结果（如适用）

---

## 21.4 Solana 公链接入策略（建议定位为展示层 / 增信层）

### 定位原则
- 联盟链承载核心交易状态。
- Solana 仅承载公开验证事件锚点、轻量展示凭证和公开索引。
- 公链信息仅为摘要、索引、可公开证明，不反向替代联盟链主状态。

### 版本边界
- **V1 生产环境默认关闭**，仅允许测试网/演示环境做批次哈希锚定。
- **V2 才进入生产增强包**：开放公开验证页、凭证/NFT 签发、可配置的产品护照展示。

### 启用条件（防止变成纯展示工程）
同时满足以下至少 2 项，方可在生产开启：
1. 有明确外部合作方或客户要求“公开可验证回执”。
2. 有品牌、招商、合作生态 KPI 需要对外展示可信交易能力。
3. 有监管/审计场景需要公开时间戳或不可否认证明补充。
4. 已完成成本评估，公链费用预算可控。

### Solana 侧建议承载对象
- 联盟链重要事件批次哈希
- 公开可验证的时间戳与回执
- 不可转让交易凭证
- 供应商认证凭证
- 数据产品护照 NFT
- 对外展示索引页所需的轻量元数据

### 不建议放到 Solana 的内容
- 原始数据
- 敏感合同正文
- 实名身份明文
- 详细调用日志
- 核心结算和争议仲裁主状态

## 21.5 凭证 Token / NFT 需求
### 不可转让交易凭证
- 用于展示“某交易已经在平台内按规则完成”
- 默认不可转让，不作为金融资产流通
- 仅包含订单摘要、产品摘要、时间、状态、签发机构摘要
- 生产签发仅在 V2 开启，V1 可在演示环境模拟签发
- 应支持失效、吊销、隐藏展示状态

### 供应商认证凭证
- 用于展示供应商已完成平台认证、合规审查或等级评定
- 应区分基础认证、增强认证、行业认证、专项认证
- 认证到期后应自动失效或更新
- 默认不作为任何交易放行的唯一依据，仍以平台内认证状态为准

### 数据产品护照 NFT
- 用于展示某数据产品的公开摘要、版本、质量、合规、更新节奏和最近审计状态
- 不代表数据所有权，不可替代正式交易授权
- 应支持元数据更新、版本切换和下架状态
- 默认只展示可公开字段，不映射受限 SKU 细节

## 21.6 公链同步、桥接与运维要求
- 公链同步默认采用单向批量锚定，不要求 V1 双向跨链业务闭环
- 如需跨链消息协议，应仅用于事件摘要同步，不直接搬运核心敏感业务数据
- 应建立专门的公链同步服务，负责批次打包、签名、广播、重试、失败告警
- 公链 Gas / 费用预算、节流策略、批量策略应纳入运维与财务模型
- 公链合约升级、权限管理、签发密钥管理应独立审批，不与联盟链治理混用
- 应提供“关闭公链展示而不影响核心业务”的降级能力
- 公链同步失败不得阻塞联盟链订单、结算、审计主流程


## 21.7 跨链请求与幂等治理要求

注意：V1/V2 仅保留对象模型与幂等字段，不实现完整跨链请求编排

当平台进入 V3 或需要与外部联盟链/可信数据空间互联时，必须把跨链请求视为受治理业务对象，而不是简单消息转发。

最小跨链请求对象至少包含：
- ccr_id / request_id
- source_chain_id / target_chain_id
- request_type
- payload_hash
- nonce
- timestamp
- gateway_signature
- witness_signature_set（如采用预言机/见证人）
- ack_hash / ack_status
- final_status

最小控制要求：
- 每个跨链请求必须具备唯一 nonce、时间戳与来源链标识，防止重放
- 目标侧必须按 request_id 幂等处理，不得因重复回执导致重复授权、重复结算或重复放行
- 跨链成功不能只以“消息已发出”为准，必须有目标侧 ack 或等价确认
- 必须支持超时、重试、人工介入、终止四类状态
- 跨链失败不得污染联盟链主状态，不得阻塞本地核心交易链路
- 涉及授权同步、结算确认、冻结同步的跨链请求必须纳入审计视图与风险工单

## 22. 交易前、交易中、交易后的服务闭环

## 22.1 交易前服务

- 主体认证与入驻辅导
- 数据治理咨询
- 数据清洗与脱敏服务
- 数据产品包装服务
- 合规评估服务
- 质量评估服务
- 定价辅导
- 样本沙箱与 POC
- 需求梳理与撮合顾问

## 22.2 交易中服务

- 询价/竞价支持
- 合同与条款配置
- 法务支持
- 审批流托管
- 支付与托管
- API/沙箱/计算环境开通
- 任务执行支持
- 技术对接支持
- 验收支持

## 22.3 交易后服务

- 账单/分账/发票
- 续费/续权/扩权
- 违规预警
- 权限回收
- 争议处理
- 审计报告
- 使用分析报告
- 融资证明/交易证明
- 产品优化反馈
- 信用评级更新
- 公开验证回执与凭证状态查询
- 公链展示撤回 / 隐藏 / 吊销服务

---

## 23. 审查日志与完整交易信息链追踪

## 23.1 平台必须记录的关键事件

### 主体侧
- 注册
- 实名/实企认证
- 资质变更
- 风险评级变更
- 账号冻结/解冻

### 产品侧
- 资源登记
- 产品创建/修改/下架
- 审核意见
- 版本变化
- 样本变化
- 合规标签变化

### 交易侧
- 询价
- 报价
- 订单创建
- 合同签署
- 审批通过/拒绝
- 支付
- 交付
- 验收
- 结算
- 退款/赔付
- 公链锚定批次生成与上链回执
- 凭证签发 / 吊销 / 失效 / 展示切换

### 使用侧
- 下载
- API 调用
- 沙箱登录
- 查询执行
- 算法上传
- 计算任务执行
- 结果导出
- 续权/扩权
- 断权

### 风控与争议侧
- 风险命中
- 冻结动作
- 告警
- 投诉
- 仲裁
- 惩戒
- 监管调阅

## 23.2 日志分层

### 第一层：业务日志
用于交易与运营

### 第二层：安全日志
用于检测越权、异常访问、攻击行为

### 第三层：证据日志
用于争议、监管、取证，需形成不可抵赖链路

## 23.3 证据链要求

每一笔交易都应能回答：

- 谁发起了交易
- 谁审批了交易
- 谁签了什么版本的条款
- 什么时候开通了什么权限
- 数据是如何交付或如何被计算的
- 买方实际做了哪些调用或任务
- 平台何时做过哪些风险控制
- 是否发生异常/违约/争议
- 结算是否与实际执行一致

---

## 23.4 审计证据对象最小范围

平台必须把以下对象纳入“可导出证据包”的最小范围：
- 交付回执、下载回执、令牌签发记录、API 首次成功调用回执
- 审批意见、合同版本、授权版本、策略生效/撤销记录
- 风险命中记录、冻结/解冻记录、争议材料、裁决单
- 计算任务摘要、训练轮次摘要、模型/结果版本摘要
- 跨链请求、见证签名、目标侧 ack、失败补偿记录
- 公链锚定批次、回执、凭证签发/吊销/失效记录

要求：
- 证据包导出必须按订单、主体、产品、任务四类视角组织
- 证据对象必须支持原文引用与摘要引用双模式
- 审计查看权限与业务查看权限必须分离
- 任何覆盖式更新必须先保留旧版本摘要，避免审计断链

## 24. 运营、信用与生态需求

## 24.1 信用体系
- 主体信用分
- 产品信用分
- 履约分
- 质量分
- 合规分
- 服务分
- 争议率
- 违规记录
- 保证金等级
- 风险观察名单状态

信用体系不应只做展示分数，还必须与交易规则联动：
- 高风险主体可被要求提高保证金、缩小可购/可售范围或进入人工审批
- 信誉持续下降时，应触发降权、限流、暂停上架或暂停扩权
- 优质主体可获得更低保证金、更快审核通道或更高额度
- 评分不能简单平均，必须引入履约结果、争议结果、违规处罚与时间衰减
- 平台应识别互刷评分、洗订单抬分、争议后报复性低分等异常行为

## 24.1.1 保证金与信誉联动机制
- V1 应支持买方保证金、卖方履约保证金或等价风险托管机制
- 保证金并非所有交易都强制收取，但高风险、高金额、低信誉订单必须可配置要求保证金
- 保证金应与订单状态、争议状态、赔付责任、自动验收结果联动
- 信誉变化应影响后续保证金策略，而不是只影响展示页评分
- 平台应支持“保证金冻结、释放、扣罚、退回、保留待裁决”五类状态
- V1 保证金状态可人工处理，系统需留痕

## 24.1.2 图风控与异常关系识别
平台在 V2/V3 需要逐步具备“规则引擎 + 图分析”的风险识别能力，用于发现：
- 串谋评分
- 洗订单
- 凭证共享或账号共用
- 异常转售链路
- 高频异常下载群体
- 关联主体绕过黑名单或额度限制

需求阶段至少要预留：主体关系、设备关系、IP 关系、订单关系、评分关系、授权传播关系等图谱输入字段。

## 24.2 生态角色
- 数据清洗服务商
- 合规评估机构
- 法律服务机构
- 质量认证机构
- 安全测评机构
- 隐私计算服务商
- 节点服务商
- 连接器服务商
- 算法开发者
- 咨询与集成服务商

## 24.3 平台运营重点
- 引入高价值供给
- 做深重点行业场景
- 形成标准合同与标准产品模板
- 培育“非副本交易”心智
- 通过案例建立信任
- 形成监管协同机制

---

## 24.4 开发者生态与应用市场
- 建设开发者中心、SDK、沙箱和应用上架审核机制
- 支持第三方在平台上提供算法插件、清洗插件、审计插件、连接器插件
- 对生态应用建立信用分、兼容性测试和安全扫描流程
- 支持按调用量、按插件订阅、按项目实施等多样合作计费方式

## 24.5 供应商履约与服务等级管理
- 平台应对 API、订阅、数据更新、计算任务建立 SLA 模型
- 供方需承诺更新频率、可用性、故障恢复时限和响应机制
- 平台应持续记录履约得分，并影响推荐权重、认证等级与保证金要求
- 对长期失约主体应触发限流、降级、暂停上架或取消认证凭证

## 24.6 数据血缘与可信证明服务
- 平台应提供数据血缘查询、版本差异对比、最近审计状态查询
- 支持对外出具“某结果来自何种数据与何种计算过程”的证明包
- 支持买方对计算任务、模型输出、交付结果发起复核或第三方验证
- 对重要场景应支持生成面向监管、法务、客户的结构化证明材料

## 24.7 安全运营与应急响应中心
- 建立安全运营工作台，统一查看风险告警、审查异常、下载异常、密钥异常、公链同步异常
- 建立应急剧本：冻结主体、停止交付、暂停展示、撤销凭证、通知监管、通知买卖双方
- 建立 KMS/HSM、密钥轮换、签名分权、多重审批机制
- 支持与 SIEM/SOC 对接，形成事件监控、告警归并和审计闭环

## 25. 建议的分阶段产品路线图

## 25.1 V1：可信交易底座

### 目标
把两大行业、五条标准链路跑通，形成可复制的“标准交易底座”。

### 范围
- Tenant / Party / Department / User / Application / Connector / Environment 对象模型
- `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD` 八个标准 SKU 及其对应交付链路
- 自动规则初审 + 人工复核
- 数字合约、电子签章、联盟链存证
- 订单、验收、账单事件、人工结算/半自动分账、争议处理
- 监管视图、证据包导出
- Solana 演示级最小锚定能力（默认关闭）

### 不在 V1 范围内
- C2D/FL/MPC/TEE/ZKP 正式交付
- 生产级 NFT/凭证公开签发
- 跨境自动交付
- 面向大众自然人供给

### V1 退出标准
- 五条标准链路均完成端到端交易验证
- 订单成功率、验收成功率、账单准确率达到上线门槛
- 审计证据包可导出并完成一次模拟监管检查

## 25.2 V2：可用不可见能力层 + 公链展示增强

### 范围
- C2D / 受控计算
- FL / MPC / TEE 接入
- 结果级授权与贡献度分润
- Solana 生产级锚定、公开验证页
- 不可转让交易凭证 / 供应商认证凭证 / 数据产品护照 NFT

### V2 退出标准
- 至少 2 个场景从“副本/API/沙箱”升级到“受控计算”模式
- 公链展示能力在至少 1 个真实客户或合作场景中被使用

## 25.3 V3：可信数据空间、跨域协作与生态外溢

### 范围
- 连接器互联
- 多空间互通
- 跨平台策略协同
- 跨区域/跨境受控流通
- 融资与登记增值服务
- 更成熟的公链事件同步与生态合作

## 25.4 分阶段验收与里程碑要求

需求阶段即应定义分阶段退出标准，避免后续版本“功能堆积但不可验收”。

### V1 最小验收要求
- 至少连续跑通 20 笔以上标准订单，覆盖文件快照/版本订阅、只读共享、API/服务、模板查询 lite、查询沙箱、固定报告/结果产品链路
- 必须同时验证交易闭环、争议闭环、审计闭环、断权闭环
- 保证金/托管、交付回执、自动验收、账单归档至少完成一轮真实演练

### V2 最小验收要求
- 至少完成 1 类受控计算或 1 类联邦协作闭环
- 必须验证策略控制、结果边界、贡献记录或奖励记录可回溯

### V3 最小验收要求
- 至少打通 1 条跨链授权同步或跨链结算确认链路
- 至少落地 1 类图风控告警并支持人工处置
- 必须提供监管只读穿透视图与冻结处置能力

### 里程碑原则
- 不建议三期并行建设
- 应先把 V1 做到稳定运营，再把高敏协作场景迁入 V2，再建设 V3 的跨域互联与风险治理
- 每一阶段都必须先定义“退出标准”，再定义“新增功能”

## 26. 非功能需求（可验收版）

## 26.1 可用性 / 可靠性 SLO
| 模块 | 指标 |
|---|---|
| 门户与下单服务 | 月可用性 ≥ 99.9% |
| 授权与断权服务 | 月可用性 ≥ 99.95% |
| 审计与证据日志 | 数据持久化成功率 ≥ 99.999% |
| 账单与结算服务 | 账单生成成功率 ≥ 99.95%，人工结算指令执行成功率 ≥ 99.9% |
| 灾备 | RPO ≤ 15 分钟，RTO ≤ 4 小时 |

## 26.2 性能 SLO
| 场景 | 指标 |
|---|---|
| 标准下单/合同查看/账单查询 | p95 响应时间 ≤ 2 秒 |
| 权限校验 | p95 ≤ 200ms |
| API 凭证签发 | p95 ≤ 5 秒 |
| 沙箱账号开通 | 90% 订单 ≤ 10 分钟 |
| 审计事件入库 + 上链摘要写入 | 95% 事件 ≤ 1 分钟 |
| V1 公链锚定（如启用） | 批次延迟 ≤ 30 分钟，不影响主流程 |

## 26.3 安全 SLO
| 控制项 | 指标 |
|---|---|
| 高风险操作二次认证覆盖率 | 100% |
| 核心密钥轮换 | 至少每 90 天一次 |
| 审批双人复核覆盖率 | 高风险流程 100% |
| 安全告警响应 | P1 告警 30 分钟内处置，P2 2 小时内处置 |

## 26.4 审计与监管 SLO
| 项目 | 指标 |
|---|---|
| 证据包导出时长 | 单订单 ≤ 10 分钟 |
| 关键链路可追溯率 | 100% |
| 合规审查流转记录留存 | 100% |
| 监管视图数据同步延迟 | ≤ 15 分钟 |

## 26.5 扩展性要求
- 产品类型、合同模板、审批流、计费模板应支持配置扩展。
- 连接器与执行环境应支持插件式扩展。
- 链上锚定目标链应支持后续增加，但不得影响联盟链主链路。

## 26.6 可观测性要求
- 平台应支持联盟链、公链、链下系统三侧统一监控。
- 所有自动化任务（签发、同步、吊销、批量锚定）必须有重试、告警、人工接管机制。
- 应向运营、监管、合作方提供分级查询视图。
- `V1` 正式采用 `OpenTelemetry + Prometheus + Alertmanager + Grafana + Loki + Tempo`。
- 审计日志不得只依赖普通日志系统保存，`audit.*` 仍是证据主链。
- PostgreSQL 只保存关键结构化系统日志镜像、trace 索引、告警事件、工单和观测配置，不承接全量原始应用日志。

## 26.7 测试与演练要求

平台在进入详细设计和实施前，应明确以下测试与演练要求：
- 单元测试：覆盖计费、授权、风控命中、状态流转、赔付计算等核心规则
- 集成测试：覆盖链上链下状态一致性、对象存储交付、消息投影、回执回写
- 安全测试：覆盖越权、令牌泄露、对象越权下载、密钥轮换、合约升级风险
- 性能测试：覆盖目录查询、下单、交付、验收、账单归集、审计检索
- 容灾测试：覆盖链节点异常、数据库切换、消息队列积压、证书吊销与恢复
- UAT：覆盖真实业务角色的完整演练，包括争议、冻结、解冻、续权、撤权
- V1 允许以脚本化演练 + 手工验证替代完整自动化演练平台


最低要求是：V1 上线前必须完成一次以“交易成功 + 争议成立 + 冻结处置 + 恢复”为主线的全链路演练。

## 27. 进入研发前必须锁死的下位文档

以下 6 份文档不是“建议”，而是进入研发排期前必须冻结的基线附件：

### 27.1 《V1 范围锁定表》
明确 Must / Should / Won’t，冻结产品类型、行业、角色、交付方式、公链开关。

### 27.2 《首批场景清单》
至少包含 2 个行业、5 条标准交易链路、对应 SKU、合同模板和验收模板。

### 27.3 《权利与产品模型矩阵》
按产品类型明确可售权利、默认限制、合同模板、计费模式、交付方式、验收方式。

### 27.4 《审查与风控决策表》
把输入字段、命中规则、审批层级、阻断点、证据材料、输出动作冻结下来。

### 27.5 《订单-验收-结算状态机》
按 `FILE_STD / FILE_SUB`、`SHARE_RO`、`API_SUB / API_PPU`、`QRY_LITE`、`SBX_STD`、`RPT_STD`、`C2D(V2)` 分别定义状态流转、失败分支、补救分支和账单事件源；若使用共享状态机，必须另附 SKU 差异点矩阵。

### 27.6 《主体与权限对象模型》
冻结 Tenant、Party、Department、User、Application、Connector、Execution Environment 之间的关系。


## 28. 本产品的推荐落地策略（结论）

### 28.1 推荐主路线
**联盟链主链路 + Solana 公链展示层 + 数字身份 + 数字合约 + 受控交付 + 隐私计算 + 全流程审计**

### 28.2 推荐优先交易模式
优先顺序建议为：

**V1：**
1. API / 订阅
2. 查询沙箱
3. 定制报告服务
4. 低敏副本下载

**V2 以后：**
5. Compute-to-Data
6. 联邦学习 / MPC
7. ZKP 证明服务
8. 模型/算法能力交易

### 28.3 推荐优先服务对象
- 企业供方与企业买方
- 公共数据授权运营主体
- 医疗/金融/工业等高价值但高敏场景中的受控协作主体
- 科研/行业联盟中的联合建模主体

### 28.4 不建议的误区
- 把“区块链”理解成“数据全上链”
- 把“数据交易”理解成“文件下载商城”
- 把“确权”简单理解成“绝对所有权”
- 把“合规”放到交易后补救
- 把“个人数据市场”作为第一阶段主战场
- 把“跨境自动流通”作为初期默认能力

---

## 29. 正文结论

这个平台要做的，不是“一个区块链概念的数据商城”，而是：

**一个以联盟链为可信底座、以 Solana 公链为公开验证与展示补充层，以数字合约为规则执行核心，以受控交付与隐私保护计算为主要交付方式，以全流程审计和监管协同为保障的数据流通交易基础设施。**

它要解决的不只是“如何成交”，更是：

- 谁能交易
- 卖的到底是什么权利
- 数据如何在不失控的前提下被使用
- 如何做到合规、安全、可审计
- 如何把一次性交易升级为可持续授权和价值共创

如果后续进入详细 PRD/原型阶段，建议下一步优先输出四份下位文档：

1. 《角色与权限矩阵详细稿》
2. 《数据产品分类与交易模式详细稿》
3. 《合规审查与风控规则详细稿》
4. 《链上链下技术架构与能力边界稿》

---

## 30. 参考依据（供后续详细设计继续展开）

### 30.1 用户提供的核心参考
- 《核心交易链路》（用户上传文档）

### 30.2 国内政策与规则
1. 《中共中央 国务院关于构建数据基础制度更好发挥数据要素作用的意见》
   - https://www.nia.gov.cn/n794014/n1050181/n1050479/c1562809/content.html
2. 国家数据局《可信数据空间发展行动计划（2024—2028年）》
   - https://www.nda.gov.cn/sjj/zwgk/zcfb/1122/20241122164142182915964_pc.html
3. 国家数据局《构建数据基础制度更好发挥数据要素作用2025年工作要点》
   - https://www.nda.gov.cn/sjj/swdt/sjdt/0428/20250428132338848329482_pc.html
4. 《上海市数据交易场所管理实施暂行办法》及解读
   - https://app.sheitc.sh.gov.cn/sjxwxgwj/694679.htm
   - https://sheitc.sh.gov.cn/sjxwxgzcjd/20230322/ff2f0510ca4c45e6a020ccea7332f9cd.html
5. 深圳地方标准《数据交易合规评估规范》
   - https://sf.sz.gov.cn/ztzl/hg/hgglbzyzy/content/post_11938681.html
6. 《中华人民共和国数据安全法》
   - https://www.npc.gov.cn/npc/c2/c30834/202106/t20210610_311888.html
7. 《中华人民共和国个人信息保护法》
   - https://www.cac.gov.cn/2021-08/20/c_1631050028355286.htm
8. 《促进和规范数据跨境流动规定》
   - https://www.cac.gov.cn/2024-03/22/c_1712776611775634.htm

### 30.3 国际与行业参考
1. European Data Governance Act
   - https://digital-strategy.ec.europa.eu/en/policies/data-governance-act
2. EU Data Act
   - https://digital-strategy.ec.europa.eu/en/policies/data-act
3. Ocean Protocol – Compute-to-Data
   - https://docs.oceanprotocol.com/developers/compute-to-data
4. International Data Spaces（IDS）数据主权说明
   - https://internationaldataspaces.org/why/data-sovereignty/
5. Gaia-X
   - https://gaia-x.eu/about/
6. Catena-X
   - https://catena-x.net/ecosystem/overview/
7. Solana Programs / Accounts / Token Extensions / Non-Transferrable Tokens / Metadata Pointer
   - https://solana.com/docs/core/programs
   - https://solana.com/docs/core/accounts
   - https://solana.com/docs/tokens/extensions
   - https://solana.com/docs/tokens/extensions/non-transferrable-tokens
   - https://solana.com/docs/tokens/extensions/metadata
8. Wormhole Introduction
   - https://wormhole.com/docs/protocol/introduction/





---

## 31. 附录总览

第 31 章至第 49 章用于补充模板、定价、赔付、字段字典、版本映射、开发调试、支付、认证、审计、一致性、搜索、推荐、可观测性与交易链监控等落地约束。

附录内容与正文共同构成当前版本需求基线；如存在重复表达，以当前保留后的最终定义和最终对照表为准。

## 32. V1 模板基线清单

## 32.1 目标

本清单用于锁死 V1 的标准交易模板，避免开发过程中因 SKU、合同、验收、退款、计费口径不统一而反复返工。

V1 原则如下：

- 只支持**标准化交易模板**，不支持复杂自由组合合同。
- 统一按 **6 种 V1 商品交付形态、8 个标准 SKU** 收口：`FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD`。
- 每种 SKU 必须绑定一套默认模板：
  - 合同模板
  - 授权模板
  - 交付模板
  - 验收模板
  - 退款/赔付模板
  - 计费模板
- V1 页面和订单引擎只允许选择平台支持的模板，不开放任意字段拼装。

## 32.2 V1 支持的标准 SKU 总表

| 商品交付形态 | 标准 SKU | 默认交付方式 | 默认售卖权利 | 默认计费方式 | 默认验收方式 | 默认退款规则 |
|---|---|---|---|---|---|---|
| 文件快照 | FILE_STD | 一次性文件下载/限次下载 | 使用权 + 有限内部共享权（可选） | 一次性收费 | 下载回执 + 时效验收 | 未下载可退款；下载后仅按质量争议退款 |
| 版本订阅 | FILE_SUB | 周期文件更新 | 使用权 + 有限内部共享权（可选） | 月/季/年订阅 | 首次交付验收 + 周期履约 | 未到首交付日前可退款；交付后按剩余周期处理 |
| 只读共享 | SHARE_RO | linked dataset / datashare / share grant | 访问权 + 查询权 + 结果获取权 | 周期费 / 订阅费 | 开通验收 + 可查询性验收 | 未开通可退款；开通后按 SLA/共享可用性规则 |
| API 订阅 | API_SUB | API Token / Key | 访问权 | 月/季/年订阅 + 调用量配额 | 连通性自动验收 | 首次未开通可退款；开通后按 SLA/质量规则 |
| API 按次 | API_PPU | API 调用 | 访问权 | 按次 / 按量 | 成功返回计费 | 已成功调用部分不退款 |
| 模板查询 lite | QRY_LITE | 白名单模板查询 / 受限结果交付 | 查询权 + 结果获取权 | 按次 / 周期 / 套餐 | 模板执行成功 + 输出边界验收 | 未开通可退款；已执行按结果争议处理 |
| 查询沙箱 | SBX_STD | 平台受控沙箱访问 | 查询权 + 结果获取权（必要时可附有限内部共享权） | 项目包 / 周期费 | 开通验收 + 运行可用性验收 | 未开通可退款；开通后按服务可用性处理 |
| 固定报告/结果产品 | RPT_STD | PDF/CSV/分析报告 | 结果获取权 + 使用权 + 有限内部共享权（可选） | 一次性收费 | 文件/结果回执验收 | 未交付可退款；交付后按内容不符争议处理 |

## 32.3 各 SKU 对应模板基线

#### 32.3.1 文件包模板基线

**适用场景**：标准数据集、脱敏数据包、标签文件、样本包、统计数据包。
**默认合同模板**：`CONTRACT_FILE_V1`
**默认授权模板**：`LICENSE_FILE_USE_V1`
**默认验收模板**：`ACCEPT_FILE_V1`
**默认退款模板**：`REFUND_FILE_V1`

固定字段：

- 产品名称
- 版本号
- 数据覆盖时间范围
- 更新频率
- 文件格式
- 文件大小区间
- 字段清单摘要
- 数据分级
- 使用限制
- 下载有效期
- 最大下载次数
- 是否允许离线保存

可配置字段：

- 交付文件数量
- 到期时间
- 是否支持历史版本补发
- 是否附带样例文件
- 是否开启水印

默认限制：

- 不允许转售
- 不允许再授权
- 不允许对外公开传播
- 默认不允许训练公开模型
- 默认只允许约定主体内部使用

#### 32.3.2 API 模板基线

**适用场景**：指标查询、画像查询、风控分、数据验证、标准接口调用。
**默认合同模板**：`CONTRACT_API_SUB_V1 / CONTRACT_API_PPU_V1`
**默认授权模板**：`LICENSE_API_SUB_V1 / LICENSE_API_PPU_V1`
**默认验收模板**：`ACCEPT_API_SUB_V1 / ACCEPT_API_PPU_V1`
**默认退款模板**：`REFUND_API_SUB_V1 / REFUND_API_PPU_V1`

固定字段：

- API 名称
- 版本号
- 基础 URL / 网关路由
- 授权方式
- QPS 限制
- 日/月调用额度
- 返回格式
- SLA 等级
- 错误码规范
- 支持环境（生产/测试）

可配置字段：

- 配额大小
- 白名单 IP 数量
- Token 有效期
- 是否支持沙箱测试
- 是否支持回调
- 是否支持专线/专网接入

默认限制：

- 默认只允许调用结果，不返回底层原始数据表
- 默认不得缓存超过约定时长
- 默认不得用作模型训练数据集沉淀
- 默认只允许绑定指定应用和环境使用

#### 32.3.3 沙箱模板基线

**适用场景**：可用不可见分析、联合调试、受控可视化查询、项目制协作。
**默认合同模板**：`CONTRACT_SANDBOX_V1`
**默认授权模板**：`LICENSE_SANDBOX_USE_V1`
**默认验收模板**：`ACCEPT_SANDBOX_V1`
**默认退款模板**：`REFUND_SANDBOX_V1`

固定字段：

- 沙箱名称
- 项目编号
- 可访问数据集列表
- 允许角色清单
- 运行时长
- 会话并发限制
- 导出规则
- 审批要求
- 结果导出格式
- 日志保留周期

可配置字段：

- 项目周期
- 可用算力档位
- 可创建 Notebook 数量
- 允许安装的依赖包范围
- 是否支持结果集导出审批

默认限制：

- 不允许原始表导出
- 默认仅允许导出聚合结果、图表、审核通过的结果文件
- 所有操作留痕
- 默认绑定项目与指定成员，不允许共享登录

#### 32.3.4 报告/结果包模板基线

**适用场景**：定制报告、统计结果、评分结果、核验结果。
**默认合同模板**：`CONTRACT_REPORT_V1`
**默认授权模板**：`LICENSE_RESULT_USE_V1`
**默认验收模板**：`ACCEPT_REPORT_V1`
**默认退款模板**：`REFUND_REPORT_V1`

固定字段：

- 交付件类型
- 交付时间
- 报告目录 / 字段说明
- 适用范围
- 使用期限
- 引用限制
- 是否允许二次编辑

可配置字段：

- 结果格式（PDF/CSV/JSON）
- 交付频率
- 结果置信度说明
- 是否附原始说明附件

默认限制：

- 默认不附带原始明细数据
- 默认不得将结果反向还原为原始样本
- 默认不可作为公开宣传材料

### 32.4 V1 模板与系统模块映射

| 模板类型 | 系统落点 | 是否 V1 必做 |
|---|---|---|
| 合同模板 | 下单页、订单详情、电子签章 | 是 |
| 授权模板 | SKU 配置、权限引擎、交付控制 | 是 |
| 验收模板 | 订单状态机、售后页 | 是 |
| 退款模板 | 售后规则引擎、财务对账 | 是 |
| 计费模板 | 报价页、账单引擎、结算引擎 | 是 |
| 赔付模板 | 售后争议、SLA 处理 | 是 |
| 公链展示模板 | 凭证页、公示页 | 否（V1 预留） |

### 32.5 V1 固定字段与可配字段总原则

V1 所有模板统一采用“**固定字段优先，可配字段有限开放**”原则：

- 固定字段：必须存在，参与搜索、审核、合同生成和审计。
- 可配字段：由管理员在 SKU 建档时填写，但不得新增字段类型。
- 不支持：任意动态表单设计器、复杂审批流引擎、复杂合同条款编排。

---

## 33. 报价与定价规则稿

### 33.1 定价目标

V1 定价规则应满足四个要求：

1. 买方能快速理解价格组成。
2. 卖方能明确知道平台支持哪些收费模式。
3. 账单引擎能直接根据事件生成账单。
4. 续费、超额计费、退款、赔付可以按固定公式处理。

### 33.2 V1 支持的定价模式

| SKU 类别 | V1 支持定价模式 | V1 不支持 |
|---|---|---|
| 文件包 | 一次性固定价、周期订阅价 | 动态竞价、按字段差异实时调价 |
| API | 月/季/年套餐、按次、超额计费 | 复杂阶梯折扣、实时竞价 |
| 沙箱 | 项目包价、按周期租用 | 按 CPU/内存秒级计费 |
| 报告/结果包 | 一次性固定价、定期交付价 | 按效果分成 |

### 33.3 价格结构

所有 SKU 的最终成交价由以下部分组成：

`成交价 = 基础商品价 + 平台服务费 + 可选增值服务费 + 税费 + 超额费用（如有） - 优惠金额`

说明：

- 基础商品价：卖方对产品本身的报价。
- 平台服务费：平台抽成或技术服务费。
- 可选增值服务费：如专线接入、额外存储、定制报告、加急交付。
- 税费：按实际开票与税率处理。
- 超额费用：仅适用于 API 或周期类沙箱服务。
- 优惠金额：仅支持平台统一优惠券/折扣，不支持复杂组合营销。

### 33.4 各 SKU 默认定价规则

#### 33.4.1 文件包

支持：

- 一次性固定价
- 周期更新订阅价（月/季/年）

默认报价字段：

- 基础价
- 有效期
- 是否含历史版本
- 是否含后续更新
- 允许下载次数

默认计费规则：

- 一次性文件包：下单即锁价，支付成功后进入交付。
- 周期文件订阅：按订阅周期预付。
- 不支持按下载次数重复收费。

#### 33.4.2 API

支持：

- 套餐价（月/季/年）
- 按次计费
- 套餐 + 超额计费

默认报价字段：

- 套餐周期
- 包含调用量
- 超额单价
- QPS 档位
- 白名单应用数量
- 是否含测试环境

默认计费规则：

- 套餐类：预付费，按账期生效。
- 按次类：充值或后付费二选一，V1 默认推荐预充值。
- 超额部分：当期账单结算，不跨期冲抵。

#### 33.4.3 沙箱

支持：

- 项目包价
- 周期租用价（月）
- 项目包 + 结果导出附加费

默认报价字段：

- 项目周期
- 可使用成员数
- 算力档位
- 可访问数据集范围
- 默认导出次数

默认计费规则：

- 项目启动前预付。
- 项目延期单独生成补充订单。
- 额外导出审批通过后生成附加费用。

#### 33.4.4 报告/结果包

支持：

- 固定项目价
- 定期交付价（周/月）

默认报价字段：

- 报告类型
- 交付频率
- 交付周期
- 是否含复核版本

默认计费规则：

- 固定项目价：预付或 50/50 分期，V1 默认支持两档。
- 定期交付：按账期预付。

### 33.5 报价审批简化原则（个人项目适用）

V1 不做复杂报价审批流，只做两级：

- 系统自动校验：价格字段是否完整、是否满足最低配置、是否与 SKU 模板一致。
- 管理员人工确认：仅在以下场景触发：
  - 价格低于最低阈值
  - 使用了特批折扣
  - 使用了非标准付款方式
  - 涉及大额订单

### 33.6 账单事件源定义

| 事件 | 是否生成账单 | 说明 |
|---|---|---|
| 订单支付成功 | 是 | 生成主账单 |
| API 套餐开通 | 否 | 仅更新服务状态 |
| API 超额调用结算日到达 | 是 | 生成超额账单 |
| 沙箱延期审批通过 | 是 | 生成补充账单 |
| 结果导出审批通过且收费 | 是 | 生成附加账单 |
| 退款成功 | 是（负账单） | 冲减原账单 |
| SLA 赔付确认 | 是（赔付账单） | 可冲减或返现 |

### 33.7 报价与定价的 V1 范围边界

V1 不做：

- 动态竞价引擎
- 千人千价推荐
- AI 自动估值
- 复杂促销编排
- 多币种链上结算

---

## 34. 争议责任与赔付规则稿

### 34.1 目标

本规则用于统一处理：

- 验收失败
- 质量争议
- SLA 争议
- 退款
- 赔付
- 权限误开通/误停用

V1 原则：**先规则化、后自动化；先标准争议、后复杂仲裁。**

### 34.2 基本责任划分

| 争议类型 | 主要责任方 | 平台责任 | 处理方式 |
|---|---|---|---|
| 商品描述与实际不符 | 卖方 | 提供证据与裁定辅助 | 退款/补交付/赔付 |
| 数据质量低于承诺指标 | 卖方 | 记录质检与处理证据 | 部分退款/补交付 |
| API 长时间不可用 | 卖方或平台（视故障归属） | 提供日志和可用性证据 | SLA 赔付/延期补偿 |
| 平台系统导致无法交付 | 平台 | 直接责任 | 全额退款或赔付 |
| 买方误操作/超范围使用 | 买方 | 留痕与处置 | 不退款/限制账户 |
| 买方越权使用/违规导出 | 买方 | 风控拦截、证据保留 | 冻结权限/追责 |
| 因监管要求被阻断 | 非单方责任 | 提供通知与流程留痕 | 终止或部分退款 |

### 34.3 证据优先级

争议处理时，证据优先级按如下顺序：

1. 链上交易与授权存证摘要
2. 电子签章合同与授权版本
3. 交付回执 / API 调用日志 / 沙箱操作日志
4. 质量检测报告 / 验收报告
5. 平台系统日志 / 告警日志
6. 买卖双方补充说明

### 34.4 各 SKU 默认争议与赔付规则

#### 34.4.1 文件包

可发起争议的典型原因：

- 文件无法下载
- 文件损坏
- 字段清单与说明不符
- 数据覆盖周期明显缺失

默认处理：

- **未下载前**：可全额退款。
- **下载后 48 小时内** 且证明确有质量问题：可申请补交付或部分/全额退款。
- 因买方本地环境问题无法打开：原则上不退款，平台提供一次技术协助。

#### 34.4.2 API

可发起争议的典型原因：

- 未按时开通
- 连通失败
- 可用性低于 SLA
- 返回值结构与文档严重不一致

默认处理：

- 未开通成功：可全额退款。
- 已开通但 24 小时内持续不可用：按未开通处理。
- 当月可用性低于承诺阈值：按阶梯赔付，优先赔服务时长，其次返现。
- 因买方超额或违反调用规范导致失败：不赔付。

#### 34.4.3 沙箱

可发起争议的典型原因：

- 约定时间未开通
- 成员无法登录
- 允许访问的数据集与约定不符
- 导出规则与合同不一致

默认处理：

- 未开通：全额退款。
- 晚开通：按延误时间补偿服务期。
- 因卖方未按约提供数据集导致项目受阻：补开通或部分退款。
- 因买方自定义脚本问题导致任务失败：不退款。

#### 34.4.4 报告/结果包

可发起争议的典型原因：

- 未按时交付
- 交付件缺页/缺字段
- 结果口径与约定不符

默认处理：

- 未交付：可退款。
- 可修复问题：优先补交付。
- 口径明显不符且无法修复：部分或全额退款。

### 34.5 SLA 赔付建议口径

V1 建议采用简单档位：

| 当月可用性 | 默认处理 |
|---|---|
| ≥ 99.5% | 无赔付 |
| 99.0% - 99.5% | 补偿 3 天服务期 |
| 98.0% - 99.0% | 补偿 7 天服务期或等值代金券 |
| < 98.0% | 补偿 15 天服务期或部分现金赔付 |

### 34.6 退款与赔付边界

V1 统一规则：

- 不因“买方主观认为数据价值不高”直接退款。
- 不因已明确披露的限制项退款。
- 不因买方自行超出授权用途被限制而退款。
- 平台赔付责任默认不超过该订单平台服务费部分，除非平台直接导致全链路失败。
- 卖方赔付上限默认不超过订单实收金额，除非合同另有约定。

### 34.7 争议处理最小流程

1. 买方发起争议
2. 系统自动收集订单、模板、日志、回执
3. 管理员初判：是否命中标准规则
4. 命中标准规则：系统输出建议处理结果
5. 未命中：转人工处理
6. 处理结果写入订单、账单与审计日志

---

## 35. 元数据字段字典与搜索索引规范

### 35.1 目标

本规范用于统一：

- 上架表单字段
- 产品详情页字段
- 审核页字段
- 搜索筛选字段
- 风控输入字段
- 合同模板映射字段

V1 原则：**同一字段多处复用，避免“上架一套字段、搜索一套字段、审核再一套字段”。**

冻结要求：

- 本章字段字典是商品中心、审核中心、搜索中心、合同中心、风控中心、推荐中心的**唯一元数据字段源**
- 其他章节允许解释字段含义，但不得自行扩展冲突枚举或重写字段口径
- 若出现字段冲突，以本章和第 48、49 章的冻结口径为准

### 35.2 字段分层

| 字段层级 | 用途 | V1 要求 |
|---|---|---|
| 核心识别字段 | 用于产品识别与搜索 | 必填 |
| 交易字段 | 用于报价、合同、交付 | 必填 |
| 合规字段 | 用于审核、限制和风控 | 必填 |
| 质量字段 | 用于质量展示与筛选 | 建议必填 |
| 展示字段 | 用于产品详情展示 | 可选 |
| 扩展字段 | 行业特有补充字段 | V1 预留 |

### 35.2A 核心建模轴冻结

| 字段轴 | 对应字段 | 角色 | 约束 |
|---|---|---|---|
| 权利主轴 | `rights_type` | 唯一权利枚举源 | 只能使用 `use / access / query / result_get / internal_share_limited` 作为 V1 正式售卖值 |
| SKU 主轴 | `sku_type` | 标准可售对象枚举 | 只能使用 8 个标准 SKU 编码 |
| 交付轴 | `delivery_mode` | 表达交付路径 | 不得替代权利枚举，不得单独派生新 SKU |
| 计费轴 | `pricing_mode` | 表达计费路径 | 不得替代权利枚举，不得单独派生新 SKU |
| 产品分类轴 | `product_type` | 目录与展示分类 | 不得替代 `rights_type` 或 `sku_type` |
| 派生组合轴 | `rights_profile_code` | 模板映射、展示、检索标签 | 可由 `rights_type + sku_type + delivery_mode + pricing_mode` 派生，但不构成独立枚举源 |

### 35.3 V1 核心元数据字段字典

| 字段名 | 英文字段建议 | 类型 | 是否必填 | 是否搜索索引 | 是否筛选项 | 说明 |
|---|---|---|---|---|---|---|
| 产品 ID | product_id | String | 是 | 是 | 否 | 系统唯一编码 |
| SKU 类型 | sku_type | Enum | 是 | 是 | 是 | `FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD` |
| 产品名称 | product_name | String | 是 | 是 | 否 | 标题搜索主字段 |
| 产品副标题 | product_subtitle | String | 否 | 是 | 否 | 补充关键词 |
| 商品类型 | product_type | Enum | 是 | 是 | 是 | `data_product / service_product / result_product` |
| 所属行业 | industry | Enum | 是 | 是 | 是 | 金融/政务/供应链等 |
| 应用场景 | use_case | Enum/List | 是 | 是 | 是 | 风控、营销、核验等 |
| 卖方主体名称 | seller_name | String | 是 | 是 | 是 | 展示与审计 |
| 标签集合 | tag_set | List | 是 | 是 | 是 | 行业标签/场景标签/关键词标签 |
| 数据来源说明 | data_source_desc | Text | 是 | 否 | 否 | 审核重点字段 |
| 覆盖时间范围 | coverage_time | String | 是 | 否 | 是 | 例：2024-01 至 2025-12 |
| 更新频率 | refresh_frequency | Enum | 是 | 是 | 是 | 一次性/日/周/月 |
| 数据分级 | data_classification | Enum | 是 | 是 | 是 | 公开/内部/敏感/高敏 |
| 是否含个人信息 | contains_pi | Bool | 是 | 否 | 是 | 合规输入字段 |
| 是否含敏感个人信息 | contains_spi | Bool | 是 | 否 | 是 | 合规输入字段 |
| 是否含重要数据 | contains_important_data | Bool | 是 | 否 | 是 | 合规输入字段 |
| 脱敏状态 | masking_status | Enum | 是 | 是 | 是 | 未脱敏/已脱敏/匿名化 |
| 权利类型 | rights_type | Enum/List | 是 | 是 | 是 | `use / access / query / result_get / internal_share_limited` |
| 权利组合标签 | rights_profile_code | String | 否 | 是 | 否 | 派生字段，不构成独立权利枚举源 |
| 交付方式 | delivery_mode | Enum | 是 | 是 | 是 | `file_download / revision_push / share_grant / api_access / template_grant / sandbox_workspace / report_delivery` |
| 计费模式 | pricing_mode | Enum | 是 | 是 | 是 | `one_time / subscription / pay_per_use / project_fee` |
| 基础价格 | base_price | Number | 是 | 否 | 是 | 排序/筛选使用 |
| 最小起订周期 | min_term | String | 否 | 否 | 是 | 如 1 月 / 1 次 |
| 质量评分 | quality_score | Number | 否 | 是 | 是 | 平台内部或展示用 |
| 卖方信誉分 | seller_reputation_score | Number | 否 | 是 | 是 | 搜索排序与展示用 |
| 卖方风险等级 | seller_risk_level | Number | 否 | 是 | 是 | 排序惩罚与过滤用 |
| 样例可用性 | sample_available | Bool | 是 | 是 | 是 | 是否可预览 |
| 上架状态 | listing_status | Enum | 是 | 否 | 是 | 草稿/审核中/已上架/下架 |

字段命名说明：

- 本章的 `delivery_mode / pricing_mode` 是对外逻辑字段名，用于页面、接口、搜索和合同语义统一。
- 数据库持久化层若仍保留 `catalog.product.delivery_type / price_mode`，应视为等价映射，不得再派生第三套含义。
- 本章的 `product_type` 仅用于目录与展示分类，不等同于第 `48` 章的商品交付形态，也不得替代 `sku_type`。
- `pricing_mode` 表达商品或 SKU 的对外收费模式；具体账单归集、履约计量和结算计算一律由 `Billing Event` 与计费模板驱动，不再单独引入与之并列的正式 `billing_mode` 字段。

### 35.3.1 十大元信息域最小要求

- 业务描述：用途、适用行业、使用场景、目标买方、禁止用途。
- 结构描述：字段清单、字段类型、主键/时间字段、编码规则、版本结构摘要。
- 质量描述：缺失率、覆盖范围、刷新频率、采样方式、异常率、质量评分。
- 安全描述：敏感等级、是否含个人信息、脱敏方式、样本掩码策略。
- 合规描述：地域限制、用途限制、导出限制、证据要求、特殊前置许可。
- 交付描述：文件/API/共享/模板查询/结果产品的交付方式、可用时效和限额。
- 版本描述：`revision`、发布日期、变更说明、是否支持未来更新、回滚说明。
- 验收描述：买方验收口径、阈值、拒收理由模板、争议触发条件。
- 授权描述：授权范围、是否允许下载、是否允许转授权、是否允许派生输出。
- 责任描述：提供者、加工者、平台、第三方评估方和各自责任边界。

### 35.3.2 数据契约最小内容

每个上架 SKU 对应的数据契约至少应固化：

- 交付义务与交付对象
- 验收标准与拒收判定口径
- 用途、地域、导出、时效和派生限制
- 质量承诺与质量异议边界
- 加工责任链与责任归属
- 争议证据口径与赔付触发条件

### 35.3.3 数据原样处理与产品化加工对象

`V1` 起平台必须显式建模以下对象：

- 原始接入批次
- 原始对象清单
- 格式识别结果
- 抽取/标准化任务
- 预览或样例工件
- 加工责任链任务

这些对象不是实现细节，而是交易前“原始数据 -> 可卖数据商品”链路的正式业务对象。

### 35.3.4 数据商品分层存储对象

`V1` 起平台必须显式建模以下存储对象：

- 存储命名空间
- 存储策略档案
- 资产版本存储绑定
- 分层存储区标识（`raw / curated / product / preview / delivery / archive / evidence / model`）
- 交付对象登记

这些对象用于表达“对象放在哪里、属于哪个分层区域、采用什么生命周期策略、由谁托管、是否进入正式交付面”，不允许只靠自由文本描述。

### 35.3.5 数据商品查询与执行面对象

`V1` 起平台必须显式建模以下查询/执行对象：

- 查询面定义
- 查询模板定义
- 查询面授权
- 查询执行运行记录
- 查询结果对象/报告结果对象

这些对象用于表达“哪个版本通过何种入口被查询、允许执行哪些模板、谁被授权、每次执行产生了什么结果与证据”，不允许只把模板摘要塞进授权表后省略运行链路。

### 35.3.6 敏感数据受控交付对象

`V1` 起平台必须显式建模以下敏感治理对象：

- 敏感处理策略档案
- 处理依据证明
- 安全预览工件
- 敏感执行策略快照
- 执行证明/远程证明记录
- 结果披露审查记录
- 销毁/保留证明

这些对象用于表达“为什么可以处理、允许如何交付、允许如何执行、输出到什么范围、谁审批过、到期后如何撤权和销毁”，不允许只靠商品备注或工单文本表达。

### 35.4 V1 搜索索引规范

#### 35.4.1 全文索引字段

- 产品名称
- 产品副标题
- 行业标签
- 场景标签
- 卖方名称
- 关键词标签
- 接口/数据集简述
- 卖方行业标签

#### 35.4.2 精确筛选字段

- SKU 类型
- 行业
- 应用场景
- 更新频率
- 交付方式
- 权利类型
- 数据分级
- 是否含个人信息
- 是否可样例预览
- 价格区间
- 卖方主体类型
- 标签集合
- 搜索对象范围（商品/服务/卖方）

#### 35.4.3 排序字段

- 综合排序（默认）
- 最新上架
- 价格
- 质量评分
- 信誉
- 热度
- 近期成交量（V1 可内部使用，前台可选隐藏）
- 更新日期

### 35.5 上架页、详情页、审核页字段最小集合

#### 上架页必须出现

- SKU 类型
- 产品名称
- 场景标签
- 数据来源说明
- 数据分级
- 权利类型
- 交付方式
- 计费模式
- 价格
- 使用限制
- 样例说明

#### 详情页必须出现

- 产品概述
- 适用场景
- 交付方式
- 更新频率
- 权利范围
- 使用限制
- 价格说明
- 样例入口
- 卖方信息
- 审核状态摘要

#### 审核页必须出现

- 主体信息
- 来源说明
- 分级标记
- PI/SPI/重要数据标记
- 脱敏状态
- 权利声明
- 合同模板选择
- 风险提示
- 审核结论

### 35.6 搜索与元数据的 V1 边界

V1 不做：

- 自定义复杂 DSL 搜索
- 首页强制向量推荐
- 自动标签抽取纠错
- 跨模态搜索

V1 推荐边界：

- 必做规则型推荐与个性化发现
- 不做复杂千人千面与强模型依赖推荐
- 推荐可使用搜索投影、行为聚合、质量/信誉/热度作为输入
- 推荐最终放行仍回 PostgreSQL 校验

V1 生产环境要求：

- PostgreSQL 继续作为商品、卖方、标签、价格、质量、信誉等主数据与搜索投影权威源
- OpenSearch 作为商品搜索专用读模型
- Redis 仅做搜索缓存与 facet / 自动补全短缓存
- local / demo 环境允许 PostgreSQL fallback 搜索

---

## 36. 版本标记与范围映射表

### 36.1 目标

本表用于防止项目范围失控。后续所有功能、页面、接口、数据库字段和开发任务，都应先映射到以下版本标记：

- **V1 必做**：不做则无法形成可交易、可演示、可验收闭环。
- **V1 预留**：本版不完整实现，但字段、状态或接口应预留。
- **V2 再做**：明确不纳入 V1 交付。
- **V3 再做**：中远期生态能力。

### 36.2 范围映射总表

| 模块 | 子模块 | 版本标记 | 说明 |
|---|---|---|---|
| 账号与主体 | 企业主体注册、KYC/KYB | V1 必做 | 核心准入能力 |
| 账号与主体 | 部门级授权 | V1 预留 | 先用简化角色替代 |
| 账号与主体 | 子账号转授 | V2 再做 | 不纳入首版 |
| 产品中心 | 文件包 SKU | V1 必做 | 首版核心商品 |
| 产品中心 | API SKU | V1 必做 | 首版核心商品 |
| 产品中心 | 沙箱 SKU | V1 必做 | 仅基础受控沙箱 |
| 产品中心 | C2D 任务 SKU | V2 再做 | 暂不承诺正式交付 |
| 产品中心 | MPC / FL / TEE 产品化 | V2 再做 | 首版只保留架构预留 |
| 搜索发现 | 基础搜索/筛选 | V1 必做 | 必须能找得到商品 |
| 搜索发现 | 规则型推荐 / 个性化发现 | V1 必做 | 首页、详情、卖方页、工作台推荐位，首版不做复杂千人千面 |
| 搜索发现 | 模型化推荐 / 个性化排序 | V2 再做 | `V2` 再引入更强协同与模型化推荐 |
| 订单交易 | 标准下单流程 | V1 必做 | 文件快照/版本订阅、只读共享、API/服务、沙箱/模板查询、结果产品 |
| 订单交易 | 询价/竞价 | V1 预留 | 可先保留询单入口 |
| 合同授权 | 标准模板合同 | V1 必做 | 模板化签约 |
| 合同授权 | 条款自由编排 | V2 再做 | 不纳入首版 |
| 交付 | 文件下载 | V1 必做 | |
| 交付 | API 开通 | V1 必做 | |
| 交付 | 沙箱开通 | V1 必做 | |
| 交付 | 结果包交付 | V1 必做 | |
| 验收与售后 | 标准验收与退款 | V1 必做 | |
| 验收与售后 | 复杂仲裁流 | V2 再做 | 首版人工处理 |
| 账单结算 | 基础账单 | V1 必做 | |
| 账单结算 | 自动分润 | V1 预留 | 首版可先人工结算 |
| 风控合规 | 规则命中阻断 | V1 必做 | |
| 风控合规 | 多级审批流 | V1 预留 | 先用管理员确认 |
| 审计存证 | 联盟链关键事件存证 | V1 必做 | |
| 审计存证 | Solana 公开展示锚定 | V1 预留 | 演示级/默认关闭 |
| 公链增强 | 不可转让凭证 / 护照 NFT | V2 再做 | 作为展示增强层 |
| 连接器 | 标准文件上传 | V1 必做 | |
| 连接器 | 第三方系统连接器 | V2 再做 | |
| 执行环境 | 平台内置基础沙箱 | V1 必做 | |
| 执行环境 | 外部可信执行环境接入 | V2 再做 | |
| 生态能力 | 开发者市场 | V2 再做 | |
| 生态能力 | 多方数据空间协同 | V3 再做 | |

### 36.3 开发任务标记规则

后续每条研发任务至少带一个标记：

- `MUST_V1`
- `RESERVE_V1`
- `V2`
- `V3`

示例：

- 订单页生成标准合同 PDF：`MUST_V1`
- Solana 交易凭证可视化页：`RESERVE_V1`
- 不可转让供应商认证 NFT 铸造：`V2`
- 联邦学习任务编排：`V2`

### 36.4 个人项目的范围控制原则

对本项目，额外增加两条控制规则：

1. **任何新功能如果不能直接映射到某个已确定的 V1 SKU，就默认不进入 V1。**
2. **任何功能若需要新增一套独立基础设施（如复杂跨链桥、复杂审批流、独立算力调度系统），默认放入 V2。**

### 36.5 当前版本范围判断补充

当前版本的范围判断补充如下：

- V1 必做：标准注册准入、文件/API/沙箱/结果包、标准模板合同、基础验收退款、基础账单、联盟链存证、基础搜索与审核。
- V1 预留：询价入口、管理员人工审批位、Solana 演示级锚定字段、公链展示页占位。
- V2 再做：不可转让交易凭证、供应商认证凭证、数据产品护照 NFT、C2D/FL/MPC/TEE 正式产品化、复杂连接器。
- V3 再做：跨域可信数据空间、跨平台互认证书、生态市场化协同。

## 37. 附录使用原则

- 页面、接口、数据库、状态机与测试用例设计，应同时遵循正文范围边界和附录冻结规则。
- 附录中的模板、字段、定价、赔付和运行要求，属于研发拆解前的正式输入，不得按个人理解另起一套口径。
- 后续若拆分为独立规格文档，应以本附录内容为直接来源，保持语义一致。

---

## 38. 开发者通道

### 38.1 目标

开发者通道面向本项目研发、联调、演示和测试，用于提供统一开发调试入口。目标是降低以下成本：

- 测试网环境信息分散
- 测试代币获取困难
- SDK / API / 回调调试入口不统一
- Mock 数据、测试沙箱、真实测试网切换成本高
- 链上回执、链下日志、订单状态之间排查困难

### 38.2 V1 定位

V1 建议仅做“**轻量开发者通道**”，不做完整开发者生态平台。

V1 必做：

- 测试网络信息面板
- Faucet / 测试代币获取入口聚合
- API / SDK / OpenAPI 示例入口
- Mock 数据与测试沙箱入口
- 应用凭证管理与调试入口
- 链上回执、订单状态、审计日志联查入口

V1 预留：

- 第三方开发者应用市场
- 插件市场
- 外部开发者认证体系
- 公开开发者社区与积分体系

### 38.3 测试网络信息面板

开发者通道需集中展示每条支持网络的调试信息，包括：

- 网络名称
- 网络类型（联盟链测试环境 / Solana Devnet / 其他测试网）
- RPC 地址
- 浏览器地址
- chain id / cluster 标识
- 原生代币符号
- 当前状态（可用 / 限流 / 故障）
- 官方 faucet 链接
- 额度限制说明
- 推荐钱包/CLI 命令

V1 原则：

- 平台内统一收口，不要求开发者到处找测试环境信息。
- 每条链只维护少量可信入口，避免文档失控。

### 38.4 Faucet / 测试代币支持

当前链上开发中，测试代币获取依然是典型痛点，尤其在以下场景：

- 频繁部署/重部署合约
- 自动化测试需要大量地址和 gas
- 公共 faucet 有频控、冷却时间或登录限制
- 团队联调时多个地址同时领币

因此，V1 开发者通道建议支持：

- 官方 faucet 聚合链接
- 第三方可信 faucet 备用链接
- 每条链的领取限制说明
- 最近一次领取时间记录
- 常见错误说明（额度不足、限流、网络故障）

个人项目的推荐策略：

1. **本地链 / 本地 validator 优先**，减少对公共 faucet 的依赖。
2. **公共测试网仅用于联调、回执验证、演示验证**。
3. **维护少量平台控制的测试钱包**，在额度允许时提前领取测试代币，作为调试缓冲池。
4. **对 faucet 失败场景给出备用入口和手工处理指引**，避免开发中断。

### 38.5 API / 应用调试入口

开发者通道应提供统一的应用调试入口，至少包括：

- 创建测试应用
- 查看/重置 API Key
- 查看配额与限流状态
- 查看最近调用记录
- 查看错误码与失败原因
- 查看 Webhook 投递状态
- 测试回调重放

V1 不要求做复杂的在线 API Playground，但至少要让开发者能快速确认：

- Key 是否已生效
- 权限是否正确
- 配额是否已命中
- 请求是否到达网关
- 响应为何失败

### 38.6 Mock 数据与测试沙箱

V1 建议提供两种调试模式：

- **Mock 模式**：不依赖真实链上回执、不依赖真实外部 KYC/支付/签章服务。
- **联调模式**：接测试环境，验证真实状态流转和回执留痕。

开发者通道应支持：

- 标准 Mock 产品样例
- 测试用文件包
- 测试 API
- 测试沙箱账号
- 典型回调样例
- 标准错误场景样例

### 38.7 链上与链下联查入口

开发者调试最容易卡住的，不是单个接口，而是“链上、链下、审计日志三侧状态不一致”。

因此开发者通道需支持按以下任一主键联查：

- order_id
- contract_id
- bill_event_id
- audit_event_id
- public_anchor_batch_id
- onchain_tx_hash

联查结果至少展示：

- 当前订单状态
- 当前授权状态
- 当前交付状态
- 最近账单事件
- 最近审计事件
- 对应链上摘要/哈希/回执
- 最近错误信息

### 38.8 V1 范围边界

V1 不做：

- 外部开发者开放注册市场
- 自动分发无限测试代币的公开 faucet
- 多链复杂测试资源调度
- 自动化插件商店
- 完整在线 IDE

---

## 39. 开发与调试支持要求

### 39.1 目标

本节用于避免项目在开发阶段因环境复杂、依赖过多、状态不一致而陷入调试低效。
要求重点覆盖：

- 最小运行模式
- Mock 与真实依赖切换
- 链上链下状态一致性
- 调试日志与错误码
- 测试数据与测试账户

### 39.2 最小运行模式

V1 至少定义 3 种运行模式：

1. **本地最小模式**
   - 仅启动核心业务服务、数据库、对象存储、基础网关、Mock 审计。
   - 不依赖真实联盟链、公链、支付、签章、KYC。
   - 适合页面联调、订单流测试、账单规则测试。

2. **联调模式**
   - 接入测试版联盟链/公链、测试签章、测试支付、测试通知服务。
   - 用于验证真实回执和状态流转。

3. **演示模式**
   - 在联调模式基础上启用演示数据、演示账户、演示公链锚定。
   - 用于对外展示和业务演练。

要求：

- 任一模式都必须有清晰的配置项，不允许靠手工改代码切换。
- 本地最小模式必须能独立跑通至少 1 条文件链路和 1 条 API 链路。

### 39.3 Mock 与真实依赖切换

以下外部依赖必须支持 Mock 开关：

- KYC / KYB
- 电子签章
- 支付 / 发票
- 联盟链写入
- 公链锚定
- 通知 / 短信 / 邮件
- 风控评分外部依赖

要求：

- 所有外部依赖都必须定义统一适配层。
- Mock 模式应返回稳定、可预测的测试结果。
- 联调模式必须能切换到真实测试环境。
- 禁止在业务代码中散落硬编码“测试分支”。

### 39.4 链上链下状态一致性要求

平台必须显式处理以下调试难点：

- 链下成功、链上失败
- 链上成功、链下未更新
- 重试导致重复写入
- 回调乱序导致状态回退
- 外部回执迟到导致账单或审计不一致

V1 需至少支持以下机制：

- 全局业务事件 ID
- 幂等键
- 出站事件表 / 回执记录表
- 重试次数与退避策略
- 失败补偿任务
- 死信/异常事件列表

要求：

- 任一关键状态流转都必须可追溯其事件来源。
- 重试不得造成重复扣费、重复交付、重复上链。

### 39.5 调试日志与错误码规范

V1 必须统一以下调试主键：

- request_id
- order_id
- contract_id
- sku_id
- bill_event_id
- audit_event_id
- onchain_tx_hash

V1 必须统一以下日志分类：

- 接口访问日志
- 业务状态流转日志
- 链接入/链回执日志
- 外部依赖调用日志
- 风控与审计日志

V1 必须有最小错误码规范：

- 参数错误
- 权限错误
- 合规阻断
- 资源不存在
- 配额超限
- 交付失败
- 链写入失败
- 回执超时
- 账单生成失败

要求：

- 同一错误码需有固定含义和固定排查方向。
- 前端、后端、网关、异步任务不得各自发明一套错误定义。

### 39.6 测试数据、测试账户与测试钱包

V1 应维护一套最小测试资产：

- 测试租户
- 测试供方
- 测试买方
- 测试管理员
- 测试应用
- 测试文件产品
- 测试 API 产品
- 测试沙箱产品
- 测试账单样本
- 测试争议样本

如接入公链/测试网，还应维护：

- 平台测试钱包
- 演示钱包
- 测试代币领取记录
- 最近 faucet 状态说明

要求：

- 测试数据可一键初始化。
- 测试账户和钱包不得与生产环境混用。

### 39.7 调试优先级

开发调试优先级如下：

1. 本地最小模式
2. Mock / Real 切换
3. 链上链下联查
4. 开发者通道页面完善

本地快速复现能力优先于测试代币和界面体验优化。

### 39.8 紧迫性分级

- **P0**：本地最小模式、Mock 开关、统一错误码、链上链下联查主键
- **P1**：测试钱包与 faucet 指引、标准测试数据、联调模式说明
- **P2**：开发者门户体验优化、公开开发者社区、插件市场

### 39.9 最小可执行落地方案

为避免这些要求停留在原则层，V1 研发阶段应按以下 6 条最小方案实施：

1. **双层权威模型 + 异步事件驱动**
   - 平台内部业务运行态以链下数据库为准。
   - Fabric 负责关键承诺、关键摘要和可信证明确认。
   - 支付、CA、对象存储等外部系统对其事实回执拥有来源权威。
   - 所有上链动作统一走异步事件链路，不允许业务请求直接同步阻塞在链确认上。

2. **三套运行模式**
   - `local`：本地最小模式，仅依赖本地数据库、对象存储、Mock 服务。
   - `staging`：联调模式，接测试版联盟链、公链、签章、支付等外部依赖。
   - `demo`：演示模式，在联调模式基础上加载演示数据与演示公链锚定能力。

3. **统一 Provider 适配层**
   - KYC、签章、支付、链写入、通知等外部能力都走统一 Provider。
   - 每个 Provider 至少提供 `mock` 与 `real` 两套实现。
   - 运行环境只通过配置切换 Provider，不允许在业务代码中写散落的测试分支。

4. **统一调试主键与错误码**
   - 所有关键请求必须至少带 `request_id`。
   - 关键业务对象必须有 `order_id`、`contract_id`、`bill_event_id`、`audit_event_id`。
   - 涉链流程必须能关联 `onchain_tx_hash` 或 `anchor_batch_id`。
   - 错误码必须按统一命名空间管理，不允许前后端各自发明一套错误定义。

5. **本地链优先 + 测试钱包池**
   - 本地开发优先使用本地 validator / 本地链环境。
   - 公共测试网只用于联调、回执验证、对外演示。
   - 平台维护少量测试钱包和测试代币领取记录，用于日常开发调试缓冲。

6. **最小开发者通道**
   - V1 至少提供 4 个调试入口：
     - 开发者首页：网络信息、RPC、浏览器、faucet、SDK 文档入口
     - 测试应用页：API Key、配额、调用日志、回调测试
     - 联查页：按订单号、事件号、交易哈希联查状态
     - 测试资产页：Mock 数据、测试账户、测试钱包、测试代币说明

### 39.10 链上链下状态一致性机制

这是本项目开发阶段最容易出问题的部分，必须按“双层权威模型”明确机制。

V1 要求：

- 数据库事务内同时写业务对象、审计事件与 `outbox_event`。
- `outbox_event` 由 publisher worker 异步发布到 Kafka。
- 下游消费者再分别处理链写、通知、搜索、风控、对账等副作用。
- Fabric / 支付 / CA / 存储等确认结果回写数据库。
- 失败后按退避策略重试，超过阈值进入 `dead_letter_event` 与 Kafka `DLQ topic`。

必须支持的字段建议：

- event_id
- event_type
- business_object_type
- business_object_id
- request_id
- trace_id
- idempotency_key
- authority_scope
- proof_commit_policy
- target_bus
- target_topic
- partition_key
- payload_hash
- submit_status
- retry_count
- max_retries
- last_error_code
- last_error_message
- tx_hash
- confirmed_at
- reconcile_status

必须支持的机制：

- 幂等提交
- 可重试
- 不重复扣费
- 不重复交付
- 不重复存证
- 消费端幂等去重
- dead letter 可重处理
- 回放与对账可修复差异

### 39.11 运行模式与配置切换要求

#### 39.11.1 local 模式

用途：

- 页面联调
- 订单流调试
- 账单规则验证
- 审计日志验证

要求：

- 不依赖真实联盟链、公链、签章、支付、KYC。
- 必须能本地跑通至少：
  - 文件链路
  - API 链路
  - 账单生成
  - 基础审计留痕

#### 39.11.2 staging 模式

用途：

- 联调测试
- 回执验证
- 测试代币验证
- 公链/联盟链锚定验证

要求：

- 可切换到真实测试链和测试外部依赖。
- 日志等级、错误码、回执链路与生产保持一致。

#### 39.11.3 demo 模式

用途：

- 对外演示
- 客户交流
- 场景化验证

要求：

- 加载演示数据
- 提供演示账号
- 可选启用演示级公链锚定
- 不与真实生产数据混用

#### 39.11.4 配置切换总原则

- 运行模式必须通过环境变量或配置文件切换。
- 禁止通过改代码切换环境。
- 所有外部依赖必须声明当前运行模式对应的 Provider 实现。

### 39.12 Provider 适配层要求

V1 至少抽象以下 Provider：

- `KycProvider`
- `SignProvider`
- `PaymentProvider`
- `ChainProvider`
- `NotificationProvider`

每个 Provider 至少支持：

- `mock` 实现
- `real` 实现

要求：

- 业务层仅依赖抽象接口。
- Provider 返回统一结果结构，包括：
  - result_code
  - result_message
  - provider_request_id
  - provider_status
  - raw_ref（如适用）

### 39.13 调试日志与错误码实施要求

#### 39.13.1 调试主键

所有关键日志和页面联查至少支持以下主键：

- request_id
- order_id
- contract_id
- sku_id
- bill_event_id
- audit_event_id
- outbox_event_id
- onchain_tx_hash

#### 39.13.2 日志分类

V1 必须统一输出以下日志：

- HTTP/API 请求日志
- 业务状态流转日志
- 外部 Provider 调用日志
- 链写入与链确认日志
- 审计与风控日志

#### 39.13.3 错误码命名建议

建议按如下命名空间统一：

- `AUTH_*`
- `PERM_*`
- `COMPLIANCE_*`
- `DELIVERY_*`
- `CHAIN_*`
- `BILLING_*`
- `SANDBOX_*`

示例：

- `CHAIN_TX_SUBMIT_FAILED`
- `CHAIN_TX_CONFIRM_TIMEOUT`
- `DELIVERY_TOKEN_INVALID`
- `BILLING_EVENT_MISSING`
- `SANDBOX_SESSION_NOT_READY`

### 39.14 测试代币与测试钱包机制

测试代币问题当前依然是链上开发中的真实痛点，因此 V1 至少应提供以下支持：

- 官方 faucet 链接聚合
- 可信第三方 faucet 备用入口
- 领取限制与冷却时间说明
- 最近一次领取记录
- 常见 faucet 失败原因说明

平台内部测试资产建议包括：

- 平台测试钱包
- 演示钱包
- 联调钱包
- 最近 faucet 状态说明
- 钱包余额检查入口

V1 原则：

- 不自建公开无限量 faucet。
- 优先依赖本地链和少量测试钱包池。
- 测试网代币只用于联调、回执验证和演示验证。

### 39.15 最小开发者通道页面清单

V1 若实现开发者通道，页面最小集合建议如下：

1. **开发者首页**
   - 展示网络信息、RPC、浏览器、faucet、SDK、OpenAPI 链接。

2. **测试应用页**
   - 创建测试应用、查看 API Key、查看配额和最近调用日志。

3. **状态联查页**
   - 输入 `order_id / event_id / tx_hash` 联查订单、账单、审计、链状态。

4. **测试资产页**
   - 展示 Mock 数据、测试账号、测试钱包、测试代币说明。

### 39.16 优先级结论

本章优先级以 39.7、39.8 和 39.9 为准：先保证本地最小模式、Provider 切换、统一调试主键与错误码、链上链下一致性链路，再补测试资产与开发者通道体验。

## 40. 支付、资金流与轻结算设计

### 40.1 设计目标

- 明确真实支付如何接入平台。
- 明确货款、保证金、退款、赔付、打款、分润和税票金额如何流转。
- 明确平台如何收费、怎么收费、收费规则如何版本化调整。
- 明确开发测试阶段如何在不支付真实货币的前提下完成联调、回调、退款、打款和对账演练。

### 40.2 子系统分层

平台必须拆分以下能力层：

1. 定价与收费规则层
2. 支付编排层
3. 内部账务与台账层
4. 外部支付渠道适配层
5. 对账与清结算层
6. Mock PaymentProvider 调试层

### 40.3 V1 支持的支付方式

- 起步司法辖区：新加坡
- `V1` 首发商业闭环按“先做新加坡落地，再保留中国规则兼容能力”执行。
- 中国数据要素、公共数据、B2B / B2G / G2B 等规则语境继续作为字段设计、权限边界、审计口径和后续扩展兼容输入，但不构成 `V1` 首发商用支付、税票和收款流程的优先基线。
- 商品建议统一以 `USD` 计价；买方付款币种、卖方收款币种和真实支付通道受起步走廊策略约束。
- `V1` 的真实生产路由以新加坡主体和新加坡合作伙伴体系为起点，其他地区须等对应走廊显式开启后进入生产。

- 支付宝
- 微信支付
- 银联
- PayPal
- 线下对公转账
- 人工确认收款
- Mock 测试支付

### 40.4 资金类型

平台至少必须结构化管理以下资金对象：

- 订单货款
- 买方保证金
- 卖方履约保证金
- 风险托管金额
- 平台服务费
- 渠道手续费
- 退款金额
- 赔付金额
- 分润金额
- 发票相关金额
- 加急/专线/额外存储/定制报告等外围服务费

### 40.5 收费规则

收费规则必须支持以下对象：

- 平台服务费
- 渠道手续费
- 保证金规则
- 增值服务费
- 分润规则

收费公式必须支持：

- 固定金额
- 比例抽成
- 阶梯比例
- 基础费 + 超额费
- 按周期收费
- 按任务/按次收费
- 按风险等级动态调整保证金

### 40.6 订单与支付的闭环

订单资金流必须形成以下闭环：

```text
下单
  -> 价格快照与费用预估
  -> 司法辖区与走廊校验
  -> 支付意图
  -> 真实/Mock 支付
  -> 回调验签与幂等更新
  -> 账务镜像
  -> 交付与验收
  -> 结算计算
  -> 退款/赔付/打款/分润
  -> 对账与差异处理
  -> 审计归档
```

### 40.7 Mock PaymentProvider

开发测试必须内置支付模拟接口，至少支持：

- 支付成功
- 支付失败
- 支付超时
- 重复 webhook
- 部分退款
- 全额退款
- 打款成功
- 打款失败
- 对账差异

### 40.8 V1 / V2 / V3 边界

- `V1`：新加坡起步司法辖区、真实支付渠道适配抽象、支付意图、基础退款、人工打款、基础对账、Mock PaymentProvider。
- `V2`：在新加坡起步框架上扩展跨境法币走廊、自动打款、自动分润、周期扣费、子商户/分账扩展。
- `V3`：多币种、跨境、交易所/数字货币结算与跨平台结算路由；仅在被允许的司法辖区和走廊上启用。

## 41. 身份认证、注册登录与会话管理设计

### 41.1 设计原则

- 平台登录身份、组织成员身份、服务身份与 Fabric 链上身份必须严格分层
- Fabric 证书不直接替代最终用户登录
- 不是所有用户都必须签发 Fabric 身份
- 浏览器门户与开放 API 必须采用不同会话策略
- 高风险动作必须触发 step-up 认证

### 41.2 注册与登录

平台至少支持：

- 企业主体注册
- 组织成员邀请注册
- 企业 OIDC SSO 首次建档
- 开发者/测试主体注册
- 本地登录
- 企业 SSO 登录
- MFA

生产交易主体默认不依赖社交登录作为正式交易身份。

### 41.3 会话与设备治理

平台必须支持：

- 服务端浏览器会话
- API 访问令牌与刷新令牌
- 设备登记与设备撤销
- 多设备在线与踢出其他设备
- idle timeout、absolute timeout
- refresh token 轮换、复用检测、整族吊销
- 新设备、新国家/地区、新 IP 段登录告警

### 41.4 高风险再认证

以下动作必须绑定 MFA 或等价强认证：

- 合同确认
- 退款、赔付、人工打款
- 保证金扣罚
- 角色提升、SSO 连接修改
- Fabric 身份签发、续签、吊销
- 高敏导出与训练放行

### 41.5 Fabric 身份与证书治理

平台必须支持：

- Fabric CA registry 管理
- Fabric 身份申请、签发、续签、吊销
- MSP、affiliation、attrs 绑定
- 证书摘要、序列号、有效期、审批记录留痕
- 链上身份与平台主体/成员/服务身份绑定

证书中只允许放最小必要属性或不透明引用，不得直接写入身份证号、护照号、营业执照号等原文。

### 41.6 V1 / V2 / V3 边界

- `V1`：本地账号、邀请注册、企业 OIDC SSO、MFA、会话/设备治理、Fabric 身份绑定与证书治理。
- `V2`：`SAML 2.0`、`SCIM`、Passkey-first、服务身份与执行环境身份增强。
- `V3`：跨平台身份联邦、自适应认证、跨域合作伙伴身份互认。

### 41.7 与整套逻辑的闭环

```text
主体注册/KYC-KYB
  -> 成员邀请/SSO 建档
  -> MFA 与设备绑定
  -> 登录建立会话
  -> 角色与作用域授权
  -> 交易/交付/支付/审计操作
  -> 高风险动作 step-up
  -> 必要时调用 Fabric 身份完成链上签名
  -> 会话、证书、令牌、操作证据全量留痕
```

新增这一部分后，平台在主体准入、登录、链上认证、支付高风险动作、审计与风控之间已经形成正式闭环。

## 42. 审计、证据链与回放设计

### 42.1 设计目标

平台必须达到：

- 强留痕
- 强可回放
- 强对账
- 强可核验
- 强可追责

审计不再只是日志展示能力，而是独立业务域。

### 42.2 五层审计体系

- `L1` 业务审计事件
- `L2` 证据对象与证据清单
- `L3` 完整性哈希链与批次根
- `L4` Fabric 摘要存证
- `L5` 联查、回放、对账与监管导出

### 42.3 统一审计事件要求

所有关键业务动作至少应记录：

- actor / actor_org / session / device / application
- request_id / trace_id
- action / result / error_code
- before_state_digest / after_state_digest
- tx_hash / evidence_manifest_id
- previous_event_hash / event_hash
- retention_class / legal_hold_status
- sensitivity_level

失败事件、拒绝事件、重复回调、补偿动作不得漏记。

### 42.4 证据对象与保全

平台必须显式管理：

- `EvidenceItem`
- `EvidenceManifest`
- `EvidencePackage`
- `AnchorBatch`
- `ReplayJob`
- `ReplayResult`
- `LegalHold`
- `RetentionPolicy`
- `AuditAccessRecord`

证据原件默认放对象存储或可信存储，数据库存元数据、哈希、保留策略和审计引用。

### 42.5 Fabric 上链范围

必须上链的摘要：

- 合同和授权快照哈希
- 订单关键状态摘要
- 支付/保证金/退款/赔付/结算批次摘要
- 争议立案与裁决摘要
- 证据清单摘要
- 审计批次根
- 监管导出包摘要
- 证书签发/吊销摘要

严禁上链：

- 原始交易数据正文
- 原始支付凭证
- 会话令牌、密码、密钥
- 证据原文
- 模型权重全文和原始训练语料

### 42.6 回放能力

平台必须支持：

- 取证回放
- 状态回放
- 对账回放
- 白名单补偿重放

默认只允许 `dry-run`；有副作用的 replay 必须二次认证并进入高风险审计。

### 42.7 审计访问权限

至少应区分：

- 审计摘要查看
- 原始证据查看
- 脱敏视图查看
- 原文视图查看
- 证据包导出
- 回放执行
- 保留策略管理
- `legal hold` 管理
- 上链批次查看/管理
- `break-glass` 查看

审计查看权限与业务查看权限必须分离；查看审计的人本身也必须被审计。

### 42.8 V1 / V2 / V3 边界

- `V1`：统一审计事件、证据清单、证据包、Fabric 批量摘要、回放 dry-run、支付/交付/争议/证书强审计
- `V2`：受控计算、训练、模型、证明、分润、公链增强纳入强审计
- `V3`：跨链、图风控、监管穿透、生态互联纳入统一审计与回放

### 42.9 与整套逻辑的闭环

```text
业务动作
  -> AuditEvent
  -> EvidenceItem / EvidenceManifest
  -> WORM / retention / legal hold
  -> AnchorBatch
  -> Fabric 摘要存证
  -> 联查 / 导出 / 回放 / 对账
  -> 监管查询与争议处理
  -> 结果再次写回审计链
```

新增这一部分后，平台在交易、支付、身份、证书、争议、监管和跨链等关键域上，形成了统一的强审计基础。

## 43. 双层权威模型与链上链下一致性设计

### 43.1 总原则

平台统一采用：

- **业务运营权威：链下数据库**
- **可信证明确认层：Fabric**
- **外部事实来源权威：支付渠道 / Fabric CA / 对象存储等**
- **Kafka：事件分发与解耦总线，不是业务权威源**

### 43.2 三类状态

关键对象必须区分：

- `业务状态`：平台当前运行态
- `证明提交状态`：摘要/承诺是否已进入 Fabric
- `外部事实状态`：支付、证书、存储等外部回执是否成立

### 43.3 统一字段建议

关键业务表建议统一具备或映射以下字段：

- `status` / `payment_status` / `settlement_status`：业务状态
- `proof_commit_state`
- `proof_commit_policy`
- `external_fact_status`
- `reconcile_status`
- `authority_model`

### 43.4 事件链路

```text
数据库事务
  -> 业务对象
  -> AuditEvent
  -> outbox_event
  -> publisher worker
  -> Kafka
  -> 下游消费者
  -> Fabric / 支付 / CA / 存储确认
  -> 回写 proof_commit_state / external_fact_status / reconcile_status
  -> dead letter / replay / reconciliation
```

### 43.5 对象分类

#### 纯链下业务权威

- 注册、登录、会话、设备
- 商品草稿、询价、报价
- 下载令牌、API Key、沙箱席位
- 审计原始日志

#### 双层权威对象

- 合同摘要
- 订单关键状态摘要
- 交付回执摘要
- 验收结果摘要
- 结算批次摘要
- 争议裁决摘要
- 审计批次与证据清单摘要

#### 外部事实来源对象

- 支付成功/失败
- 退款成功/失败
- 打款成功/失败
- 证书签发/吊销
- 对象删除/销毁完成

### 43.6 Kafka 与 Dead Letter

- `V1` 正式引入 Kafka 作为事件总线。
- `outbox_event` 继续作为数据库事务边界内的可靠出站事件表。
- `dead_letter_event` 作为平台运营和人工修复主记录。
- Kafka `DLQ topic` 作为消费者失败隔离层。
- `LISTEN / NOTIFY` 仅可作为 worker 唤醒提示，不替代可靠队列。
- `CDC / Debezium` 作为 `V2+` 优化项，不替代事务性 outbox 语义。

### 43.7 闭环结论

新增本设计后，平台的一致性逻辑统一为：

- 平台内部运行态看数据库
- 对外可信证明看 Fabric 确认
- 外部事实结果看来源系统回执
- Kafka 负责分发，不负责定义真相

这与支付、审计、身份、交付、跨链等现有模块可以形成闭环，不再出现“链上和链下都像主状态机”的冲突。

---

## 44. 商品搜索、排序与索引同步设计

### 44.1 搜索架构边界

平台搜索正式采用：

- PostgreSQL：主数据权威源 + 搜索投影表
- OpenSearch：商品搜索与卖方搜索读模型
- Redis：搜索结果与 facet 缓存
- Kafka：搜索索引同步事件总线

规则：

- 搜索引擎不是新的业务真相源
- 商品详情、下单、支付、交付前的最终状态一律回 PostgreSQL 校验

### 44.2 搜索对象范围

V1 至少支持：

- 数据商品搜索
- 服务商品搜索
- 卖方主体搜索

V2 增量支持：

- 模型商品搜索
- 混合检索

V3 增量支持：

- 合作伙伴 / 跨平台主体搜索

### 44.3 搜索投影对象

新增或强化以下对象：

- `search.product_search_document`
- `search.seller_search_document`
- `search.search_signal_aggregate`
- `search.ranking_profile`
- `search.index_alias_binding`
- `search.index_sync_task`

### 44.4 统一同步链路

```text
业务写 PostgreSQL 主表
  -> 同事务刷新搜索投影
  -> 同事务写 outbox_event
  -> publisher worker
  -> Kafka
  -> Search Indexer
  -> 从 PostgreSQL 读取最新搜索投影
  -> 写入 OpenSearch
  -> 回写索引同步状态
  -> Redis 失效相关缓存
```

### 44.5 查询链路

```text
搜索请求
  -> Redis 热缓存
  -> OpenSearch 查询与聚合
  -> PostgreSQL 最终状态校验
  -> 返回最终结果
```

### 44.6 排序原则

搜索必须区分：

- 召回层
- 排序层

V1 默认综合排序至少考虑：

- 文本相关性
- 质量评分
- 卖方信誉
- 热度
- 新鲜度
- 风险惩罚

硬过滤必须先于排序：

- 未上架
- 已冻结
- 风险阻断
- 不适用当前司法辖区
- 不满足主体准入限制

### 44.7 一致性与修复

每个搜索投影对象必须具备：

- `document_version`
- `source_updated_at`
- `index_sync_status`
- `indexed_at`
- `last_index_error`

并必须支持：

- 单对象重建
- 批量重建
- dead letter 重处理
- PostgreSQL 与 OpenSearch 版本对账
- Redis 缓存失效回放

### 44.8 闭环结论

新增本设计后，平台搜索形成完整闭环：

- PostgreSQL 负责真相源
- OpenSearch 负责检索与聚合
- Redis 负责缓存
- Kafka / outbox 负责同步
- PostgreSQL 最终校验负责避免脏结果进入交易链路

---

## 45. 商品推荐与个性化发现设计

### 45.1 模块定位

推荐模块是平台的“发现与转化”能力，不替代搜索，不替代交易主状态机，也不替代合规/权限/风控放行。

### 45.2 推荐对象

`V1` 至少支持：

- 数据商品
- 服务商品
- 卖方主体

`V2` 增量支持：

- 模型商品
- 训练/推理相关服务

`V3` 增量支持：

- 伙伴 / 连接器
- 生态合作主体

### 45.3 推荐架构边界

推荐正式采用：

- `PostgreSQL`：行为事件、画像快照、推荐请求、推荐结果、配置与审计主存
- `OpenSearch`：推荐候选召回与相似检索读模型
- `Redis`：热门榜单缓存、推荐结果短缓存、已看集合缓存
- `Kafka`：行为流、商品变更流与推荐结果回流总线
- `recommendation-service`：推荐编排与排序服务

统一结论：

**`PostgreSQL 主数据权威 + OpenSearch 候选召回 + Redis 缓存 + Kafka 事件同步 + recommendation-service 编排排序 + PostgreSQL 最终业务校验`**

### 45.4 推荐位

`V1` 必做推荐位：

- 首页精选
- 行业专题推荐
- 商品详情页相似推荐
- 商品详情页配套服务推荐
- 卖方主页热门推荐
- 买方工作台推荐
- 搜索零结果兜底推荐

### 45.5 候选召回

`V1` 推荐采用多路候选召回：

- 热门召回
- 内容相似召回
- 聚合协同行为召回
- 新品探索召回
- 配套/捆绑召回
- 卖方主体召回

Fabric 事件只作为可信业务结果补充流，不替代应用行为流。

### 45.6 行为事件

推荐域至少记录：

- `recommendation_panel_viewed`
- `recommendation_item_exposed`
- `recommendation_item_clicked`
- `product_detail_viewed`
- `service_detail_viewed`
- `seller_profile_viewed`
- `sample_preview_viewed`
- `quote_requested`
- `poc_requested`
- `order_submitted`
- `payment_succeeded`
- `delivery_accepted`
- `not_interested`
- `refund_or_dispute_after_order`

### 45.7 排序原则

`V1` 采用规则型、可解释综合排序，至少考虑：

- 意图/上下文匹配
- 内容相似度
- 热度
- 数据质量
- 卖方信誉
- 新鲜度
- 转化信号
- 配套关系
- 风险惩罚
- 重复惩罚

推荐结果必须支持解释码，例如：

- `popular_overall`
- `similar_to_current_item`
- `same_seller_more_items`
- `new_and_qualified`
- `trusted_seller_boost`
- `service_bundle_match`

### 45.8 开源能力使用原则

推荐域允许使用开源项目简化开发，但必须服从平台主状态和放行边界：

- `V1`：自建 `recommendation-service` 为主；可选接入 `Gorse` 作为候选召回增强器
- `V2`：可引入 `LibRecommender` 或等价能力做模型训练与排序增强
- `RecBole` 更适合离线实验，不建议首版直接作为线上依赖
- 推荐引擎不得直接替代 PostgreSQL 最终业务校验

### 45.9 与搜索和审计的关系

- 推荐与搜索共享 `search.*` 投影，但排序与候选逻辑独立
- 推荐请求、返回结果、解释码、点击/转化回流必须进入审计
- 不建议把全量行为明细逐条上链
- 可将推荐策略版本和每日推荐审计摘要做摘要上链

### 45.10 闭环结论

新增本设计后，平台推荐形成完整闭环：

```text
商品/服务/卖方主数据
  -> search.* 投影
  -> Kafka 同步
  -> OpenSearch 候选召回
  -> recommendation-service 多路召回与重排
  -> PostgreSQL 最终业务校验
  -> recommendation_result
  -> 曝光/点击/转化行为回流
  -> 画像/热度/相似度更新
  -> 审计与可回放
```

## 46. 日志、可观测性与告警设计

### 46.1 目标

平台需要建立统一的运行态观测能力，用于支撑：

- 应用、支付、Fabric、搜索、推荐、跨链、受控计算等链路的故障定位
- 强留痕平台的运行联查
- 告警驱动的人工处置与接管
- 与审计、证据链、回放、一致性修复的联动

### 46.2 域边界

- 审计域：`audit.*`
  - 负责证据、回放、导出、上链摘要
- 观测域：`ops.* + Loki/Tempo/Prometheus`
  - 负责运行日志、trace、指标、告警、工单

### 46.3 V1 技术栈

- `OpenTelemetry`：统一采集 `logs / metrics / traces`
- `Prometheus`：指标采集
- `Alertmanager`：规则告警分发
- `Grafana`：统一看板
- `Loki`：V1 日志后端
- `Tempo`：V1 trace 后端

限制：

- 不将 `ELK / Elasticsearch / OpenSearch` 作为 V1 日志主后端硬依赖
- OpenSearch 继续只承担商品/服务/卖方/模型/伙伴搜索读模型

### 46.4 PostgreSQL 落库范围

PostgreSQL 中只承接：

- 观测后端配置
- 日志保留策略
- 关键结构化系统日志镜像
- trace 索引
- 告警规则
- 告警事件
- 事件工单
- SLO 定义与快照

### 46.5 V1 页面

V1 至少补齐：

- 可观测性总览页
- 日志检索页
- Trace 联查页
- 告警中心
- 事件工单页
- SLO 管理页

### 46.6 权限与治理

新增权限族：

- `ops.observability.read/manage`
- `ops.log.query/export`
- `ops.trace.read`
- `ops.alert.read/ack/manage`
- `ops.incident.read/manage`
- `ops.slo.read/manage`

规则：

- 原始日志导出必须 step-up
- 告警规则修改必须强审计
- 工单强制关闭必须留痕

### 46.7 闭环结论

新增本设计后，平台运行态形成闭环：

```text
业务服务 / 网关 / 任务 / 节点
  -> OpenTelemetry
  -> Loki / Tempo / Prometheus
  -> 告警 / 工单 / 关键日志镜像入 PostgreSQL
  -> request_id / trace_id 串联业务主键
  -> 与 audit.* / outbox / dead letter / consistency trace 联查
```

## 47. 交易链监控、公平性与信任安全设计

### 47.1 设计目标

- 平台必须把“完整交易链监控”定义为正式产品能力，而不是仅依赖基础日志或区块链上链结果。
- 交易链监控必须覆盖交易前、交易中、交易后以及敏感数据受控交付的完整生命周期。
- 平台必须能够回答四类问题：
  - 当前交易走到哪一步
  - 哪一步缺失或超时
  - 哪个外部事实已经被确认
  - 是否已经触发不公平、不安全或不可结算风险

### 47.2 六层监控模型

- `L1` 业务状态监控：订单、合同、交付、验收、结算、争议、撤权。
- `L2` 交付与执行监控：下载、API 调用、模板查询、沙箱、受控执行、结果导出。
- `L3` 证据与审计监控：审计事件、证据清单、锚定批次、回放任务、保留与销毁证明。
- `L4` 公平性与风控监控：迟交付、恶意拒收、重复交付、结果绕过、回执缺失、外部事实异常。
- `L5` 运维可观测性监控：日志、trace、告警、事件工单、SLO。
- `L6` Fabric 可信确认层：提交确认、链码事件、锚定状态、投影缺口。

### 47.3 全链路统一标识

- 所有交易链路必须统一使用并联查以下标识：
  - `request_id`
  - `trace_id`
  - `order_id`
  - `tx_id / tx_hash`
  - `event_id`
- 未带统一标识的关键链路不允许进入正式监控、回放和争议裁决主链。

### 47.4 关键监控对象

- `TradeLifecycleCheckpoint`：生命周期检查点，表达交易在每个阶段“应发生/已发生/是否超时/是否缺失”。
- `ExternalFactReceipt`：外部事实回执，统一表达支付回调、打款回执、销毁证明、执行证明、合作方 ack 等。
- `FairnessIncident`：公平性事件，统一表达迟交付、恶意拒收、重复消费、越权导出、结果绕过等事件。
- `MonitoringPolicyProfile`：监控策略档案，定义检查点规则、告警规则、自动动作和升级阈值。
- `ChainProjectionGap`：链上链下投影缺口，定义预期链事件与实际投影状态之间的差异。

### 47.5 公平性与信任规则

- 关键状态不允许单边“说完成就完成”，必须由业务状态、外部事实和可信确认共同收敛。
- 文件型商品、共享/API 商品、模板查询/沙箱商品、敏感结果商品必须使用不同的公平规则与验收规则。
- 平台不得把 Kafka 消费成功、链写提交成功或外部回调任一单点视为全局唯一权威。
- 对敏感数据，平台监控重点不是“文件是否被下载”，而是“是否按批准边界被受控使用、是否发生越权输出、是否完成撤权与销毁/保留证明”。

### 47.6 自动化动作

- 命中严重监控规则时，平台必须支持：
  - 自动阻断交付或导出
  - 自动冻结结算或赔付
  - 自动发起争议或风险工单
  - 自动要求补充外部事实回执或销毁/保留证明
  - 自动写入审计链并形成链上摘要锚定

### 47.7 分阶段要求

- `V1`：必须交付生命周期检查点、外部事实回执、公平性基础事件、Fabric 提交/事件确认、敏感链路最小闭环监控。
- `V2`：必须把训练、受控计算、模型服务、分润和自动打款纳入交易链监控。
- `V3`：必须把跨链、跨平台回执、合作伙伴 ack、监管穿透和生态协同纳入统一监控总线。

---

## 48. V1 权利与 SKU 最终对照表

| 权利类型 | V1 是否正式支持 | 唯一解释 | 适用商品交付形态 | 默认限制 | 必须绑定的控制点 |
|---|---|---|---|---|---|
| 使用权 | 是 | 在约定业务场景中内部使用文件、结果或受限交付物 | 文件快照/版本订阅、结果产品 | 不得转售、不得公开传播、默认不得训练公开模型 | 合同用途、主体范围、有效期、地域、导出限制 |
| 访问权 | 是 | 通过 API、只读共享入口、平台网关访问受控数据能力 | 只读共享、API/服务 | 不得共享凭证、不得绕过网关/配额、不得沉淀越权副本 | App 绑定、IP/环境限制、配额、到期断权 |
| 查询权 | 是 | 在受控环境内运行平台批准的模板、沙箱查询或限定分析 | 只读共享、模板查询 lite、查询沙箱 | 不得自由 SQL 越权、不得直接导出原始明细 | 模板白名单、参数校验、输出校验、导出规则 |
| 结果获取权 | 是 | 获取符合输出规则的报告、聚合结果、受限结果集 | 只读共享、API/服务、模板查询 lite、查询沙箱、结果产品 | 不得反向恢复原始样本、不得超出合同口径对外使用 | 输出模板、脱敏规则、结果回执、审计记录 |
| 有限内部共享权 | 是 | 仅允许在买方同一 Tenant 内，向被点名的 Department / User / Application 共享 | 文件快照/版本订阅、结果产品、部分查询/API 商品 | 不得跨 Tenant、不得默认扩散给全员、不得对外转授权 | Tenant 绑定、部门/用户/App 白名单、审批留痕 |
| 计算权 | 否（V2） | 提交算法或任务到数据侧执行 | C2D / 受控计算 | V1 不可正式售卖 | 任务审批、算法白名单、执行环境证明 |
| 受控模型训练权 | 否（V2） | 在受控条件下用于模型训练或微调 | V2 计算/模型产品 | V1 默认禁止 | 训练目的声明、样本边界、输出边界 |
| 结果级商业输出权 | 否（V2） | 将结果用于对外商业服务 | V2 模型/结果产品 | V1 默认禁止 | 法务审批、合同附加条款 |
| 转授权权 | 否 | 再授权第三方使用 | 无 | V1 默认禁止 | 仅未来特批 |
| 所有权转让 | 否 | 资产权属转移 | 无 | V1 默认禁止 | 不进入标准平台商品 |

### 48.1 V1 权利配套规则

1. `rights_type` 的 V1 可枚举值统一为：`use / access / query / result_get / internal_share_limited`。
2. `internal_share_limited` 不能单独售卖，必须附着于某一主权利商品。
3. `subscription` 统一不进入 rights_type，而进入 `pricing_mode / delivery_mode / revision_subscription`。
4. V1 任意商品只要出现“算法提交、任务执行、算力上限、容器审核、证明材料齐全”等描述，必须标记为 V2 预留，不得默认落入 V1 沙箱商品。

### 48.2 商品交付形态到标准 SKU 的唯一映射

V1 目录层允许使用 6 种商品交付形态进行展示与理解，但工程实现必须进一步收口到 8 个标准 SKU。其中：

- 文件类交付形态拆分为 `FILE_STD / FILE_SUB`
- API / 服务类交付形态拆分为 `API_SUB / API_PPU`
- 其余 4 种交付形态各自对应 1 个标准 SKU

| V1 商品交付形态 | 标准 SKU | 是否 V1 正式支持 | 默认可售权利 | 默认交付方式 | 默认验收模板 | 默认退款模板 | 默认计费模板 | 备注 |
|---|---|---|---|---|---|---|---|---|
| 文件快照 | FILE_STD | 是 | 使用权 + 有限内部共享权（可选） | 文件令牌下载 / 限次下载 | ACCEPT_FILE_V1 | REFUND_FILE_V1 | BILL_FILE_ONCE_V1 | 对应一次性交付 |
| 版本订阅 | FILE_SUB | 是 | 使用权 + 有限内部共享权（可选） | 周期版本推送 / 周期交付 | ACCEPT_FILE_SUB_V1 | REFUND_FILE_SUB_V1 | BILL_FILE_SUB_V1 | 对应 revision/周期履约 |
| 只读共享 | SHARE_RO | 是 | 访问权 + 查询权 + 结果获取权 | Share grant / linked dataset / datashare | ACCEPT_SHARE_RO_V1 | REFUND_SHARE_RO_V1 | BILL_SHARE_RO_V1 | 对应独立共享模板族 |
| API 订阅 | API_SUB | 是 | 访问权 + 结果获取权 | API Key / OAuth / 网关开通 | ACCEPT_API_SUB_V1 | REFUND_API_SUB_V1 | BILL_API_SUB_V1 | 周期费 + 调用量 |
| API 按量 | API_PPU | 是 | 访问权 + 结果获取权 | API 调用 | ACCEPT_API_PPU_V1 | REFUND_API_PPU_V1 | BILL_API_PPU_V1 | 仅成功调用计费 |
| 模板查询 lite | QRY_LITE | 是 | 查询权 + 结果获取权 | 白名单模板查询 / 受限结果交付 | ACCEPT_QUERY_LITE_V1 | REFUND_QUERY_LITE_V1 | BILL_QUERY_LITE_V1 | 对应独立模板查询模板族 |
| 查询沙箱 | SBX_STD | 是 | 查询权 + 结果获取权 | 沙箱席位 / 项目空间 / 受限导出 | ACCEPT_SANDBOX_V1 | REFUND_SANDBOX_V1 | BILL_SANDBOX_V1 | 不得写成 V1 计算权 |
| 固定报告/结果产品 | RPT_STD | 是 | 结果获取权 + 使用权 + 有限内部共享权（可选） | 报告交付 / 结果包下载 | ACCEPT_REPORT_V1 | REFUND_REPORT_V1 | BILL_REPORT_V1 | 对应固定报告、结果包 |

### 48.3 各商品交付形态的唯一主模板族

| 商品交付形态 | 合同模板 | 授权模板 | 交付模板 | 验收模板 | 退款模板 | 赔付模板 | 计费模板 |
|---|---|---|---|---|---|---|---|
| 文件快照 | CONTRACT_FILE_V1 | LICENSE_FILE_USE_V1 | DELIVERY_FILE_V1 | ACCEPT_FILE_V1 | REFUND_FILE_V1 | COMP_FILE_V1 | BILL_FILE_ONCE_V1 |
| 版本订阅 | CONTRACT_FILE_SUB_V1 | LICENSE_FILE_USE_V1 | DELIVERY_FILE_SUB_V1 | ACCEPT_FILE_SUB_V1 | REFUND_FILE_SUB_V1 | COMP_FILE_SUB_V1 | BILL_FILE_SUB_V1 |
| 只读共享 | CONTRACT_SHARE_RO_V1 | LICENSE_SHARE_RO_V1 | DELIVERY_SHARE_RO_V1 | ACCEPT_SHARE_RO_V1 | REFUND_SHARE_RO_V1 | COMP_SHARE_RO_V1 | BILL_SHARE_RO_V1 |
| API 订阅 | CONTRACT_API_SUB_V1 | LICENSE_API_SUB_V1 | DELIVERY_API_SUB_V1 | ACCEPT_API_SUB_V1 | REFUND_API_SUB_V1 | COMP_API_SUB_V1 | BILL_API_SUB_V1 |
| API 按量 | CONTRACT_API_PPU_V1 | LICENSE_API_PPU_V1 | DELIVERY_API_PPU_V1 | ACCEPT_API_PPU_V1 | REFUND_API_PPU_V1 | COMP_API_PPU_V1 | BILL_API_PPU_V1 |
| 模板查询 lite | CONTRACT_QUERY_LITE_V1 | LICENSE_QUERY_LITE_V1 | DELIVERY_QUERY_LITE_V1 | ACCEPT_QUERY_LITE_V1 | REFUND_QUERY_LITE_V1 | COMP_QUERY_LITE_V1 | BILL_QUERY_LITE_V1 |
| 查询沙箱 | CONTRACT_SANDBOX_V1 | LICENSE_SANDBOX_USE_V1 | DELIVERY_SANDBOX_V1 | ACCEPT_SANDBOX_V1 | REFUND_SANDBOX_V1 | COMP_SANDBOX_V1 | BILL_SANDBOX_V1 |
| 固定报告/结果产品 | CONTRACT_REPORT_V1 | LICENSE_RESULT_USE_V1 | DELIVERY_REPORT_V1 | ACCEPT_REPORT_V1 | REFUND_REPORT_V1 | COMP_REPORT_V1 | BILL_REPORT_V1 |

### 48.4 六种 V1 商品交付形态的唯一业务口径

#### 48.4.1 文件快照 / 版本订阅

- 主权利：使用权
- 可附加：有限内部共享权
- 不得表述为：访问权主售、计算权、默认训练权
- 交付单位：文件包 / 版本包
- 自动验收：下载成功并校验哈希一致，或交付后 3 个工作日无异议
- 退款主口径：未下载可退款；下载后仅按质量争议退款；订阅类按首交付与剩余周期处理

#### 48.4.2 只读共享

- 主权利：访问权 + 查询权 + 结果获取权
- 不得表述为：下载使用包、全库开放、自由导出
- 交付单位：share grant / recipient grant / linked dataset / datashare
- 自动验收：grant 生效、recipient 绑定成功、指定只读对象可访问且无写权限
- 退款主口径：未开通成功可退款；已开通但共享对象不可访问、对象范围明显不符或持续不可用时按部分/全额退款或补偿处理

#### 48.4.3 API / 服务

- 主权利：访问权 + 结果获取权
- 不得表述为：底层原始库表导出权
- 交付单位：应用级访问权
- 自动验收：首次成功调用返回 2xx/业务成功，或开通后 5 个工作日无异议
- 退款主口径：未开通成功可退款；SLA 违约按阶梯补偿；已成功调用部分通常不退款

#### 48.4.4 模板查询 lite / 查询沙箱

- 主权利：查询权 + 结果获取权
- 可附加：有限内部共享权
- 不得表述为：V1 计算权、自由 SQL、任意脚本执行、原始表导出
- 交付单位：模板白名单 / 席位 / 项目空间 / 受限导出
- 自动验收：首次登录且指定查询成功，或开通后 5 个工作日无异议
- 退款主口径：未开通可退款；模板长期不可用、导出规则与合同不符、报告未按约交付时按补开通/补偿/部分退款处理

#### 48.4.5 固定报告 / 结果产品

- 主权利：结果获取权
- 可附加：使用权、有限内部共享权
- 不得表述为：默认附带原始明细数据、默认可公开宣传、默认可训练
- 交付单位：报告、结果包、聚合结果集
- 自动验收：结果包生成并满足输出模板，或交付后约定时限无异议
- 退款主口径：未交付可退款；可修复问题优先补交付；口径明显不符且无法修复时部分或全额退款

---

### 48.6 研发、法务、测试的直接落地要求

#### 48.6.1 研发

- 订单中心、商品中心、合同中心、权限引擎、账单引擎都必须以第 `48.2`、`48.3` 节为唯一枚举来源。
- 数据库中若存在 `rights_type = subscription` 或 `rights_type = compute` 的 V1 配置，必须清理或迁移。
- `rights_bundle_type` 不得继续作为 V1 权利主枚举字段；如保留历史字段，仅可由 `rights_type + sku_type + delivery_mode + pricing_mode` 派生生成。
- API 类模板命名必须细分到 `API_SUB / API_PPU`，不得在同一版本内混用 `ACCEPT_API_V1 / REFUND_API_V1` 与 `ACCEPT_API_SUB_V1 / REFUND_API_SUB_V1` 两套命名。
- 模板中心必须提供 `SHARE_RO`、`QRY_LITE`、`SBX_STD` 三类独立模板族，不得混配。

#### 48.6.2 法务

- 所有标准合同模板需与五权利口径对齐。
- “有限内部共享权”必须带 Tenant 边界与指定主体边界，不得写成模糊的“集团内部可共享”。

#### 48.6.3 测试

- 必须按 8 个标准 SKU 设计测试用例。
- 必须覆盖：模板错配阻断、权利越权阻断、自动验收触发、退款规则分流、账单事件正确归集。

---

### 48.7 最终结论

V1 的唯一真值应当是：

- **六种 V1 商品交付形态**：文件快照/版本订阅、只读共享、API/服务、模板查询 lite、查询沙箱、固定报告/结果产品
- **五类正式权利**：使用权、访问权、查询权、结果获取权、有限内部共享权
- **八个标准 SKU**：`FILE_STD / FILE_SUB / SHARE_RO / API_SUB / API_PPU / QRY_LITE / SBX_STD / RPT_STD`
- **V2 才引入**：计算权、受控模型训练权、结果级商业输出权

自本版起，PRD 仅使用上述现行口径。

---

## 49. 工程约束冻结要求

本章用于冻结工程实施所需的关键约束。若与前文存在表达冲突，以本章为准。

### 49.1 P0 冲突项的最终冻结

#### 49.1.1 权利模型唯一源

- `V1` 唯一正式权利枚举：`use / access / query / result_get / internal_share_limited`
- `subscription` 统一进入 `delivery_mode / pricing_mode / revision_subscription`
- `compute` 统一视为 `V2` 权利，不得在 `V1` 表单、索引、合同、权限点、测试用例中出现为正式售卖值
- `rights_type` 的唯一枚举源为第 `35` 章字段字典与第 `48` 章唯一真值表
- `sku_type / delivery_mode / pricing_mode` 为并列业务轴，但不得反向生成新的权利枚举；权利判断一律回到 `rights_type`
- `product_type` 仅作为目录与展示分类轴使用，不得参与正式权利判定或替代标准 SKU 判定

#### 49.1.1A Product 与 SKU 事实源冻结

- `Data Product` 是目录、展示、审核与聚合视图事实源。
- `Product SKU` 是下单、合同、授权、交付、验收、账单与结算事实源。
- `product.delivery_mode` 仅作为目录默认值或聚合展示值；进入交易时一律以 `sku.delivery_mode` 快照为准。
- `product.sku_template_id` 仅作为 SKU 模板族默认值或批量生成入口；`sku.contract_template_id / default_usage_policy_id / acceptance_template_id / refund_template_id` 才是最终模板绑定事实源。
- `usage_mode` 仅表达推荐使用姿态或默认使用方式，不得替代 `allowed_rights`。
- `pricing_mode` 是正式对外收费模式字段；账单归集、履约计量和结算计算统一由 `Billing Event` 与计费模板驱动，不再并列引入正式 `billing_mode` 主字段。

#### 49.1.2 架构图与模块边界

- 所有总览图、时序图、模块图都必须标记 `[V1 Active] / [V2 Reserved] / [V3 Reserved]`
- 图中出现预留模块，不代表该模块进入当前版本开发排期
- `C2D / FL / MPC / TEE / ZKP / 完整 clean room / 复杂共享连接器框架` 在 `V1` 一律只允许保留对象模型、接口占位、审计与权限边界，不进入正式交付承诺

#### 49.1.3 统一状态字段落到实体

以下字段从本版起视为关键业务对象的统一一致性状态族，`Order / Contract / Authorization / Delivery / Settlement / Dispute / Billing Event` 必须具备或通过视图映射具备：

- `authority_model`
- `proof_commit_policy`
- `proof_commit_state`
- `external_fact_status`
- `reconcile_status`
- `idempotency_key`
- `request_id`
- `trace_id`
- `status_version`

落地要求如下：

| 对象 | 落地方式 | 最低要求 |
|---|---|---|
| `Order` | 实体同名字段 | 作为流程主对象，必须完整落库并作为状态映射基准 |
| `Digital Contract` | 实体同名字段 | 合同签署、授权放行、链上摘要和回放必须直接引用同名字段 |
| `Authorization` | 实体同名字段 | 授权实例不得只由 `Usage Policy` 规则推导，必须形成可审计授权对象 |
| `Delivery` | 实体同名字段 | 交付回执、开通状态、回放和赔付必须直接关联交付对象状态族 |
| `Settlement` | 实体同名字段 | 结算状态、分账、打款与对账必须直接关联结算对象状态族 |
| `Dispute` | 实体同名字段 | 争议阻断、复议、处理结果和恢复动作必须落在争议对象状态族 |
| `Billing Event` | 实体同名字段 | 账单事件允许按事件源建模，但涉及补记、冲正、赔付、退款时仍必须具备完整状态族 |

任何服务若另起一套同义字段，视为违反建模规范。若采用事件表或视图映射实现，字段名也必须与本表保持一致，不得再发明第二套语义相同但命名不同的状态字段。

#### 49.1.4 字段字典单一事实源

- 第 `35` 章是商品元数据、搜索索引、审核字段、合同映射、风控输入、推荐特征的单一事实源
- OpenSearch / PostgreSQL 检索投影 / 缓存索引 / 审核表单不得维护第二份冲突枚举
- 所有枚举变更必须先改字段字典，再同步下游实现

### 49.2 外部接口与回调统一规范

所有支付、KYC/KYB、签章、链服务、通知、对象存储回执、连接器回执统一遵守以下规则：

- 每次出站请求必须带：`request_id`、`trace_id`、`idempotency_key`
- 每次回调或外部回执必须落库：`provider_code`、`provider_request_id`、`callback_event_id`、`event_version`、`provider_status`、`provider_occurred_at`、`payload_hash`
- 所有回调必须验签、验时间窗、做幂等去重后再进入状态机
- 重复回调命中幂等键时必须返回成功确认，不得重复扣费、重复交付、重复开通、重复上链
- 高风险外部回执幂等记录最少保留 `180` 天；普通回执最少保留 `30` 天
- 平台内部重试采用指数退避，必须设置 `max_retries`，超限进入 `dead_letter_event / DLQ`
- 人工重放只能通过显式管理入口触发，必须记录触发人、原因、审批链和是否 `dry-run`

### 49.3 租户隔离与密钥层级冻结

#### 49.3.1 V1 隔离模型

- PostgreSQL 采用“共享集群 + 领域 schema + 强制 `tenant_id` + 服务层范围校验 + 核心高风险表 `RLS`”的混合隔离模式
- 对象存储采用“按环境分 bucket / 按租户前缀分区 / 按产品与版本归档”的路径规范
- 缓存、消息、索引文档都必须显式携带 `tenant_id`
- 跨租户查询只允许通过监管视图、审计视图或显式授权的运营后台完成

#### 49.3.2 密钥层级

- `V1` 采用“环境主密钥 + 租户逻辑密钥域 + 对象级数据密钥（DEK）”的封装模式
- 明文对象默认不得长期落本地磁盘；托管对象采用信封加密
- `break-glass` 查看原文必须双人审批、时限授权、强审计留痕
- `V2/V3` 再演进到 `BYOK / HSM / 每租户专属 CMK`

### 49.4 连接器与执行环境生命周期冻结

连接器、执行环境、链网关、预言机一律按以下最小生命周期管理：

`draft -> pending_review -> attested -> active -> degraded -> suspended -> rotating -> offboarding -> revoked / retired`

V1 至少必须支持：

- 注册审批
- 首次 attestation / 基线校验
- 心跳与健康检查
- 版本兼容矩阵校验
- 密钥/证书轮换
- 隔离、冻结、恢复
- offboarding 与在途订单保护

当连接器进入 `offboarding / revoked / suspended` 时：

- 立即阻止新授权开通
- 在途订单必须进入显式迁移、人工接管或退款分支
- 相关证据和外部回执必须保留到订单生命周期结束

### 49.5 发布、迁移与回滚策略冻结

- 数据库变更必须采用版本化 `upgrade / downgrade SQL`
- 搜索索引必须采用版本索引 + alias 切换，不允许直接覆盖生产索引结构
- 字段变更顺序固定为：`新增字段 -> 双写 -> 回填 -> 读切换 -> 停旧写 -> 下线旧字段`
- 合同模板、授权模板、验收模板、计费模板、退款模板必须具备版本号和生效区间
- 订阅类商品版本升级不得隐式改变已生效合同，必须走补充协议或下一周期生效
- 任意迁移都必须定义：前置检查、回滚条件、数据修复入口、审计记录

### 49.6 V1 容量模型与压测口径基线

以下指标为 `V1` 开发与联调的最小容量假设，不等于最终商业预测，但必须作为架构、缓存、索引、Topic、对象存储和压测脚本的共同输入：

| 维度 | V1 基线 |
|---|---|
| 门户/API 综合流量 | 峰值 `200 RPS` |
| 下单/状态变更写流量 | 峰值 `30 RPS` |
| 搜索请求 | 峰值 `120 RPS` |
| 外部回调入口 | 峰值 `50 RPS / provider` |
| 审计事件写入 | 峰值 `1,000 events/s` |
| outbox 发布能力 | 峰值 `500 events/s` |
| 交易链监控检查点写入 | 峰值 `200 events/s` |
| 单租户并发沙箱席位 | `20` |
| 对象存储日新增 | `500 GB/day` 以内按 V1 基线设计 |
| 证据包导出 | `50` 个并发任务以内 |

压测至少覆盖：

- 正常流量
- 2 倍峰值瞬时冲击
- 重复回调
- Kafka 堆积
- OpenSearch 重建或延迟
- 对象存储抖动

### 49.7 测试与演练矩阵冻结

V1 自动化和联调必须覆盖以下最小场景：

- 重复支付回调、乱序回调、晚到回调
- 支付成功但链锚定失败
- 链成功但数据库投影失败
- 数据库成功但搜索索引延迟或失败
- 证据包导出失败与重试
- 到期断权延迟、撤权失败与补偿
- 对账差异检测、修复与二次核验
- `legal hold` 命中删除申请
- 回放 `dry-run` 与有副作用回放阻断
- 连接器心跳超时、证书过期、轮换失败
- 共享撤权、API token 吊销、沙箱会话踢出

未进入该矩阵的能力，不得宣称“已完成上线验收”。

### 49.8 固定审批节点矩阵

`V1` 不做复杂审批流引擎，但必须固定以下审批节点，不允许各服务自行定义：

| 动作 | 触发条件 | 审批角色 | 结果要求 |
|---|---|---|---|
| 主体注册增强审查 | 命中高风险行业/地区/黑灰名单 | 合规 + 风控 | 通过/拒绝/补证 |
| 商品上架审批 | 含敏感、高敏、重要数据或特殊交付模式 | 运营 + 合规 | 上架/驳回/补证 |
| 合同生效审批 | 非标准条款、大额订单、特殊用途 | 法务 + 业务负责人 | 放行/驳回 |
| 退款/赔付审批 | 超阈值、争议成立、人工打款 | 财务 + 运营 | 执行/驳回 |
| 高敏导出审批 | 结果导出、原文查看、break-glass | 安全 + 业务负责人 | 限时放行/拒绝 |
| 证书签发/吊销审批 | Fabric 身份开通、续签、吊销 | 平台安全管理员 | 签发/吊销 |
| 连接器激活审批 | 首次接入、重大升级、用途变化 | 平台安全 + 运维 | 激活/冻结 |
| 人工重放审批 | 有副作用补偿或重放 | 运维 + 审计 | 执行/拒绝 |

### 49.9 不阻塞前期开发、但必须进入未来问题清单的事项

以下事项当前不阻塞 `V1` 开发起步，但必须持续跟踪，进入更大规模上线前必须回头冻结：

- 每租户专属 `CMK / BYOK / HSM` 的完整落地
- 多协议共享连接器框架与连接器联邦
- 搜索索引蓝绿重建的平台化自动编排
- 基于真实业务数据的二次容量校准
- 跨地域容灾与常态化混沌演练
- 复杂审批流引擎与可视化工作流编排
