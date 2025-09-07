# Dockerfile
FROM rust:1.78-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

FROM gcr.io/distroless/cc-debian11
COPY --from=builder /app/target/release/payment-system /payment-system
COPY --from=builder /app/migrations /migrations
USER nonroot:nonroot
EXPOSE 3000
CMD ["/payment-system"]