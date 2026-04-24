#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-datab}"
DB_USER="${DB_USER:-datab}"
DB_PASSWORD="${DB_PASSWORD:-datab_local_pass}"
OUT_FILE="${OUT_FILE:-docs/03-db/table-catalog.md}"

export PGPASSWORD="$DB_PASSWORD"
PSQL=(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -v ON_ERROR_STOP=1 -X -q -tA)

mkdir -p "$(dirname "$OUT_FILE")"

{
  echo "# V1 数据库表字典（自动导出）"
  echo
  echo "- 导出时间（UTC）：$(date -u '+%Y-%m-%d %H:%M:%S')"
  echo "- 数据库：\`${DB_NAME}\`"
  echo "- 来源：PostgreSQL 系统目录（\`pg_catalog\`）"
  echo '- 范围：`core/iam/authz/catalog/contract/trade/delivery/payment/billing/risk/audit/search/ops/support/developer/chain`'
  echo
} > "$OUT_FILE"

schemas=(
  core iam authz
  catalog contract trade delivery
  payment billing risk support
  audit search ops developer chain
)

responsibility_for_schema() {
  case "$1" in
    core) echo "身份主体、组织、账号、连接器与执行环境" ;;
    iam) echo "认证、会话、设备、证书、SSO 与身份凭证" ;;
    authz) echo "角色、权限、主体授权绑定" ;;
    catalog) echo "数据资产、版本、商品、SKU、标签与元信息" ;;
    contract) echo "模板、策略、数字合同与法律依据" ;;
    trade) echo "询单、订单、授权主链路" ;;
    delivery) echo "交付对象、票据、API、沙箱、报告、查询执行面" ;;
    payment) echo "支付渠道、意图、webhook、对账与提现" ;;
    billing) echo "计费规则、账务、结算、退款、赔付" ;;
    risk) echo "风险评级、风控事件、公平性事件" ;;
    support) echo "工单、争议、客服协同" ;;
    audit) echo "审计事件、证据、锚定、回放、保全" ;;
    search) echo "搜索投影、索引同步、排序配置" ;;
    ops) echo "outbox、DLQ、监控、告警、任务与系统日志" ;;
    developer) echo "开发者测试资产、模拟绑定、演示支付案例" ;;
    chain) echo "链上事件投影与锚定记录" ;;
    *) echo "-" ;;
  esac
}

for schema in "${schemas[@]}"; do
  count="$("${PSQL[@]}" -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='${schema}' AND table_type='BASE TABLE';")"
  if [[ "$count" == "0" ]]; then
    continue
  fi

  {
    echo "## Schema: \`${schema}\`"
    echo
    echo "- 对象职责：$(responsibility_for_schema "$schema")"
    echo "- 表数量：${count}"
    echo
    echo "| 表名 | 主键 | 唯一键 | 外键 | 索引 | 对象职责 |"
    echo "| --- | --- | --- | --- | --- | --- |"
  } >> "$OUT_FILE"

  "${PSQL[@]}" -c "
WITH tables AS (
  SELECT table_schema, table_name
  FROM information_schema.tables
  WHERE table_schema = '${schema}' AND table_type='BASE TABLE'
),
pk AS (
  SELECT tc.table_schema, tc.table_name, string_agg(kcu.column_name, ', ' ORDER BY kcu.ordinal_position) AS cols
  FROM information_schema.table_constraints tc
  JOIN information_schema.key_column_usage kcu
    ON tc.constraint_name = kcu.constraint_name
   AND tc.table_schema = kcu.table_schema
   AND tc.table_name = kcu.table_name
  WHERE tc.constraint_type = 'PRIMARY KEY'
  GROUP BY tc.table_schema, tc.table_name
),
uk AS (
  SELECT tc.table_schema, tc.table_name,
         string_agg(tc.constraint_name || '(' || cols.cols || ')', '; ' ORDER BY tc.constraint_name) AS uks
  FROM information_schema.table_constraints tc
  JOIN (
    SELECT tc2.table_schema, tc2.table_name, tc2.constraint_name,
           string_agg(kcu2.column_name, ', ' ORDER BY kcu2.ordinal_position) AS cols
    FROM information_schema.table_constraints tc2
    JOIN information_schema.key_column_usage kcu2
      ON tc2.constraint_name = kcu2.constraint_name
     AND tc2.table_schema = kcu2.table_schema
     AND tc2.table_name = kcu2.table_name
    WHERE tc2.constraint_type = 'UNIQUE'
    GROUP BY tc2.table_schema, tc2.table_name, tc2.constraint_name
  ) cols
    ON tc.table_schema = cols.table_schema
   AND tc.table_name = cols.table_name
   AND tc.constraint_name = cols.constraint_name
  WHERE tc.constraint_type = 'UNIQUE'
  GROUP BY tc.table_schema, tc.table_name
),
fk AS (
  SELECT tc.table_schema, tc.table_name,
         string_agg(kcu.column_name || '->' || ccu.table_schema || '.' || ccu.table_name || '(' || ccu.column_name || ')', '; ' ORDER BY tc.constraint_name, kcu.ordinal_position) AS fks
  FROM information_schema.table_constraints tc
  JOIN information_schema.key_column_usage kcu
    ON tc.constraint_name = kcu.constraint_name
   AND tc.table_schema = kcu.table_schema
   AND tc.table_name = kcu.table_name
  JOIN information_schema.constraint_column_usage ccu
    ON tc.constraint_name = ccu.constraint_name
   AND tc.table_schema = ccu.table_schema
  WHERE tc.constraint_type = 'FOREIGN KEY'
  GROUP BY tc.table_schema, tc.table_name
),
idx AS (
  SELECT schemaname AS table_schema, tablename AS table_name,
         string_agg(indexname, '; ' ORDER BY indexname) AS idxs
  FROM pg_indexes
  WHERE schemaname = '${schema}'
  GROUP BY schemaname, tablename
)
SELECT
  t.table_name || '|' ||
  COALESCE(pk.cols, '-') || '|' ||
  COALESCE(uk.uks, '-') || '|' ||
  COALESCE(fk.fks, '-') || '|' ||
  COALESCE(idx.idxs, '-')
FROM tables t
LEFT JOIN pk ON pk.table_schema = t.table_schema AND pk.table_name = t.table_name
LEFT JOIN uk ON uk.table_schema = t.table_schema AND uk.table_name = t.table_name
LEFT JOIN fk ON fk.table_schema = t.table_schema AND fk.table_name = t.table_name
LEFT JOIN idx ON idx.table_schema = t.table_schema AND idx.table_name = t.table_name
ORDER BY t.table_name;" | while IFS='|' read -r table_name pk_cols uk_cols fk_cols idx_cols; do
    echo "| \`${table_name}\` | \`${pk_cols}\` | \`${uk_cols}\` | \`${fk_cols}\` | \`${idx_cols}\` | $(responsibility_for_schema "$schema") |" >> "$OUT_FILE"
  done

  echo >> "$OUT_FILE"
done

echo "[ok] exported table catalog: ${OUT_FILE}"
