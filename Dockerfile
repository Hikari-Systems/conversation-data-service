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
COPY migrations ./migrations
COPY static ./static
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/server ./server
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/static ./static
COPY config.json ./

RUN useradd -r -u 1000 appuser
USER appuser

EXPOSE 3000
ENTRYPOINT ["/app/server"]
