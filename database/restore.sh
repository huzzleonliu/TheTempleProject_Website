#!/bin/bash
set -e

# 等待PostgreSQL服务启动
until pg_isready -U $POSTGRES_USER; do
  sleep 1
done

# 执行带修复参数的恢复命令
pg_restore -U $POSTGRES_USER -d $POSTGRES_DB --no-owner </restore/backup.dump
echo "Database restored with --no-owner fix"
