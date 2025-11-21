# 基于 PostgreSQL 镜像

FROM docker.io/library/postgres:latest


ENV POSTGRES_DB=tp_db \
    POSTGRES_USER=huzz \
    POSTGRES_PASSWORD=liurui301

# 复制初始化脚本
# RUN mkdir -p /restore
# COPY ./backup.dump /restore/backup.dump

# 复制 restore.sh（如果还需要恢复 dump）
# COPY ./restore.sh /docker-entrypoint-initdb.d/
# RUN chmod +x /docker-entrypoint-initdb.d/restore.sh

# 数据初始内容在容器启动后通过脚本导入，无需在构建阶段复制 CSV

# 暴露 5432 端口
EXPOSE 5432

