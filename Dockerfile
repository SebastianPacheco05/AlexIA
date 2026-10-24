# Bootstrap image: generate Cargo.lock on first build. Commit Cargo.lock after
# local validation, then use cargo build --release --locked for reproducibility.
FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN groupadd --system AlexIA && useradd --system --gid AlexIA --no-create-home AlexIA
WORKDIR /app
COPY --from=builder /app/target/release/AlexIA /usr/local/bin/AlexIA
ENV BIND_ADDR=0.0.0.0:3000
USER AlexIA
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/AlexIA"]
