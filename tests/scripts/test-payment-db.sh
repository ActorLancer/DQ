#!/bin/bash

################################################################################
# 支付系统数据库验证脚本
# 功能: 验证支付意图、订单锁定、账务事件等数据库数据完整性
################################################################################

set -e

# ============================================================================
# 配置部分
# ============================================================================

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-55432}"
DB_USER="${DB_USER:-luna}"
DB_PASS="${DB_PASS:-5686}"
DB_NAME="${DB_NAME:-luna_data_trading}"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# 统计
VERIFY_PASSED=0
VERIFY_FAILED=0

# ============================================================================
# 日志函数
# ============================================================================

log_title() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
}

log_section() {
    echo ""
    echo -e "${CYAN}>>> $1${NC}"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
    ((VERIFY_PASSED++))
}

log_error() {
    echo -e "${RED}✗${NC} $1"
    ((VERIFY_FAILED++))
}

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}!${NC} $1"
}

separator() {
    echo -e "${BLUE}───────────────────────────────────────────────────────────${NC}"
}

# ============================================================================
# 数据库操作函数
# ============================================================================

db_connect() {
    PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME"
}

db_query() {
    local sql="$1"
    PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -tAc "$sql"
}

db_query_formatted() {
    local sql="$1"
    PGPASSWORD="$DB_PASS" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" "$sql"
}

# ============================================================================
# 前置检查
# ============================================================================

check_db_connection() {
    log_title "数据库连接检查"

    log_info "连接信息: postgres://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"

    if db_query "SELECT 1" > /dev/null 2>&1; then
        log_success "数据库连接成功"
    else
        log_error "无法连接到数据库"
        exit 1
    fi

    separator
}

# ============================================================================
# 支付意图表验证
# ============================================================================

verify_payment_intent_table_schema() {
    log_section "验证支付意图表结构"

    local sql="SELECT column_name, data_type, is_nullable
               FROM information_schema.columns
               WHERE table_schema = 'payment' AND table_name = 'payment_intent'
               ORDER BY ordinal_position;"

    log_info "SQL: $sql"
    echo ""

    db_query_formatted "$sql"

    echo ""
    log_success "支付意图表结构验证完成"
    separator
}

verify_payment_intent_records() {
    log_section "验证支付意图记录"

    local count=$(db_query "SELECT COUNT(*) FROM payment.payment_intent;")

    log_info "总记录数: $count"

    if [ "$count" -gt 0 ]; then
        log_success "支付意图表有 $count 条记录"
    else
        log_warn "支付意图表为空"
    fi

    separator
}

verify_payment_intent_by_idempotency_key() {
    local idempotency_key="$1"

    log_section "按幂等密钥查询支付意图: $idempotency_key"

    local sql="SELECT
                payment_intent_id::text as payment_intent_id,
                order_id::text as order_id,
                intent_type,
                provider_key,
                status,
                amount::text as amount,
                currency_code,
                idempotency_key,
                request_id,
                created_at::text as created_at,
                updated_at::text as updated_at
              FROM payment.payment_intent
              WHERE idempotency_key = '$idempotency_key'
              ORDER BY created_at DESC
              LIMIT 5;"

    log_info "SQL: 按 idempotency_key='$idempotency_key' 查询"
    echo ""

    db_query_formatted "$sql"

    local count=$(db_query "SELECT COUNT(*) FROM payment.payment_intent WHERE idempotency_key = '$idempotency_key';")

    echo ""
    if [ "$count" -gt 0 ]; then
        log_success "找到 $count 条记录"
    else
        log_warn "未找到相关记录"
    fi

    separator
}

verify_payment_intent_by_id() {
    local payment_intent_id="$1"

    log_section "按ID查询支付意图: $payment_intent_id"

    local sql="SELECT
                payment_intent_id::text as payment_intent_id,
                order_id::text as order_id,
                intent_type,
                provider_key,
                payer_subject_type,
                payer_subject_id::text as payer_subject_id,
                payee_subject_type,
                payee_subject_id::text as payee_subject_id,
                status,
                amount::text as amount,
                currency_code,
                price_currency_code,
                idempotency_key,
                request_id,
                created_at::text as created_at,
                updated_at::text as updated_at
              FROM payment.payment_intent
              WHERE payment_intent_id = '$payment_intent_id'::uuid;"

    log_info "SQL: 按 payment_intent_id 查询"
    echo ""

    db_query_formatted "$sql"

    local exists=$(db_query "SELECT COUNT(*) FROM payment.payment_intent WHERE payment_intent_id = '$payment_intent_id'::uuid;")

    echo ""
    if [ "$exists" -gt 0 ]; then
        log_success "支付意图记录存在"
    else
        log_error "支付意图记录不存在"
    fi

    separator
}

verify_payment_intent_status_values() {
    log_section "支付意图状态分布"

    local sql="SELECT status, COUNT(*) as count
              FROM payment.payment_intent
              GROUP BY status
              ORDER BY count DESC;"

    log_info "SQL: 按状态统计支付意图"
    echo ""

    db_query_formatted "$sql"

    echo ""
    log_success "状态分布统计完成"
    separator
}

# ============================================================================
# 订单锁定验证
# ============================================================================

verify_order_lock_table() {
    log_section "验证订单锁定表"

    local exists=$(db_query "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_schema='trade' AND table_name='order_lock');")

    if [ "$exists" = "t" ]; then
        log_success "订单锁定表存在"

        local count=$(db_query "SELECT COUNT(*) FROM trade.order_lock;")
        log_info "锁定记录数: $count"
    else
        log_warn "订单锁定表不存在或未初始化"
    fi

    separator
}

# ============================================================================
# 账务事件验证
# ============================================================================

verify_billing_event_table() {
    log_section "验证账务事件表"

    local sql="SELECT event_type, COUNT(*) as count
              FROM billing.billing_event
              GROUP BY event_type
              ORDER BY count DESC
              LIMIT 10;"

    log_info "SQL: 按事件类型统计账务事件"
    echo ""

    db_query_formatted "$sql"

    echo ""
    log_success "账务事件统计完成"
    separator
}

# ============================================================================
# 数据一致性检查
# ============================================================================

verify_data_consistency() {
    log_section "数据一致性检查"

    # 检查支付意图关联的订单是否存在
    log_info "检查: 支付意图关联的订单是否存在"

    local orphaned_count=$(db_query "SELECT COUNT(*)
                                     FROM payment.payment_intent pi
                                     LEFT JOIN trade.order_main om ON pi.order_id::uuid = om.order_id
                                     WHERE om.order_id IS NULL;")

    if [ "$orphaned_count" -eq 0 ]; then
        log_success "所有支付意图都关联到有效的订单"
    else
        log_warn "发现 $orphaned_count 条孤立的支付意图"
    fi

    # 检查幂等密钥唯一性
    log_info "检查: 幂等密钥唯一性"

    local duplicate_count=$(db_query "SELECT COUNT(*)
                                      FROM (SELECT idempotency_key, COUNT(*) as cnt
                                            FROM payment.payment_intent
                                            WHERE idempotency_key IS NOT NULL
                                            GROUP BY idempotency_key
                                            HAVING COUNT(*) > 1) t;")

    if [ "$duplicate_count" -eq 0 ]; then
        log_success "幂等密钥唯一性检查通过"
    else
        log_warn "发现 $duplicate_count 个重复的幂等密钥"
    fi

    separator
}

# ============================================================================
# 索引检查
# ============================================================================

verify_database_indexes() {
    log_section "支付模块关键索引检查"

    local sql="SELECT schemaname, tablename, indexname
              FROM pg_indexes
              WHERE schemaname = 'payment'
              ORDER BY tablename, indexname;"

    log_info "SQL: 列出支付模块的所有索引"
    echo ""

    db_query_formatted "$sql"

    echo ""
    log_success "索引检查完成"
    separator
}

# ============================================================================
# 查询性能测试
# ============================================================================

verify_query_performance() {
    log_section "查询性能测试"

    # 按idempotency_key查询性能
    log_info "测试1: 按 idempotency_key 查询"

    local sql="EXPLAIN ANALYZE
              SELECT * FROM payment.payment_intent
              WHERE idempotency_key = 'idem-test-001'
              LIMIT 1;"

    db_query_formatted "$sql" | head -20

    echo ""
    log_info "测试2: 按 payment_intent_id 查询"

    local sql="EXPLAIN ANALYZE
              SELECT * FROM payment.payment_intent
              WHERE payment_intent_id = '4f4b3a2e-508b-4902-ba35-97aa905b3772'::uuid
              LIMIT 1;"

    db_query_formatted "$sql" | head -20

    echo ""
    log_success "查询性能测试完成"
    separator
}

# ============================================================================
# 审计日志检查
# ============================================================================

verify_audit_logs() {
    log_section "审计日志验证"

    local sql="SELECT COUNT(*) as audit_count
              FROM audit.audit_event
              WHERE object_type = 'payment.payment_intent';"

    local count=$(db_query "$sql")

    log_info "支付意图相关的审计日志: $count 条"

    if [ "$count" -gt 0 ]; then
        log_success "审计日志记录存在"

        local audit_sql="SELECT
                        event_id::text,
                        object_type,
                        operation,
                        created_at::text as created_at
                       FROM audit.audit_event
                       WHERE object_type = 'payment.payment_intent'
                       ORDER BY created_at DESC
                       LIMIT 5;"

        echo ""
        log_info "最近的审计日志:"
        echo ""
        db_query_formatted "$audit_sql"
    else
        log_warn "未找到审计日志"
    fi

    separator
}

# ============================================================================
# 数据库统计信息
# ============================================================================

show_database_stats() {
    log_section "数据库统计信息"

    log_info "支付模块表大小"
    echo ""

    local sql="SELECT
               schemaname,
               tablename,
               pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
              FROM pg_tables
              WHERE schemaname = 'payment'
              ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;"

    db_query_formatted "$sql"

    echo ""
    log_success "统计信息获取完成"
    separator
}

# ============================================================================
# 自定义查询
# ============================================================================

run_custom_query() {
    local query="$1"

    log_section "执行自定义查询"

    log_info "查询: $query"
    echo ""

    db_query_formatted "$query"

    separator
}

# ============================================================================
# 生成报告
# ============================================================================

print_summary() {
    log_title "验证总结"

    local total=$((VERIFY_PASSED + VERIFY_FAILED))

    echo ""
    echo "验证结果:"
    echo "  ✓ 通过: $VERIFY_PASSED"
    echo "  ✗ 失败: $VERIFY_FAILED"
    echo "  总计:  $total"
    echo ""

    if [ $VERIFY_FAILED -eq 0 ]; then
        echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
        echo -e "${GREEN}所有验证通过！${NC}"
        echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    else
        echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
        echo -e "${RED}有 $VERIFY_FAILED 项验证失败${NC}"
        echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
    fi
}

# ============================================================================
# 帮助文档
# ============================================================================

show_help() {
    cat << EOF
${BLUE}支付系统数据库验证脚本${NC}

用法: $0 [命令] [选项]

命令:
    full                    执行完整验证 (默认)
    schema                  验证表结构
    records                 验证记录数据
    query-by-key <KEY>      按幂等密钥查询
    query-by-id <ID>        按支付意图ID查询
    status                  查询状态分布
    consistency             检查数据一致性
    indexes                 检查数据库索引
    performance             性能测试
    audit                   查看审计日志
    stats                   数据库统计
    custom <SQL>            执行自定义SQL查询
    help                    显示此帮助

选项:
    --host <HOST>           数据库主机 (默认: 127.0.0.1)
    --port <PORT>           数据库端口 (默认: 55432)
    --user <USER>           数据库用户 (默认: luna)
    --password <PASS>       数据库密码 (默认: 5686)
    --db <NAME>             数据库名 (默认: luna_data_trading)

示例:
    # 执行完整验证
    $0 full

    # 按幂等密钥查询
    $0 query-by-key idem-bil002-001

    # 按ID查询
    $0 query-by-id 4f4b3a2e-508b-4902-ba35-97aa905b3772

    # 自定义查询
    $0 custom "SELECT * FROM payment.payment_intent LIMIT 10"

    # 指定数据库连接
    $0 --host myhost --port 5432 full

EOF
}

# ============================================================================
# 主程序
# ============================================================================

main() {
    echo ""
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         支付系统数据库验证工具 (Database Verify)           ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    local command="${1:-full}"

    # 处理数据库连接选项
    while [[ $# -gt 0 ]]; do
        case $1 in
            --host) DB_HOST="$2"; shift 2 ;;
            --port) DB_PORT="$2"; shift 2 ;;
            --user) DB_USER="$2"; shift 2 ;;
            --password) DB_PASS="$2"; shift 2 ;;
            --db) DB_NAME="$2"; shift 2 ;;
            help) show_help; exit 0 ;;
            *) shift ;;
        esac
    done

    # 检查数据库连接
    check_db_connection

    # 执行命令
    case "$command" in
        full)
            verify_payment_intent_table_schema
            verify_payment_intent_records
            verify_payment_intent_status_values
            verify_data_consistency
            verify_database_indexes
            verify_audit_logs
            show_database_stats
            print_summary
            ;;
        schema)
            verify_payment_intent_table_schema
            ;;
        records)
            verify_payment_intent_records
            ;;
        query-by-key)
            verify_payment_intent_by_idempotency_key "${2:-idem-bil002-001}"
            ;;
        query-by-id)
            verify_payment_intent_by_id "${2:-4f4b3a2e-508b-4902-ba35-97aa905b3772}"
            ;;
        status)
            verify_payment_intent_status_values
            ;;
        consistency)
            verify_data_consistency
            ;;
        indexes)
            verify_database_indexes
            ;;
        performance)
            verify_query_performance
            ;;
        audit)
            verify_audit_logs
            ;;
        stats)
            show_database_stats
            ;;
        custom)
            run_custom_query "$2"
            ;;
        help)
            show_help
            ;;
        *)
            echo -e "${RED}未知命令: $command${NC}"
            show_help
            exit 1
            ;;
    esac
}

# 运行主程序
main "$@"
