FROM alpine:3.21.3
WORKDIR /backend

# 安装基础依赖
RUN apk add --no-cache curl build-base git openssl-dev pkgconfig libgcc

# Rust 环境配置
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && source $HOME/.cargo/env 

# COPY . .

# 构建项目
ENV PATH="/root/.cargo/bin:${PATH}"

EXPOSE 8081

ENTRYPOINT ["cargo", "run"]
