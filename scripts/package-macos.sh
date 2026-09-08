#!/bin/sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
version=$(awk -F '"' '/^version = / { print $2; exit }' "$project_dir/Cargo.toml")
output_dir="$project_dir/dist"
output_file="$output_dir/CodexLight-$version-macOS-arm64.pkg"
stage_dir=$(mktemp -d /tmp/codex-light-macos.XXXXXX)

cleanup() {
    rm -rf "$stage_dir"
}
trap cleanup EXIT INT TERM

mkdir -p "$output_dir" "$stage_dir/usr/local/bin"
cargo build --manifest-path "$project_dir/Cargo.toml" --release --target aarch64-apple-darwin
install -m 0755 \
    "$project_dir/target/aarch64-apple-darwin/release/codex-light" \
    "$stage_dir/usr/local/bin/codex-light"
/usr/bin/xattr -cr "$stage_dir"
find "$stage_dir" -name '._*' -delete
rm -f "$output_file"

COPYFILE_DISABLE=1 pkgbuild \
    --root "$stage_dir" \
    --filter '(^|/)\._' \
    --identifier "com.codexlight.cli" \
    --version "$version" \
    --install-location / \
    "$output_file"

echo "Created $output_file"
