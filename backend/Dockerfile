# syntax=docker/dockerfile:1.7

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
RUN --mount=type=cache,id=backend-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=backend-cargo-git,target=/usr/local/cargo/git \
    cargo fetch

# SQLX 离线元数据与迁移（编译期宏可能需要），先拷贝提升缓存命中
COPY .sqlx ./.sqlx

# 再拷贝源码
COPY src ./src
COPY migrations ./migrations

# 启用 SQLX 离线编译（使用 .sqlx 元数据）
ENV SQLX_OFFLINE=true

# 构建 release 二进制
RUN --mount=type=cache,id=backend-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=backend-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=backend-target,target=/app/target \
    cargo build --release \
 && cp target/release/royale-arena-backend /app/royale-arena-backend

# ---------- Runtime stage ----------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates tzdata \
 && rm -rf /var/lib/apt/lists/*

# 非 root 运行
RUN useradd -m -u 1001 -s /usr/sbin/nologin appuser

# 使用 /srv/app 作为工作目录，确保存在并且属于 appuser
RUN mkdir -p /srv/app && chown appuser:appuser /srv/app
WORKDIR /srv/app

# 拷贝二进制到 /srv/app（以便 current_exe 的 parent 在 /srv/app）
COPY --from=builder /app/royale-arena-backend /srv/app/royale-arena-backend
RUN chown appuser:appuser /srv/app/royale-arena-backend

USER appuser

ENV RUST_LOG=info
EXPOSE 3000

CMD ["./royale-arena-backend"]