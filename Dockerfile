# Build stage: a release binary. Migrations are compiled into it.
FROM rust:1.97-slim-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --locked

# Run stage: no shell, no package manager, not root.
FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /app/target/release/tomi-api /tomi-api
COPY library /library
ENV PORT=8080
ENV LIBRARY_DIR=/library
EXPOSE 8080
ENTRYPOINT ["/tomi-api"]
