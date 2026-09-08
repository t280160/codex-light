#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
npm --prefix "$project_dir" run tauri build -- --bundles app,dmg

echo "Created macOS desktop bundles under $project_dir/src-tauri/target/release/bundle"

