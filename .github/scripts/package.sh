#!/usr/bin/env bash
set -eu

# When run in a container, the ownership will be messed up, so mark the
# checkout dir as safe regardless of our env
git config --global --add safe.directory "$GITHUB_WORKSPACE"

if [[ "$TARGET" =~ windows ]]; then
    release_archive="$NAME-$OS.zip"
    bin="$NAME.exe"
else
    release_archive="$NAME-$OS.tar.gz"
    bin="$NAME"
fi

tar -cf "$release_archive" -C "target/$TARGET/release/" "$bin"
tar -uf "$release_archive" "assets"

export TAG_NAME = cargo pkgid | cut -d "#" -f2

# Windows environments in github actions don't have the gnu coreutils installed,
# which includes the shasum exe, so we just use powershell instead
if [[ "$TARGET" =~ windows ]]; then
    echo "(Get-FileHash \"${release_archive}\" -Algorithm SHA256).Hash | Out-File -Encoding ASCII -NoNewline \"${release_archive}.sha256\"" | pwsh -c -
else
    echo -n "$(shasum -ba 256 "${release_archive}" | cut -d " " -f 1)" > "${release_archive}.sha256"
fi
