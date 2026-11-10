# Image de production (Coolify, ou tout hébergeur Docker).
FROM rust:1-slim-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
# cubiomes (C, MIT) : seed viewer
COPY vendor vendor
COPY src src
COPY locales locales
COPY docs docs
COPY legal legal
# logo intégré au captcha puzzle (include_bytes!)
COPY public/assets/icone.png public/assets/icone.png
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/misty-site /usr/local/bin/misty-site
COPY public public
ENV PORT=3000
# Cache des releases, historique du statut et copie des téléchargements :
# à monter sur un volume persistant (Coolify : Persistent Storage sur /app/data).
ENV MISTY_CACHE_DIR=/app/data
RUN mkdir -p /app/data
EXPOSE 3000
CMD ["misty-site"]
