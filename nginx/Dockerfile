# 基于 Nginx Alpine 镜像（轻量化）
FROM nginx:latest

# 删除默认配置文件
RUN rm /etc/nginx/conf.d/default.conf

# 将本地 nginx.conf 复制到容器配置目录
COPY nginx.conf /etc/nginx/conf.d/app.conf

# 验证配置语法（可选，但推荐）
# RUN nginx -t

# 暴露 80 和 443 端口
EXPOSE 80

# 启动 Nginx 并保持前台运行（Alpine 镜像默认行为）
CMD ["nginx", "-g", "daemon off;"]
