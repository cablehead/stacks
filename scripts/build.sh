
# https://tauri.app/v1/guides/distribution/sign-macos/
# https://github.com/mitchellh/gon

set -eu

ROOT="$(realpath "$(dirname "$0")")"

source "$ROOT"/../secrets.sh

rm -rf ./dist
npm run tauri build -- -v --target aarch64-apple-darwin -b dmg,app,updater 2>&1
npm run tauri build -- -v --target x86_64-apple-darwin -b dmg,app,updater 2>&1
