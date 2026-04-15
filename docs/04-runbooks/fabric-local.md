# Fabric Local 启动说明（ENV-022/023/024）

## 命令入口

- `make fabric-up`
- `make fabric-down`
- `make fabric-reset`
- `make fabric-channel`

## 组件说明

- `fabric-ca`：本地 CA 容器占位
- `fabric-orderer`：本地 orderer 容器占位
- `fabric-peer`：本地 peer 容器占位

## 链码占位部署

```bash
./infra/fabric/deploy-chaincode-placeholder.sh
```

该脚本生成链码接口占位清单，覆盖：
- 订单摘要
- 授权摘要
- 验收摘要
- 证据批次根
