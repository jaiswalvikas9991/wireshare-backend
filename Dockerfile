# ---------- build ----------
FROM rust:1.92 as builder
WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Build actual app
COPY src ./src
RUN cargo build --release

# ---------- runtime ----------
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/wireshare-backend /app/app

EXPOSE 8080
CMD ["/app/app"]

