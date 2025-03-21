# Build stage ---------------

EXPOSE 9090/tcp
FROM docker.io/rust:1.85.0 AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
ENV SWAGGER_UI_OVERWRITE_FOLDER=/app/swagger-ui
RUN cargo build --release

# Runtime stage -------------

FROM docker.io/rust:1.85.0-slim-bookworm AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/lacoctelera lacoctelera
COPY config config
COPY static static
ENTRYPOINT [ "./lacoctelera" ]
