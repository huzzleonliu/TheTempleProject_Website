#!/bin/bash
set -e

# 等待PostgreSQL服务启动
until pg_isready -U $POSTGRES_USER; do
  sleep 1
done

echo "开始导入 CSV 文件..."

# 遍历 dataFolder 下的所有 CSV 文件
for csv_file in /dataFolder/*.csv; do
    # 检查文件是否存在（如果没有 CSV 文件，通配符会返回字面量）
    if [ ! -f "$csv_file" ]; then
        continue
    fi

    # 从文件名提取表名（去掉路径和 .csv 扩展名）
    filename=$(basename "$csv_file")
    table_name="${filename%.csv}"
    
    echo "处理文件: $filename -> 表名: $table_name"
    
    # 特殊处理：如果是 directories.csv，使用 ltree 类型
    if [ "$table_name" = "directories" ]; then
        # 创建表（如果不存在）
        psql -U $POSTGRES_USER -d $POSTGRES_DB <<EOF
            CREATE EXTENSION IF NOT EXISTS ltree;
            CREATE TABLE IF NOT EXISTS directories (
                path ltree PRIMARY KEY,
                has_layout BOOLEAN NOT NULL,
                has_visual_assets BOOLEAN NOT NULL,
                has_text INTEGER NOT NULL,
                has_images INTEGER NOT NULL,
                has_subnodes BOOLEAN NOT NULL
            );
            TRUNCATE TABLE directories;
EOF
    else
        # 对于其他 CSV 文件，读取第一行创建表
        header=$(head -n 1 "$csv_file")
        # 将列名转换为 CREATE TABLE 语句格式（所有列都是 TEXT 类型）
        columns=$(echo "$header" | tr ',' '\n' | sed 's/^/"/;s/$/" TEXT/' | tr '\n' ',' | sed 's/,$//')
        
        # 创建表（如果不存在）
        psql -U $POSTGRES_USER -d $POSTGRES_DB <<EOF
            CREATE TABLE IF NOT EXISTS "$table_name" (
                $columns
            );
            TRUNCATE TABLE "$table_name";
EOF
    fi
    
    # 导入 CSV 数据
    psql -U $POSTGRES_USER -d $POSTGRES_DB -c "
        COPY \"$table_name\" FROM '$csv_file' WITH (FORMAT csv, HEADER true);
    " && echo "✓ 成功导入 $filename 到表 $table_name" || echo "✗ 导入 $filename 失败"
done

echo "CSV 文件导入完成"

