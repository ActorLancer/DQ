#!/bin/bash

################################################################################
# 快速测试入口脚本 (Quick Test Entry Point)
# 用法: ./quick-test.sh [选项]
################################################################################

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 测试脚本路径
API_TEST_SCRIPT="$SCRIPT_DIR/test-payment-api.sh"
DB_TEST_SCRIPT="$SCRIPT_DIR/test-payment-db.sh"

# ============================================================================
# 工具函数
# ============================================================================

log_title() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}!${NC} $1"
}

# ============================================================================
# 检查前置条件
# ============================================================================

check_prerequisites() {
    log_title "前置条件检查"

    local missing_tools=0

    # 检查必要工具
    for tool in curl jq psql; do
        if command -v "$tool" &> /dev/null; then
            log_success "$tool 已安装"
        else
            log_error "$tool 未安装"
            missing_tools=$((missing_tools + 1))
        fi
    done

    # 检查测试脚本存在
    if [ -f "$API_TEST_SCRIPT" ]; then
        log_success "API测试脚本存在"
    else
        log_error "API测试脚本不存在: $API_TEST_SCRIPT"
        missing_tools=$((missing_tools + 1))
    fi

    if [ -f "$DB_TEST_SCRIPT" ]; then
        log_success "数据库测试脚本存在"
    else
        log_error "数据库测试脚本不存在: $DB_TEST_SCRIPT"
        missing_tools=$((missing_tools + 1))
    fi

    if [ $missing_tools -gt 0 ]; then
        log_error "缺少 $missing_tools 个必要组件，请先安装"
        exit 1
    fi

    echo ""
    log_success "所有前置条件检查通过"
}

# ============================================================================
# 显示菜单
# ============================================================================

show_menu() {
    log_title "支付系统集成测试工具"

    echo ""
    echo "请选择测试类型:"
    echo ""
    echo "  1. API 集成测试 (完整流程)"
    echo "  2. 数据库验证 (交互式)"
    echo "  3. 快速API测试 (简化版)"
    echo "  4. 快速数据库查询"
    echo "  5. 全部测试 (API + DB)"
    echo "  6. 显示帮助文档"
    echo "  0. 退出"
    echo ""
}

# ============================================================================
# 测试执行函数
# ============================================================================

run_api_test() {
    log_title "执行 API 集成测试"

    if [ ! -x "$API_TEST_SCRIPT" ]; then
        chmod +x "$API_TEST_SCRIPT"
    fi

    bash "$API_TEST_SCRIPT"
    return $?
}

run_db_test() {
    log_title "执行数据库验证"

    if [ ! -x "$DB_TEST_SCRIPT" ]; then
        chmod +x "$DB_TEST_SCRIPT"
    fi

    bash "$DB_TEST_SCRIPT"
    return $?
}

run_quick_api_test() {
    log_title "快速 API 测试"

    local base_url="http://127.0.0.1:8080"

    log_info "测试服务健康检查..."
    if curl -s "$base_url/healthz" | jq . 2>/dev/null; then
        log_success "服务健康"
    else
        log_error "服务不可用"
        return 1
    fi

    echo ""
    log_info "创建支付意图..."

    local response=$(curl -s -X POST "$base_url/api/v1/payments/intents" \
        -H "Content-Type: application/json" \
        -H "x-role: tenant_admin" \
        -H "x-idempotency-key: quick-test-$(date +%s)" \
        -d '{
            "order_id": "30000000-0000-0000-0000-000000000101",
            "provider_key": "mock_payment",
            "payer_subject_type": "organization",
            "payer_subject_id": "30000000-0000-0000-0000-000000000102",
            "payee_subject_type": "organization",
            "payee_subject_id": "30000000-0000-0000-0000-000000000103",
            "amount": "10000",
            "payment_method": "wallet"
        }')

    if echo "$response" | jq . 2>/dev/null | grep -q "payment_intent_id"; then
        log_success "支付意图创建成功"
        echo ""
        echo "$response" | jq '.data | {payment_intent_id, status, amount}'
    else
        log_error "支付意图创建失败"
        echo "$response" | jq . 2>/dev/null || echo "$response"
        return 1
    fi
}

run_quick_db_query() {
    log_title "快速数据库查询"

    local db_host="${DB_HOST:-127.0.0.1}"
    local db_port="${DB_PORT:-55432}"
    local db_user="${DB_USER:-luna}"
    local db_pass="${DB_PASS:-5686}"
    local db_name="${DB_NAME:-luna_data_trading}"

    log_info "数据库: postgres://$db_user@$db_host:$db_port/$db_name"

    PGPASSWORD="$db_pass" psql -h "$db_host" -p "$db_port" -U "$db_user" -d "$db_name" << EOF
SELECT
    COUNT(*) as total_records,
    COUNT(DISTINCT status) as distinct_statuses
FROM payment.payment_intent;

SELECT status, COUNT(*) as count
FROM payment.payment_intent
GROUP BY status
ORDER BY count DESC;
EOF

    echo ""
    log_success "数据库查询完成"
}

run_all_tests() {
    log_title "执行完整测试套件"

    log_info "第 1 步: API 集成测试"
    run_api_test
    local api_result=$?

    echo ""
    read -p "是否继续执行数据库验证? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "第 2 步: 数据库验证"
        run_db_test
        local db_result=$?
    else
        local db_result=0
    fi

    echo ""
    log_title "测试套件总结"

    if [ $api_result -eq 0 ]; then
        log_success "API 测试通过"
    else
        log_error "API 测试失败"
    fi

    if [ $db_result -eq 0 ]; then
        log_success "数据库验证通过"
    fi
}

show_help() {
    cat << 'EOF'

╔════════════════════════════════════════════════════════════════════════╗
║                        快速测试指南 (Quick Guide)                       ║
╚════════════════════════════════════════════════════════════════════════╝

【API 测试】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  脚本: test-payment-api.sh

  功能:
    ✓ 健康检查
    ✓ 创建支付意图
    ✓ 幂等性验证
    ✓ 权限控制测试
    ✓ 数据库验证

  命令:
    ./quick-test.sh 1         # 交互式运行 API 测试
    bash test-payment-api.sh  # 直接运行 API 测试脚本

【数据库验证】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  脚本: test-payment-db.sh

  功能:
    ✓ 支付意图表验证
    ✓ 按幂等密钥查询
    ✓ 按支付意图ID查询
    ✓ 状态分布统计
    ✓ 数据一致性检查
    ✓ 审计日志查看

  命令:
    ./quick-test.sh 2                                    # 交互式运行
    bash test-payment-db.sh query-by-key idem-bil002-001  # 按密钥查询
    bash test-payment-db.sh query-by-id <payment_intent_id>  # 按ID查询
    bash test-payment-db.sh full                         # 完整验证

【快速命令】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  # 查看服务状态
  curl http://127.0.0.1:8080/healthz | jq .

  # 创建支付意图
  curl -X POST http://127.0.0.1:8080/api/v1/payments/intents \
    -H "Content-Type: application/json" \
    -H "x-role: tenant_admin" \
    -d '{
      "order_id": "30000000-0000-0000-0000-000000000101",
      "provider_key": "mock_payment",
      "payer_subject_type": "organization",
      "payer_subject_id": "30000000-0000-0000-0000-000000000102",
      "payee_subject_type": "organization",
      "payee_subject_id": "30000000-0000-0000-0000-000000000103",
      "amount": "10000",
      "payment_method": "wallet"
    }' | jq .

  # 查询支付意图
  curl http://127.0.0.1:8080/api/v1/payments/intents/<payment_intent_id> \
    -H "x-role: tenant_operator" | jq .

  # 查询数据库
  PGPASSWORD=5686 psql -h 127.0.0.1 -p 55432 -U luna -d luna_data_trading \
    -c "SELECT * FROM payment.payment_intent LIMIT 5;"

【环境变量】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  API 测试:
    BASE_URL              API 基础地址 (默认: http://127.0.0.1:8080)
    DB_HOST               数据库主机 (默认: 127.0.0.1)
    DB_PORT               数据库端口 (默认: 55432)
    DB_USER               数据库用户 (默认: luna)
    DB_PASSWORD           数据库密码 (默认: 5686)
    DB_NAME               数据库名 (默认: luna_data_trading)

【典型工作流】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  1. 启动服务
     cargo run -p platform-core-bin --release

  2. 运行 API 测试
     ./quick-test.sh 1

  3. 查询数据库
     ./quick-test.sh 4

  4. 全部验证
     ./quick-test.sh 5

【故障排查】
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  服务无法连接:
    ✓ 确认服务已启动 (检查端口 8080)
    ✓ 检查防火墙设置
    ✓ 查看服务日志

  数据库连接失败:
    ✓ 确认 PostgreSQL 运行中
    ✓ 验证连接参数 (主机、端口、用户、密码)
    ✓ 检查数据库是否存在

  API 返回 401 Unauthorized:
    ✓ 验证 x-role 请求头
    ✓ 检查权限配置
    ✓ 查看错误详情

EOF
}

# ============================================================================
# 主程序
# ============================================================================

main() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║       支付系统快速测试工具 (Payment System Quick Test)      ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    # 检查前置条件
    check_prerequisites

    # 如果有命令行参数，直接执行
    if [ $# -gt 0 ]; then
        case $1 in
            1|api)
                run_api_test
                exit $?
                ;;
            2|db)
                run_db_test
                exit $?
                ;;
            3|quick-api)
                run_quick_api_test
                exit $?
                ;;
            4|quick-db)
                run_quick_db_query
                exit $?
                ;;
            5|all)
                run_all_tests
                exit $?
                ;;
            6|help)
                show_help
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                show_menu
                exit 1
                ;;
        esac
    fi

    # 交互式菜单
    while true; do
        show_menu

        read -p "请选择 [0-6]: " choice

        case $choice in
            1)
                run_api_test
                ;;
            2)
                run_db_test
                ;;
            3)
                run_quick_api_test
                ;;
            4)
                run_quick_db_query
                ;;
            5)
                run_all_tests
                ;;
            6)
                show_help
                ;;
            0)
                log_info "退出"
                exit 0
                ;;
            *)
                log_error "无效选择"
                ;;
        esac

        echo ""
        read -p "按 Enter 继续..."
    done
}

# 运行主程序
main "$@"
