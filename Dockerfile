# Image de production (Coolify, ou tout hébergeur Docker).
#
# Le build est découpé pour que le cache de Docker serve d'un déploiement à l'autre :
#   1. les dépendances (≈ 270 crates) ne sont recompilées que si Cargo.toml ou Cargo.lock changent ;
#   2. seul le code du site est recompilé à chaque déploiement ;
#   3. les fichiers statiques sont compressés une fois ici (gzip et brotli), pas à chaque requête.

# ---------- 1 et 2 : le serveur ----------
FROM rust:1-slim-bookworm AS build
WORKDIR /app
# Dépendances seules : un faux main.rs, puis cargo build. Cette couche reste en cache tant que
# Cargo.toml et Cargo.lock ne changent pas.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
 && cargo build --release --locked \
 && rm -rf src target/release/misty-site target/release/deps/misty_site-* target/release/.fingerprint/misty-site-*
# Le site
COPY src src
COPY locales locales
COPY docs docs
COPY legal legal
# logo intégré au captcha puzzle (include_bytes!)
COPY public/assets/icone.png public/assets/icone.png
RUN cargo build --release --locked

# ---------- 3 : fichiers statiques précompressés ----------
FROM debian:bookworm-slim AS assets
RUN apt-get update \
 && apt-get install -y --no-install-recommends brotli \
 && rm -rf /var/lib/apt/lists/*
COPY public /public
# Les formats texte (et le WebAssembly) ; les images et les zip sont déjà compressés.
RUN find /public -type f \( -name '*.css' -o -name '*.js' -o -name '*.wasm' -o -name '*.svg' \
      -o -name '*.json' -o -name '*.txt' -o -name '*.ttf' -o -name '*.ico' \) \
      -size +1k -exec gzip -9 -k {} \; -exec brotli -q 11 -k {} \;

# ---------- Image finale ----------
FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/target/release/misty-site /usr/local/bin/misty-site
COPY --from=assets /public public
ENV PORT=3000
# Cache des releases, historique du statut et copie des téléchargements :
# à monter sur un volume persistant (Coolify : Persistent Storage sur /app/data).
ENV MISTY_CACHE_DIR=/app/data
RUN mkdir -p /app/data
EXPOSE 3000
CMD ["misty-site"]
