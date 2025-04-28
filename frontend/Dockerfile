# Podmanfile
FROM alpine:3.21.3

# 安装基础依赖
RUN apk add --no-cache curl build-base git openssl-dev pkgconfig

# Rust 环境配置
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && source $HOME/.cargo/env \
    && rustup target add wasm32-unknown-unknown \

# 安装 Node.js 和 Podmanfile
RUN apk add nodejs npm 

# 项目空间
WORKDIR /frontend
COPY . .
VOLUME ["/app"]
EXPOSE 3000

# 安装前端依赖
RUN npm install 
RUN cargo install trunk

# # 启动脚本
# COPY docker-start.sh /frontend/docker-start.sh
# RUN chmod +x /frontend/docker-start.sh
#
# CMD ["./docker-start.sh"]
