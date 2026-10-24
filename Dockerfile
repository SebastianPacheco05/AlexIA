# Bootstrap image: generate Cargo.lock on first build. Commit Cargo.lock after
# local validation, then use cargo build --release --locked for reproducibility.
FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN groupadd --system gemilexa && useradd --system --gid gemilexa --no-create-home gemilexa
WORKDIR /app
COPY --from=builder /app/target/release/gemilexa /usr/local/bin/gemilexa
ENV BIND_ADDR=0.0.0.0:3000
USER gemilexa
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/gemilexa"]
