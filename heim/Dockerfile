FROM rust:1.90-slim-bookworm AS builder

ARG GRPC_HEALTH_PROBE_VERSION=v0.4.48

RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libssl-dev \
    pkg-config \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN GRPC_HEALTH_PROBE_VERSION=v0.4.37 && \
    curl -L https://github.com/grpc-ecosystem/grpc-health-probe/releases/download/${GRPC_HEALTH_PROBE_VERSION}/grpc_health_probe-linux-amd64 -o /bin/grpc-health-probe && \
    chmod +x /bin/grpc-health-probe

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY core ./core
# COPY skju_server ./skju_server
# COPY mpu6500 ./mpu6500
# COPY flux ./flux
COPY heim ./heim
COPY proto ./proto

RUN cargo build --release --bin skju_heim


FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /workspace/target/release/skju_heim /app/skju_heim
COPY --from=builder /bin/grpc-health-probe /bin/grpc-health-probe

EXPOSE 50051

CMD ["/app/skju_heim"]
