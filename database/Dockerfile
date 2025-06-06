# 基于 Nginx Alpine 镜像（轻量化）

FROM postgres:latest


ENV POSTGRES_DB=tp_db \
    POSTGRES_USER=huzz \
    POSTGRES_PASSWORD=liurui301

# # 复制初始化脚本
RUN mkdir -p /restore
COPY ./backup.dump /restore/backup.dump

COPY ./restore.sh /docker-entrypoint-initdb.d/
RUN chmod +x /docker-entrypoint-initdb.d/restore.sh
#
# # 复制自定义配置文件（需提前准备）
# COPY ./custom.conf /etc/postgresql/postgresql.conf
#
# # 设置数据卷（生产环境推荐）
VOLUME /var/lib/postgresql/data

# 暴露 80 和 443 端口
EXPOSE 5432

