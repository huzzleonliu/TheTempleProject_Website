FROM docker.io/library/nginx:latest

RUN mkdir -p /run/nginx
RUN mkdir -p /resource

# 删除默认配置文件
RUN rm -rf /etc/nginx

# 将本地 nginx.conf 复制到容器配置目录
COPY nginx/ /etc/nginx
COPY ./resource/ /resource/

# 验证配置语法（可选，但推荐）
# RUN nginx -t

# 暴露 80 端口
EXPOSE 80

# 启动 Nginx 并保持前台运行（Alpine 镜像默认行为）
ENTRYPOINT ["nginx", "-g", "daemon off;"]
