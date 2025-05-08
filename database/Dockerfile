# 基于 Nginx Alpine 镜像（轻量化）

FROM alpine:3.21.3

RUN apk add --no-cache curl build-base git openssl-dev pkgconfig libgcc

RUN apk add postgresql
# # 构建项目


# 暴露 80 和 443 端口
EXPOSE 5432

# 启动 Nginx 并保持前台运行（Alpine 镜像默认行为）
ENTRYPOINT ["nginx", "-g", "daemon off;"]
