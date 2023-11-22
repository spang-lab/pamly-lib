#! /usr/bin/env bash

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "usage: version.sh <version>"
  echo "This will update the version for all packages"
  exit 1
fi

if command -v gsed >/dev/null 2>&1; then
    SED_COMMAND="gsed"
else
    SED_COMMAND="sed"
fi



VERSION="$(npm version --no-git-tag-version "$1")"
$SED_COMMAND -i "0,/^version/s/^version = .*/version = \"${VERSION#v}\"/" Cargo.toml

npm install  # update lockfile
git add -A
git commit --message "$VERSION"
git tag -a -m "New version: $VERSION" "$VERSION"




