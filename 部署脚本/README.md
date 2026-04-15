# 部署脚本目录说明

## 职责

- 存放本地/联调环境 compose 文件、环境样例和部署辅助文档。

## 边界

- 本目录负责中间件与本地栈编排，不承载业务模块源码。

## 依赖

- 由根 `Makefile` 与 `scripts/` 调用。
- 历史文件 `部署脚本/docker-compose.local.yml` 仅保留兼容；主入口为 `infra/docker/docker-compose.local.yml`。

## 禁止事项

- 禁止在同一个 compose 文件中无限扩张职责。
- 禁止提交生产环境专用敏感配置。
