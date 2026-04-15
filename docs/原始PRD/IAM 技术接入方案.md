# IAM 技术接入方案

- 文档名称：IAM 技术接入方案
- 适用范围：基于区块链技术的数据交易平台 `V1 / V2 / V3`
- 文档定位：技术接入基线，不等于最终定版技术选型

---

## 1. 目标

本方案用于把现有需求文档中已经冻结的身份与访问要求，收敛成可实施的技术接入边界，重点回答：

1. 认证中心是否需要统一产品承载
2. 浏览器、API、企业 SSO、机器身份分别怎么接
3. IAM 产品和平台业务数据库如何分工
4. 哪些能力 `V1` 必须落地，哪些能力 `V2/V3` 再增强

本文件不强制把某个产品写成不可替代的唯一实现，但给出当前推荐方案与替换边界。

---

## 2. 当前冻结的需求边界

在现有 PRD 体系中，以下要求已经成立，不因具体技术产品变化而改变：

- 浏览器门户使用 `OIDC / OAuth 2.0 Authorization Code + PKCE + 服务端会话 + HttpOnly Cookie`
- API / SDK 使用短期 `Access Token` + 可轮换 `Refresh Token`
- 企业联邦 `V1` 支持 `OIDC`，`V2` 增量支持 `SAML 2.0 / SCIM`
- 机器身份采用 `Client Credentials` 或等价机器身份机制
- 高风险动作必须经过 `step-up`
- Fabric 身份与平台登录身份分层
- 最终放行链路必须经过：身份 -> 主体状态 -> 权限 -> 作用域 -> 合同/Usage Policy -> 风控 -> 高风险校验 -> 审计

结论：

- `IAM` 产品只负责认证、联邦、令牌与会话相关能力
- 业务授权、作用域、合同、Usage Policy、风控与审计主数据，仍由平台自身控制

---

## 3. 推荐接入策略

## 3.1 推荐方案

当前推荐采用：

- `V1`：`Keycloak + 平台业务数据库内授权与策略判断`
- `V2`：在 `V1` 基础上补齐 `SAML / SCIM` 与更强机器身份治理
- `V2/V3`：如策略复杂度明显上升，再引入独立 `PDP` 运行时，例如 `OPA`

解释：

- `Keycloak` 适合承担统一身份协议层：
  - `OIDC`
  - `OAuth 2.0`
  - `SAML`
  - 企业联邦接入
  - 客户端/服务账号
  - 基础 MFA
- 但不建议把细粒度业务授权全部塞进 `IAM` 产品本身

因此本平台的推荐边界是：

- `Keycloak`：认证中心、联邦入口、令牌签发器
- 平台数据库：角色、权限、作用域、Usage Policy、合同、风控、审计主数据
- 平台服务：最终放行判定与高风险编排

## 3.2 不推荐的做法

- 把 `Keycloak` 当成唯一业务授权源
- 把合同、用途限制、地域限制、导出限制直接硬塞进 `IAM` 产品配置
- 让 Fabric 证书直接替代普通平台登录
- 浏览器前端长期持有高价值 refresh token

---

## 4. 逻辑分层

## 4.1 IAM 产品负责的能力

建议由统一认证中心承担：

- 本地账号认证
- 企业 `OIDC` 联邦登录
- `V2` 的 `SAML` 企业联邦
- 客户端注册与服务账号
- `OAuth 2.0` 客户端能力
- 标准端点暴露
  - discovery
  - authorize
  - token
  - userinfo
  - jwks
  - introspection
  - logout
- 基础 MFA 能力
- 会话与登录事件基础能力

## 4.2 平台应用负责的能力

必须继续由平台控制：

- 主体状态判定
- 成员与租户关系
- 角色定义与权限点
- 作用域绑定
- 合同快照
- `Usage Policy`
- 风控阻断
- 高风险 `step-up`
- 审计留痕
- Fabric 身份绑定与证书治理编排

## 4.3 双系统数据边界

建议原则：

- `IAM` 不是业务主库
- `IAM` 不拥有订单、合同、支付、交付、证据链权威状态
- 平台自己的 `iam` / `authz` 结构化数据仍需保留

推荐主数据归属：

- 登录凭证、联邦连接、客户端定义：可在 `Keycloak`
- 主体、成员、角色、作用域、Usage Policy、审计：在平台数据库
- 用户与外部身份、客户端与业务主体的绑定：在平台数据库中持久化镜像

---

## 5. V1 推荐落地方式

## 5.1 浏览器门户

建议：

- 前端不直接保存长期高价值凭证
- 采用 `BFF / 服务端会话`
- 浏览器只持有短生命周期 `HttpOnly Cookie`

推荐流程：

```text
浏览器
  -> 跳转 Keycloak authorize
  -> OIDC Code + PKCE
  -> 平台 BFF 完成 code exchange
  -> BFF 建立本地 session
  -> 后续请求携带 session cookie
```

收益：

- 降低令牌暴露风险
- 更容易做 step-up、会话撤销、设备治理

## 5.2 API / SDK

建议：

- 用户应用：`Authorization Code + PKCE`
- 服务/机器：`Client Credentials`
- 平台网关对 access token 做校验
- 业务服务读取平台本地授权数据做二次放行

## 5.3 企业联邦

`V1` 只做：

- 企业 `OIDC` 连接
- claim mapping
- JIT 建档

规则：

- 首次登录仅允许建立待激活成员
- 不自动授予高权限角色
- 必须回到平台数据库侧完成角色与作用域绑定

## 5.4 机器身份

`V1` 就必须明确支持机器身份：

- `Application`
- `Connector`
- `Execution Environment`
- 平台内部服务

推荐实现：

- 以 confidential client / service account 承载
- 客户端与租户、应用、连接器对象建立绑定
- 机器 token 必须继续叠加：
  - 来源网络/IP
  - 客户端状态
  - 绑定范围
  - 配额与风控

---

## 6. V2 / V3 演进

## 6.1 V2

- `SAML 2.0`
- `SCIM`
- 更强的企业声明映射
- 更强的 JIT 建档策略
- 机器身份隔离增强
- 服务身份 / 执行环境身份增强

## 6.2 V3

- 更复杂的跨平台身份联邦
- 风险登录与 partner-scope step-up
- 更细的 assurance level 分级

---

## 7. 与现有数据库模型的映射

当前数据库设计已经具备承接能力，重点对象包括：

- `iam.auth_method_binding`
- `iam.mfa_authenticator`
- `iam.trusted_device`
- `iam.refresh_token_family`
- `iam.user_session`
- `iam.sso_connection`
- `iam.external_identity_binding`
- `iam.provisioning_job`
- `iam.step_up_challenge`

推荐补充的实现说明：

- 平台数据库保存 `Keycloak realm/client/user` 等外部标识的镜像引用
- 所有外部身份与服务账号必须能回溯到：
  - tenant
  - user/application/connector/environment
  - role/scope
  - 审计记录

---

## 8. 关键集成点

## 8.1 必须具备的标准能力

无论最终是否选 `Keycloak`，都应支持：

- OIDC discovery
- JWKS
- token introspection 或等价校验机制
- client registration / client management
- service account / client credentials
- session logout
- event / admin audit integration

## 8.2 平台必须保留的本地逻辑

平台后端在 token 校验通过后，仍必须继续执行：

1. 主体状态校验
2. 本地权限点判定
3. 本地作用域判定
4. 合同/Usage Policy 判定
5. 风控判定
6. 高风险 step-up
7. 审计写入

因此：

- `IAM` 通过 != 请求放行
- token 有效 != 有权限访问业务对象

---

## 9. 风险与常见误区

## 9.1 误区一：把 IAM 当业务授权主库

问题：

- 业务策略变化快
- 合同和 Usage Policy 本质是业务对象
- 直接把它们塞进 IAM 配置会让治理失控

正确做法：

- IAM 负责认证和协议
- 平台负责业务授权与策略

## 9.2 误区二：让 Fabric 证书直接做网页登录

问题：

- 生命周期不同
- 风险不同
- 用户体验差
- 容易把链上身份和平台会话绑死

正确做法：

- 平台身份与链上身份分层
- 通过绑定关系联动

## 9.3 误区三：把机器身份当成人的缩写版

问题：

- 应用、连接器、执行环境的生命周期与风险完全不同

正确做法：

- 机器身份单独建模
- 令牌、证书、网络边界和作用域单独治理

---

## 10. 当前结论

当前最合理的路线是：

- `V1`：采用统一 `IAM` 产品承载认证与联邦，当前首选可评估 `Keycloak`
- `V1`：业务授权继续在平台服务和数据库中完成
- `V2`：补齐 `SAML / SCIM` 与更完整的机器身份接入
- `V2/V3`：如策略复杂度明显提升，再把策略运行时外置

因此，这份方案冻结的是：

- 接入边界
- 数据边界
- 职责边界

不是把最终供应商或部署方式永久写死。

---

## 11. 参考资料

- Keycloak OIDC layers  
  https://www.keycloak.org/securing-apps/oidc-layers
- Keycloak Authorization Services  
  https://www.keycloak.org/docs/latest/authorization_services/

