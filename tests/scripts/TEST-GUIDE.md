# 支付系统集成测试指南

## 📋 概述

本指南提供完整的支付系统测试脚本，包括 API 集成测试和数据库验证。

**提供的脚本:**
- `test-payment-api.sh` - API 集成测试 (支付意图生命周期)
- `test-payment-db.sh` - 数据库验证与查询工具
- `quick-test.sh` - 快速测试入口 (可交互式运行)

---

## 🚀 快速开始

### 方式 1: 交互式菜单

```bash
cd /home/luna/Documents/DataB
./quick-test.sh
```

选择：
- 1. API 集成测试
- 2. 数据库验证
- 3. 快速API测试
- 4. 快速数据库查询
- 5. 全部测试

### 方式 2: 直接运行命令

```bash
# API 集成测试
./test-payment-api.sh

# 数据库验证
./test-payment-db.sh full

# 按幂等密钥查询
./test-payment-db.sh query-by-key idem-bil002-001

# 按 ID 查询
./test-payment-db.sh query-by-id 4f4b3a2e-508b-4902-ba35-97aa905b3772

# 全部测试
./quick-test.sh 5
```

---

## 📝 脚本详情

### test-payment-api.sh

**测试内容:**
- 服务健康检查
- 创建支付意图 (POST)
- 幂等重放验证
- 权限控制 (tenant_admin vs tenant_operator)
- 数据库数据验证

**运行:**
```bash
./test-payment-api.sh
```

### test-payment-db.sh

**功能:**
1. 表结构验证
2. 按幂等密钥查询
3. 按支付意图 ID 查询
4. 按订单 ID 查询
5. 状态分布统计
6. 提供商分布统计
7. 最近交易记录
8. 幂等性验证
9. 数据完整性检查
10. 支付流程验证

**命令:**
```bash
./test-payment-db.sh query-by-key <key>
./test-payment-db.sh query-by-id <id>
./test-payment-db.sh status
./test-payment-db.sh consistency
./test-payment-db.sh custom "SQL"
```

---

## 🔧 环境变量

```bash
export BASE_URL="http://127.0.0.1:8080"
export DB_HOST="127.0.0.1"
export DB_PORT="55432"
export DB_USER="luna"
export DB_PASSWORD="5686"
export DB_NAME="luna_data_trading"
```

---

## 📊 测试数据

```
Order ID:        30000000-0000-0000-0000-000000000101
Payer ID:        30000000-0000-0000-0000-000000000102
Payee ID:        30000000-0000-0000-0000-000000000103
Amount:          10000 SGD
Provider:        mock_payment
```

---

## ✅ 检查清单

### API 测试
- [ ] 服务健康检查 (200)
- [ ] 支付意图创建成功
- [ ] 幂等重放返回相同 ID
- [ ] tenant_operator 可查询
- [ ] tenant_operator 被拒绝 (403)
- [ ] tenant_admin 可取消
- [ ] 数据库有记录

### 数据库验证
- [ ] payment_intent_id 有效
- [ ] status 值有效
- [ ] idempotency_key 唯一
- [ ] 必填字段完整
- [ ] 金额为正数
- [ ] 时间戳有效
- [ ] 审计日志存在

---

## 🔍 故障排查

### 服务无法连接
```bash
ps aux | grep platform-core
netstat -tlnp | grep 8080
cargo run -p platform-core-bin
```

### 数据库连接失败
```bash
psql -h 127.0.0.1 -p 55432 -U luna -d luna_data_trading -c "SELECT 1"
systemctl status postgresql
```

### API 返回 401
确保有 `x-role` 请求头:
```bash
curl -X POST ... -H "x-role: tenant_admin"
```

---

## 🎯 推荐工作流

```bash
# 1. 启动服务
cargo run -p platform-core-bin &

# 2. 等待 (5 秒)
sleep 5

# 3. 快速测试
./quick-test.sh 3

# 4. 查询数据库
./quick-test.sh 4

# 5. 完整验证
./quick-test.sh 5
```

---

**更新时间:** 2026-04-17
