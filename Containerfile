# Build stage ---------------

FROM docker.io/rust:1.85.0 AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
ENV SWAGGER_UI_OVERWRITE_FOLDER=/app/swagger-ui
RUN cargo build --release

# Runtime stage -------------

FROM registry.access.redhat.com/ubi9 AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/lacoctelera lacoctelera
COPY config config
COPY static static

EXPOSE 9090/tcp
ENTRYPOINT [ "./lacoctelera" ]
