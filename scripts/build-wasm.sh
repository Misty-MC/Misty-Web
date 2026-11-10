#!/usr/bin/env bash
# Compile cubiomes (vendor/) en WebAssembly pour le seed viewer : public/wasm/cubiomes.wasm.
# Le fichier produit est versionné dans le dépôt : ce script ne sert que si vendor/ change.
#
# Il faut clang (15 ou plus), wasm-ld et la libc WASI. Sous Debian / Ubuntu :
#   apt-get install clang lld wasi-libc libclang-rt-18-dev-wasm32   (18 : la version de clang)
set -euo pipefail
cd "$(dirname "$0")/.."

CLANG="${CLANG:-clang}"
SYSROOT="${WASI_SYSROOT:-/usr}"
mkdir -p public/wasm

"$CLANG" --target=wasm32-wasi --sysroot="$SYSROOT" -mexec-model=reactor \
  -O3 -flto -fvisibility=hidden -w \
  -Ivendor \
  vendor/cubiomes/{biomenoise,biomes,generator,layers,noise,util,finders}.c vendor/misty_shim.c \
  -Wl,--gc-sections -Wl,--strip-all -Wl,--export-dynamic \
  -Wl,-z,stack-size=1048576 -Wl,--initial-memory=33554432 -Wl,--max-memory=536870912 \
  -o public/wasm/cubiomes.wasm

echo "public/wasm/cubiomes.wasm : $(wc -c < public/wasm/cubiomes.wasm) octets"
