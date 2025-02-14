FROM rust:1.84-bullseye AS build

WORKDIR /app
COPY Cargo.lock Cargo.toml ./
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && cargo build --release && rm -rf src

COPY migrations ./migrations
COPY .sqlx ./.sqlx
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bullseye-slim

WORKDIR /app

COPY --from=build /app/target/release/sagisawa /app/sagisawa
