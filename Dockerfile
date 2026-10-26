# Build the vanilla Vite + Tailwind UI separately from the Rust service.
FROM node:24-bookworm-slim AS ui-builder
WORKDIR /build/ui
COPY ui/package.json ui/package-lock.json ./
RUN npm ci --no-fund
COPY ui/ ./
RUN npm run build

FROM rust:1.99-bookworm AS rust-builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY prompts/ ./prompts/
RUN cargo build --release --locked

FROM node:24-bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates openssl tini \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 10001 alexia \
    && useradd --uid 10001 --gid alexia --create-home alexia
WORKDIR /app
COPY --from=rust-builder /build/target/release/AlexIA /usr/local/bin/AlexIA
COPY --from=ui-builder /build/ui/dist/ ./ui/dist/
COPY tools/alexa-verifier/verify.cjs ./tools/alexa-verifier/verify.cjs
COPY prompts/ ./prompts/
RUN mkdir -p target/antigravity-local && chown -R alexia:alexia /app
ENV BIND_ADDR=0.0.0.0:3000 UI_HOST=localhost UI_PORT=3001 UI_MODE=static
USER alexia
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=45s --retries=3 \
    CMD node -e "require('node:http').get('http://127.0.0.1:3000/health',r=>process.exit(r.statusCode===200?0:1)).on('error',()=>process.exit(1))"
STOPSIGNAL SIGTERM
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/AlexIA"]
