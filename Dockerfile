FROM rust:1-bookworm AS builder

WORKDIR /app

# Cache dependencies with a stub binary before copying real source.
# Subsequent builds only recompile src/ — deps layer is reused.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release --locked
RUN rm -f target/release/server target/release/deps/server*

# Build the real binary
COPY src ./src
# COPY keeps the build context's mtimes, which are older than the stub build just
# above, so cargo would call the real sources fresh and ship the stub. Touch them.
RUN find src -name '*.rs' -exec touch {} +
COPY migrations ./migrations
COPY static ./static
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/server ./server
RUN test "$(stat -c %s /app/server)" -gt 1000000 || { echo "/app/server is stub-sized: the dependency-cache stub was shipped" >&2; exit 1; }
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/static ./static
COPY config.json ./

RUN useradd -r -u 1000 appuser
USER appuser

EXPOSE 3000
HEALTHCHECK --interval=10s --timeout=5s --start-period=15s --retries=3 \
    CMD ["/app/server", "healthcheck"]
ENTRYPOINT ["/app/server"]
