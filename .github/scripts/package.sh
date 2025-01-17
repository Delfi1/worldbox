#!/usr/bin/env bash
set -eu

# When run in a container, the ownership will be messed up, so mark the
# checkout dir as safe regardless of our env
git config --global --add safe.directory "$GITHUB_WORKSPACE"

release_zip="$NAME-$TARGET.zip"

if [[ "$TARGET" =~ windows ]]; then
    bin="$NAME.exe"
else
    bin="$NAME"
fi

zip -r "$release_zip" "target/$TARGET/release/$bin"
zip -ur "$release_zip" "./assets"

export TAG_NAME = cargo pkgid | cut -d "#" -f2

# Windows environments in github actions don't have the gnu coreutils installed,
# which includes the shasum exe, so we just use powershell instead
if [[ "$TARGET" =~ windows ]]; then
    echo "(Get-FileHash \"${release_zip}\" -Algorithm SHA256).Hash | Out-File -Encoding ASCII -NoNewline \"${release_zip}.sha256\"" | pwsh -c -
else
    echo -n "$(shasum -ba 256 "${release_zip}" | cut -d " " -f 1)" > "${release_zip}.sha256"
fi
