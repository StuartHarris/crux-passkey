#!/opt/homebrew/bin/bash

set -e

# until https://github.com/killercup/cargo-edit/pull/870 is merged
CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git cargo fetch

mapfile -t all < <(fd Cargo.toml)
roots=()
for file in "${all[@]}"; do
  roots+=("$(
    cd "$(dirname "$file")"
    cargo metadata --format-version 1 | jq -r '.workspace_root'
  )")
done

readarray -t sorted < <(printf '%s\n' "${roots[@]}" | sort -u)

for dir in "${sorted[@]}"; do
  (
    echo "---  Updating dependencies in $dir"
    cd "$dir"
    # until https://github.com/killercup/cargo-edit/pull/870 is merged
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git cargo upgrade -i --verbose
    cargo update
  )
done
