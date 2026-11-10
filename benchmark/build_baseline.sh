#!/usr/bin/env bash
# Builds the C++ implementation of the extension (the last commit before the Rust rewrite, by default) as a release
# loadable extension, to benchmark against. Needs CMake, Ninja, a C++ toolchain and vcpkg (VCPKG_ROOT, default ~/vcpkg).
#
# usage: benchmark/build_baseline.sh [git ref] [target directory]
set -euo pipefail

REF=${1:-b7b157f}
TARGET=${2:-$HOME/duckdb-avro-cpp}
VCPKG_ROOT=${VCPKG_ROOT:-$HOME/vcpkg}
REPO=$(cd "$(dirname "$0")/.." && pwd)

if [ ! -f "$VCPKG_ROOT/scripts/buildsystems/vcpkg.cmake" ]; then
    echo "vcpkg not found at $VCPKG_ROOT; set VCPKG_ROOT" >&2
    exit 1
fi

if [ ! -d "$TARGET" ]; then
    mkdir -p "$TARGET"
    git -C "$REPO" archive "$REF" | tar -x -C "$TARGET"
    # git archive leaves the submodules empty; check them out at the commits the ref pins
    git -C "$REPO" ls-tree -r "$REF" | awk '$2 == "commit" { print $3, $4 }' | while read -r commit path; do
        url=$(git -C "$REPO" show "$REF:.gitmodules" | git config -f /dev/stdin --get "submodule.$path.url")
        rm -rf "${TARGET:?}/$path"
        git clone -q "$url" "$TARGET/$path"
        git -C "$TARGET/$path" checkout -q "$commit"
    done
fi

cd "$TARGET"
VCPKG_TOOLCHAIN_PATH="$VCPKG_ROOT/scripts/buildsystems/vcpkg.cmake" GEN=ninja \
    CMAKE_BUILD_PARALLEL_LEVEL=${CMAKE_BUILD_PARALLEL_LEVEL:-$(sysctl -n hw.ncpu 2>/dev/null || nproc)} make release
echo "baseline: $TARGET/build/release/extension/avro/avro.duckdb_extension"
