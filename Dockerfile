# Image de production (Coolify, ou tout hébergeur Docker).
FROM rust:1-slim-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY locales locales
COPY docs docs
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/misty-site /usr/local/bin/misty-site
COPY public public
ENV PORT=3000
EXPOSE 3000
CMD ["misty-site"]
