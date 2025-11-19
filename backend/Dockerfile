# 构建阶段
FROM docker.io/library/rust:latest as builder

# 设置工作目录
WORKDIR /usr/src/app

# 首先复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建一个空的 main.rs 来构建依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY . .

# 重新构建项目
RUN cargo build --release

# 运行阶段
FROM docker.io/library/debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
# RUN useradd -m -u 1000 appuser

# 设置工作目录
WORKDIR /app

# 从构建阶段复制编译好的二进制文件
COPY --from=builder /usr/src/app/target/release/backend /app/backend

# 设置权限
# RUN chown -R appuser:appuser /app
RUN chmod +x /app/backend

# 切换到非 root 用户
# USER appuser

# 暴露端口
EXPOSE 80

# 运行应用
CMD ["./backend"]
