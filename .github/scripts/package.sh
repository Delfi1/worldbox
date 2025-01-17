#!/usr/bin/env bash
set -eu

# When run in a container, the ownership will be messed up, so mark the
# checkout dir as safe regardless of our env
git config --global --add safe.directory "$GITHUB_WORKSPACE"

release_name="$NAME-$TARGET"
release_zip="${release_name}.zip"
mkdir "$release_name"

if [[ "$TARGET" =~ windows ]]; then
    bin="$NAME.exe"
else
    bin="$NAME"
fi

cp "target/$TARGET/release/$bin" "$release_name/"
cp README.md "$release_name/"
zip -r "$release_zip" "$release_name"
zip -ur "$release_zip" "./assets"

rm -r "$release_name"

export TAG_NAME = cargo pkgid | cut -d "#" -f2

# Windows environments in github actions don't have the gnu coreutils installed,
# which includes the shasum exe, so we just use powershell instead
if [[ "$TARGET" =~ windows ]]; then
    echo "(Get-FileHash \"${release_name}\" -Algorithm SHA256).Hash | Out-File -Encoding ASCII -NoNewline \"${release_name}.sha256\"" | pwsh -c -
else
    echo -n "$(shasum -ba 256 "${release_name}" | cut -d " " -f 1)" > "${release_name}.sha256"
fi
