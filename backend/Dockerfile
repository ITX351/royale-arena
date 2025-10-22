# ---------- Build stage ----------
FROM rust:1.90.0-slim-bookworm AS builder

# 常见原生依赖（有些 crates 需要 C 编译器/headers）
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 预拷贝清单以利用缓存
COPY Cargo.toml Cargo.lock ./

# 仅抓取依赖，避免源代码改动导致的全量重编
RUN cargo fetch

# SQLX 离线元数据与迁移（编译期宏可能需要），先拷贝提升缓存命中
COPY .sqlx ./.sqlx

# 再拷贝源码
COPY src ./src
COPY migrations ./migrations

# 启用 SQLX 离线编译（使用 .sqlx 元数据）
ENV SQLX_OFFLINE=true

# 构建 release 二进制
RUN cargo build --release

# ---------- Runtime stage ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tzdata \
 && rm -rf /var/lib/apt/lists/*

# 非 root 运行
RUN useradd -m -u 10001 -s /usr/sbin/nologin appuser

WORKDIR /srv/app

# 拷贝二进制
COPY --from=builder /app/target/release/royale-arena-backend /usr/local/bin/royale-arena-backend

USER appuser

ENV RUST_LOG=info
EXPOSE 8080

# 如果你的程序有子命令，请按需调整
CMD ["royale-arena-backend"]