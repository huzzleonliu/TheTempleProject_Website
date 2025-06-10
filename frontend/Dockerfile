# Build stage
FROM rust:latest as builder

# Install Node.js and npm
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Install Trunk
RUN cargo install trunk

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

RUN rustup target add wasm32-unknown-unknown

# Build the frontend
RUN trunk build --release

# Production stage
FROM nginx:alpine

# 删除默认配置文件
# RUN rm /etc/nginx/nginx.conf
# RUN rm -rf /etc/nginx

# 将本地 nginx.conf 复制到容器配置目录
# COPY nginx/nginx.conf /etc/nginx/nginx.conf
# COPY nginx/ /etc/nginx/


# Copy built files from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]


