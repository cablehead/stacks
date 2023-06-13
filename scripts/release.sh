#!/usr/bin/env bash

set -eu

temp_dir=$(mktemp -d)

# Find dmg file
dmg_file=$(find ./src-tauri/target/release/bundle -name "*.dmg")

# Extract version from dmg file name
version=$(basename "$dmg_file" | cut -d'_' -f2)
arch=$(basename "$dmg_file" | cut -d'_' -f3 | cut -d'.' -f1)

# Copy dmg file to temporary directory
cp "$dmg_file" "$temp_dir"

# Copy tar.gz and sig files to temporary directory
cp "./src-tauri/target/release/bundle/macos/Stacks.app.tar.gz" "$temp_dir/Stacks_${version}_${arch}.app.tar.gz"
cp "./src-tauri/target/release/bundle/macos/Stacks.app.tar.gz.sig" "$temp_dir/Stacks_${version}_${arch}.app.tar.gz.sig"

# Get current date in required format
current_date=$(date -Iseconds)

# Read contents of .sig file
sig_file_contents=$(cat "$temp_dir/Stacks_${version}_${arch}.app.tar.gz.sig")

# Get stdin for notes
notes="$(jq -s -R)"

# Write JSON to .tauri-updater.json
cat > .tauri-updater.json << EOF
{
  "version": "$version",
  "notes": $notes,
  "pub_date": "$current_date",
  "platforms": {
    "darwin-aarch64": {
      "signature": "$sig_file_contents",
      "url": "https://github.com/cablehead/stacks/releases/download/v$version/Stacks_${version}_${arch}.app.tar.gz"
    }
  }
}
EOF

echo "Script execution finished successfully."
find $temp_dir

