# 基于 Nginx Alpine 镜像（轻量化）

FROM alpine:3.21.3

RUN apk add --no-cache curl build-base git openssl-dev pkgconfig libgcc

RUN apk add nginx
WORKDIR /etc/nginx
RUN mkdir -p /run/nginx  # 添加此命令到 Dockerfile

# 删除默认配置文件
RUN rm /etc/nginx/conf.d/default.conf

# 将本地 nginx.conf 复制到容器配置目录
COPY conf.d/nginx.conf /etc/nginx/conf.d/app.conf

# 验证配置语法（可选，但推荐）
# RUN nginx -t
# # 构建项目
# ENV PATH="/root/.cargo/bin:${PATH}"


# 暴露 80 和 443 端口
EXPOSE 80

# 启动 Nginx 并保持前台运行（Alpine 镜像默认行为）
ENTRYPOINT ["nginx", "-g", "daemon off;"]
