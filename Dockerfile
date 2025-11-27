# syntax=docker/dockerfile:1

########### 1. 公共编译阶段：编译所有 Rust crate ###########
FROM rust:1.81 as builder

WORKDIR /app

# 先拷贝工作区定义，避免依赖缓存失效太频繁
COPY Cargo.toml Cargo.lock ./

# 拷贝各个 crate（按你现在的结构）
COPY bot-host bot-host
COPY api-server api-server
COPY core-types core-types
COPY storage storage
COPY plugin-api plugin-api
COPY workflow-core workflow-core
COPY plugins plugins
COPY workflows workflows
COPY config.toml .env ./

# 统一编译：host + api + 插件
RUN cargo build --release -p bot-host -p api-server -p cpu-monitor -p api-monitor

########### 2. bot-host 运行镜像 ###########
FROM debian:bookworm-slim as bot-host

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# 拷贝可执行文件
COPY --from=builder /app/target/release/bot-host /app/bot-host

# 拷贝配置（可以后续用 volume 覆盖）
COPY config.toml .env ./

# 拷贝插件到生产目录 plugins-bin
RUN mkdir -p /app/plugins-bin

# Linux 下 cdylib 名称是 libxxx.so
COPY --from=builder /app/target/release/libcpu_monitor.so /app/plugins-bin/libcpu_monitor.so
COPY --from=builder /app/target/release/libapi_monitor.so /app/plugins-bin/libapi_monitor.so

# SQLite 数据库目录
RUN mkdir -p /app/database

ENV RUST_LOG=info \
    MONITOR_AI_DB_URL=sqlite://database/monitor_ai.db \
    MONITOR_AI_PLUGIN_MODE=prod

VOLUME ["/app/database"]

CMD ["./bot-host"]

########### 3. api-server 运行镜像 ###########
FROM debian:bookworm-slim as api-server

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api-server /app/api-server

# api-server 只需要知道 DB 地址
ENV RUST_LOG=info \
    MONITOR_AI_DB_URL=sqlite://database/monitor_ai.db

# 数据库目录和 bot-host 共享 volume
RUN mkdir -p /app/database

VOLUME ["/app/database"]

EXPOSE 3001

CMD ["./api-server"]
