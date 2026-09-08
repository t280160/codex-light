#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
version=$(awk -F '"' '/^version = / { print $2; exit }' "$project_dir/Cargo.toml")
output_dir="$project_dir/dist"
installer="$output_dir/CodexLight-$version-Windows-x64-Setup.exe"

mkdir -p "$output_dir"
cargo build \
    --manifest-path "$project_dir/Cargo.toml" \
    --release \
    --target x86_64-pc-windows-gnu

makensis \
    -DPROJECT_ROOT="$project_dir" \
    -DVERSION="$version" \
    -DOUTPUT_FILE="$installer" \
    "$project_dir/packaging/windows-installer.nsi"

echo "Created $installer"

