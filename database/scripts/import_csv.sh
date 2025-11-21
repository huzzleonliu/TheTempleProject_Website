#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
EXCHANGE_DIR="${REPO_ROOT}/database/import_exchange"
COMPOSE_CMD="${COMPOSE_CMD:-podman-compose}"

NODE_CSV="${EXCHANGE_DIR}/node.csv"
VISUAL_CSV="${EXCHANGE_DIR}/visual_assets.csv"

if [[ ! -f "${NODE_CSV}" || ! -f "${VISUAL_CSV}" ]]; then
  echo "[数据库导入] 缺少 CSV 文件。请先运行 scripts/refresh_resources.sh" >&2
  exit 1
fi

echo "[数据库导入] 正在导入 node.csv 与 visual_assets.csv ..."

${COMPOSE_CMD} up -d database >/dev/null

${COMPOSE_CMD} exec -T database bash -c '
set -euo pipefail
IMPORT_DIR="/import_exchange"
NODE_CSV="${IMPORT_DIR}/node.csv"
VISUAL_CSV="${IMPORT_DIR}/visual_assets.csv"

if [[ ! -f "${NODE_CSV}" || ! -f "${VISUAL_CSV}" ]]; then
  echo "[数据库导入] 容器内未找到 CSV 文件: ${IMPORT_DIR}" >&2
  exit 1
fi

psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" <<EOSQL
CREATE EXTENSION IF NOT EXISTS ltree;
CREATE TABLE IF NOT EXISTS directory_nodes (
    path ltree PRIMARY KEY,
    has_subnodes BOOLEAN NOT NULL,
    raw_path TEXT NOT NULL,
    raw_filename TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS file_nodes (
    file_path ltree PRIMARY KEY,
    raw_path TEXT NOT NULL,
    raw_filename TEXT NOT NULL
);
TRUNCATE directory_nodes;
TRUNCATE file_nodes;
EOSQL

psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -c "\copy directory_nodes(path,has_subnodes,raw_path,raw_filename) FROM '\''${NODE_CSV}'\'' WITH (FORMAT csv, HEADER true);"
psql -U "${POSTGRES_USER}" -d "${POSTGRES_DB}" -c "\copy file_nodes(file_path,raw_path,raw_filename) FROM '\''${VISUAL_CSV}'\'' WITH (FORMAT csv, HEADER true);"

echo "[数据库导入] 导入完成"
'

