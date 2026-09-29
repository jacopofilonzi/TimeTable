# syntax=docker/dockerfile:1

# ─── 1. Frontend → /www ────────────────────────────────────────────────────────
FROM node:24-alpine AS frontend
WORKDIR /src
RUN corepack enable
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN --mount=type=cache,id=pnpm,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
COPY frontend/ ./
RUN pnpm build && mv dist /www

# ─── 2. Backend → /app ─────────────────────────────────────────────────────────
FROM rust:1-bookworm AS backend
WORKDIR /src
COPY backend/Cargo.toml backend/Cargo.lock ./
# Build dependencies alone first so they are cached until Cargo.lock changes.
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src
COPY backend/src ./src
RUN touch src/main.rs \
    && cargo build --release --locked \
    && mkdir /app && cp target/release/timetable /app/timetable \
    && mkdir /data

# ─── 3. Runtime ────────────────────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=frontend /www /www
COPY --from=backend /app /app
# Short link database directory, writable by the nonroot user (a named volume inherits this).
COPY --from=backend --chown=65532:65532 /data /data
ENV HOST=0.0.0.0 \
    PORT=8080 \
    STATIC_DIR=/www \
    SHORTLINK_DB=/data/shortlinks.db \
    RUST_LOG=info \
    LOG_FORMAT=compact
VOLUME /data
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["/app/timetable", "healthcheck"]
ENTRYPOINT ["/app/timetable"]
