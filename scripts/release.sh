#!/usr/bin/env bash

set -eu

temp_dir=$(mktemp -d)

# Find dmg files
dmg_files=$(find ./src-tauri/target/ -name "*.dmg")

# Initialize platforms JSON object
platforms_json=""

# Loop over each dmg file
for dmg_file in $dmg_files; do
    # Extract version and architecture from dmg file name
    version=$(basename "$dmg_file" | cut -d'_' -f2)
    arch=$(basename "$dmg_file" | cut -d'_' -f3 | cut -d'.' -f1)

    # Copy dmg file to temporary directory
    cp "$dmg_file" "$temp_dir"

    # Copy tar.gz and sig files to temporary directory
    cp "./src-tauri/target/${arch}-apple-darwin/release/bundle/macos/Stacks.app.tar.gz" "$temp_dir/Stacks_${version}_${arch}.app.tar.gz"
    cp "./src-tauri/target/${arch}-apple-darwin/release/bundle/macos/Stacks.app.tar.gz.sig" "$temp_dir/Stacks_${version}_${arch}.app.tar.gz.sig"

    # Read contents of .sig file
    sig_file_contents=$(cat "$temp_dir/Stacks_${version}_${arch}.app.tar.gz.sig")

    # Add to platforms JSON object
    platforms_json+="
    \"darwin-${arch}\": {
      \"signature\": \"$sig_file_contents\",
      \"url\": \"https://github.com/cablehead/stacks/releases/download/v$version/Stacks_${version}_${arch}.app.tar.gz\"
    },"
done

# Remove trailing comma
platforms_json=${platforms_json%?}

# Get current date in required format
current_date=$(date -Iseconds)

# Get stdin for notes
notes="$(jq -s -R)"

# Write JSON to .tauri-updater.json
cat >.tauri-updater.json <<EOF
{
  "version": "$version",
  "notes": $notes,
  "pub_date": "$current_date",
  "platforms": {
    $platforms_json
  }
}
EOF

echo "Script execution finished successfully."
find $temp_dir
