# IntDB Dockerfile
# 多阶段构建以减小镜像大小

# 构建阶段
FROM rust:1.75-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 创建工作目录
WORKDIR /usr/src/intdb

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建一个虚拟的src/lib.rs来预编译依赖
RUN mkdir src && echo "fn main() {}" > src/lib.rs
RUN cargo build --release && rm src/lib.rs

# 复制源代码
COPY src ./src
COPY examples ./examples

# 构建应用
RUN cargo build --release --examples

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false -d /opt/intdb intdb

# 创建目录
RUN mkdir -p /opt/intdb/{data,logs} && \
    chown -R intdb:intdb /opt/intdb

# 复制编译好的二进制文件
COPY --from=builder /usr/src/intdb/target/release/examples/api_server /usr/local/bin/intdb-server
COPY --from=builder /usr/src/intdb/target/release/examples/test_api_server /usr/local/bin/intdb-test

# 设置权限
RUN chmod +x /usr/local/bin/intdb-*

# 切换到非root用户
USER intdb

# 设置工作目录
WORKDIR /opt/intdb

# 暴露端口
EXPOSE 3000

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# 启动命令
CMD ["intdb-server"] 