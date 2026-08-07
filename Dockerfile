# Builder stage
# Pinned to a specific toolchain for reproducible builds, and to the same
# Debian release (bookworm) as the runtime stage so the compiled binary's
# glibc is compatible with the runtime image.
FROM rust:1.90-bookworm AS builder
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release
# Runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/wizard_blog_backend  wizard_blog_backend
COPY configuration configuration
ENV APP_ENVIRONMENT=production
# The app listens on application_port (8000) from configuration/base.yaml.
EXPOSE 8000
ENTRYPOINT ["./wizard_blog_backend"]
