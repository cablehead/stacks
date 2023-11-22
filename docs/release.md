## Todo

Update for `notarytool`:

https://github.com/tauri-apps/tauri/pull/7616/files

## Release

```
# make sure dev console is disabled

# update Cargo.toml and tauri.conf.json for new version
# also update README.md
# and the landing page
# export RELEASE to the new version (with the 'v'), e.g
RELEASE=v0.5.2

./scripts/build.sh
# while that builds
vi changes/$RELEASE.md
git add changes/$RELEASE.md

# after build completes
cat changes/$RELEASE.md | ./scripts/release.sh

# export the tempdir created by release.sh to RELEASE_PATH

# commit and push
git commit -a -m "chore: release $RELEASE" && git push

gh release create $RELEASE $RELEASE_PATH/* -n "$(cat changes/$RELEASE.md)"
```
