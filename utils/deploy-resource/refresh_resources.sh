#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TOOL_DIR="${REPO_ROOT}/utils/node-generate-tool"
DEFAULT_RESOURCE_ROOT="${REPO_ROOT}/resource/resource"
RESOURCE_ROOT="${1:-${DEFAULT_RESOURCE_ROOT}}"
OUTPUT_DIR="${REPO_ROOT}/database/import_exchange"

echo "[资源扫描] 使用资源目录: ${RESOURCE_ROOT}"
if [[ ! -d "${RESOURCE_ROOT}" ]]; then
  echo "[资源扫描] 错误: 目录不存在 -> ${RESOURCE_ROOT}" >&2
  exit 1
fi

mkdir -p "${OUTPUT_DIR}"

pushd "${TOOL_DIR}" >/dev/null

echo "[资源扫描] 生成 node.csv ..."
cargo run --release -- scan node "${RESOURCE_ROOT}" -o "${OUTPUT_DIR}/node.csv"

echo "[资源扫描] 生成 visual_assets.csv ..."
cargo run --release -- scan visual "${RESOURCE_ROOT}" -o "${OUTPUT_DIR}/visual_assets.csv"

popd >/dev/null

echo "[资源扫描] CSV 已输出至 ${OUTPUT_DIR}"

COMPOSE_CMD="${COMPOSE_CMD:-podman-compose}"
echo "[资源扫描] 调用数据库导入脚本..."
"${REPO_ROOT}/database/scripts/import_csv.sh"
echo "[资源扫描] 资源刷新与数据库导入完成"

