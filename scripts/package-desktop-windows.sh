#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
npm --prefix "$project_dir" run tauri build -- --target x86_64-pc-windows-gnu --bundles nsis

echo "Created Windows desktop bundles under $project_dir/src-tauri/target/x86_64-pc-windows-gnu/release/bundle"

