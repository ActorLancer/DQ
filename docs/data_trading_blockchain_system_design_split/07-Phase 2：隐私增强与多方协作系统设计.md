<!-- Source: data_trading_blockchain_system_design_v1.md | Section: 7. Phase 2：隐私增强与多方协作系统设计 -->
[返回总览](./00-README.md)
# 7. Phase 2：隐私增强与多方协作系统设计

## 7.1 阶段目标与新增价值

Phase 2 的核心目标是让高敏感数据也能参与价值创造。交易对象不再局限于数据文件本身，而是扩展为访问权、计算权、训练权和模型贡献权。系统应支持在数据不出本地、原始样本不外泄的前提下完成多方协同训练或联合分析。

**Phase 2 新增能力**

| **能力**       | **说明**                                     | **关键组件**                                     |
|----------------|----------------------------------------------|--------------------------------------------------|
| 联邦训练任务   | 多机构在不交换原始数据的条件下完成联合建模。 | FLTaskService、训练节点 SDK、聚合器/链上协调器。 |
| 访问策略模板   | 按主体、用途、期限、地域、次数定义授权。     | PolicyService、策略执行点 PEP。                  |
| 贡献评估与激励 | 依据模型贡献、在线率、数据质量给予奖励。     | ContributionEngine、RewardContract。             |
| 模型水印与版本 | 防止模型被未授权复用，并支持回溯。           | ModelRegistry、WatermarkService。                |
| 隐私增强计算   | 支持参数加密、差分隐私、可信执行环境等扩展。 | PrivacyEnhancer、KeyService。                    |

## 7.2 架构变化

相比 Phase 1，Phase 2 在逻辑架构上新增四个关键平面：训练控制平面、策略授权平面、贡献计量平面和模型资产平面。训练控制平面负责任务生命周期和轮次编排；策略授权平面负责‘谁可以参与、使用目的是否允许、输出结果能否导出’；贡献计量平面负责把节点贡献映射到奖励和信誉；模型资产平面负责模型版本、模型摘要、模型水印与模型使用记录。

**新增模块**

| **模块**             | **职责**                                       | **与 Phase 1 的关系**                    |
|----------------------|------------------------------------------------|------------------------------------------|
| FLTaskService        | 创建训练任务、管理轮次、节点招募和任务结束。   | 复用 Identity、Catalog、Audit。          |
| ParticipantAgent/SDK | 部署在各参与机构本地，执行本地训练与参数提交。 | 作为新的链下节点类型接入。               |
| AggregationService   | 负责参数聚合、鲁棒聚合、指标收集。             | 结果写入链上摘要和 ModelRegistry。       |
| PolicyService        | 管理用途限制、主体限制、次数限制和地理限制。   | 与 Catalog、Order 和 Task 统一授权模型。 |
| ContributionEngine   | 计算贡献分、奖励分配和惩罚。                   | 与 Reputation、RewardContract 打通。     |
| ModelRegistry        | 维护模型版本、摘要、水印和发布状态。           | 模型资产可作为新商品类型。               |

## 7.3 联邦学习任务流程

13. 任务发起方创建 TrainingTask，指定目标、模型类型、轮次数、参与条件、奖励规则和输出权限。

14. 潜在参与方基于策略模板与任务条件完成报名。平台校验组织资质、节点信誉、最低在线率和可用算力。

15. 任务启动后，各参与节点在本地数据上训练本地模型，只输出模型更新、梯度摘要或其他受控中间结果，不输出原始数据。

16. 节点将更新摘要、签名、指标、可选的零知识/TEE 证明提交给 AggregationService；链上记录 round_commit_hash、node_signature 和 timing。

17. 聚合后生成新的全局模型或任务结果摘要，写入 ModelRegistry 和 RewardContract；若结果允许发布，可形成新模型商品或报告商品。

18. 任务结束后计算贡献分，发放奖励并更新参与节点信誉；异常节点进入风控队列。

## 7.4 训练任务与策略模型

**TrainingTask 示例**

<table>
<colgroup>
<col style="width: 100%" />
</colgroup>
<thead>
<tr class="header">
<th><p>{</p>
<p>"task_id": "TASK-2026-0012",</p>
<p>"task_type": "federated_classification",</p>
<p>"initiator_org_id": "ORG-A",</p>
<p>"model_family": "cnn",</p>
<p>"rounds": 20,</p>
<p>"min_participants": 3,</p>
<p>"privacy_mode": "secure_aggregation",</p>
<p>"allowed_regions": ["CN", "SG"],</p>
<p>"allowed_output": ["global_model", "metrics_report"],</p>
<p>"reward_policy_id": "RWD-001",</p>
<p>"policy_template_id": "POL-TPL-002",</p>
<p>"status": "recruiting"</p>
<p>}</p></th>
</tr>
</thead>
<tbody>
</tbody>
</table>

策略模板必须与商品授权模型统一建模。推荐采用四层约束：主体约束（谁能参加）、用途约束（允许做什么）、时间约束（何时可用）和输出约束（能导出什么结果）。策略执行可以分为链上策略摘要和链下策略执行点：链上只记录策略版本和策略摘要哈希，链下网关、训练节点 SDK 和 API 网关共同执行策略。

## 7.5 贡献评估与激励机制

**贡献评分建议公式**

<table>
<colgroup>
<col style="width: 100%" />
</colgroup>
<thead>
<tr class="header">
<th><p>contribution_score =</p>
<p>0.40 * relative_metric_gain</p>
<p>+ 0.20 * data_quality_score</p>
<p>+ 0.15 * availability_score</p>
<p>+ 0.15 * timeliness_score</p>
<p>+ 0.10 * protocol_compliance_score</p>
<p>- anomaly_penalty</p>
<p>reward_amount = reward_pool * contribution_score / sum(all_scores)</p></th>
</tr>
</thead>
<tbody>
</tbody>
</table>

relative_metric_gain 可以基于模型精度、AUC、损失下降等任务特定指标；data_quality_score 可以结合样本覆盖、缺失率、分布平衡度和稳定性；availability_score 反映节点参与率和在线率；protocol_compliance_score 反映是否按协议提交更新、是否按时响应和是否未触发风控告警。

**激励与惩罚**

| **事件**           | **处理方式**            | **影响**                     |
|--------------------|-------------------------|------------------------------|
| 按时有效提交       | 增加贡献分和信誉分      | 提升后续招募概率和奖励份额。 |
| 多轮持续高质量贡献 | 额外稳定性奖励          | 可降低后续交易或任务保证金。 |
| 上传异常更新       | 本轮作废并扣信誉        | 严重时冻结节点资格。         |
| 中途频繁退出       | 减少 availability_score | 影响后续任务参与。           |
| 试图导出未授权结果 | 立即终止会话并触发审计  | 可触发组织级处罚。           |

## 7.6 隐私增强策略

**隐私技术选项**

| **技术**   | **适用场景**           | **优点**               | **代价/注意点**                |
|------------|------------------------|------------------------|--------------------------------|
| 安全聚合   | 多方联邦学习           | 实现简单、性能相对较好 | 需要处理掉线和节点同步。       |
| 同态加密   | 强敏感参数保护         | 隐私强                 | 算力和时延较高。               |
| 差分隐私   | 防止输出泄露个体信息   | 可控噪声预算           | 精度会有损失。                 |
| TEE        | 可信执行敏感计算       | 实现复杂度可控         | 需评估硬件可信根和供应链风险。 |
| 零知识证明 | 证明某操作已按协议执行 | 适合高可信场景         | 成本较高，Phase 2 可选。       |

## 7.7 模型资产化设计

Phase 2 不仅要把数据资产化，也要把模型资产化。系统应允许将训练结果注册为 ModelAsset，并可选择是否作为新的商品进行授权或售卖。模型资产必须包含模型摘要、训练任务来源、主要参数、适用范围、性能指标、版本信息、水印信息和授权策略。模型资产的授权与数据商品授权应采用统一策略框架。

## 7.8 Phase 2 关键合约

**新增合约**

| **合约**              | **职责**                                 | **主要字段**                                               |
|-----------------------|------------------------------------------|------------------------------------------------------------|
| TrainingTaskContract  | 记录任务创建、参与、轮次提交、结束状态。 | task_id、policy_hash、round_no、participant_list、status。 |
| RewardContract        | 记录奖励池、贡献分、发放结果。           | task_id、subject_id、score、amount、status。               |
| ModelRegistryContract | 记录模型版本、摘要、水印哈希、发布状态。 | model_id、model_hash、watermark_hash、task_id、status。    |

## 7.9 Phase 2 风险点与约束

- 联邦学习不是免费的隐私能力，必须为模型投毒、数据分布异构、长尾节点掉线和通信开销预留治理机制。

- 聚合器虽然可以是链下服务，但必须在链上记录每轮聚合摘要和参与节点签名，避免聚合过程不可审计。

- 训练输出是否可商用必须由策略模板定义，不能默认允许下载全局模型。

- 当任务涉及跨域数据或跨境协作时，应在策略模板中加入地域限制和输出限制。

## 7.10 Phase 2 验收标准

- 至少支持一种标准联邦学习任务类型并完成多节点训练闭环。

- 支持任务创建、节点招募、轮次提交、聚合、奖励发放、模型注册与审计回看。

- 支持一种隐私增强手段（例如安全聚合或差分隐私）落地。

- 支持模型商品或报告商品的上架授权，证明 Phase 2 能从数据交易扩展到计算结果交易。
