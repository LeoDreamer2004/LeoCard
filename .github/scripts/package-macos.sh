#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "Usage: $0 <client-binary> <archive-name.zip>" >&2
  exit 1
fi

binary="$1"
archive="$2"
version="$(cargo pkgid -p leocard-client --locked)"
version="${version##*@}"
work_dir="$(mktemp -d "${TMPDIR:-/tmp}/leocard-macos.XXXXXX")"
trap 'rm -rf "$work_dir"' EXIT

bundle="$work_dir/LeoCard.app"
iconset="$work_dir/LeoCard.iconset"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources" "$iconset" dist
install -m755 "$binary" "$bundle/Contents/MacOS/leocard"

for size in 16 32 128 256 512; do
  sips -z "$size" "$size" assets/icons/app-icon.png \
    --out "$iconset/icon_${size}x${size}.png" > /dev/null
  retina_size=$((size * 2))
  sips -z "$retina_size" "$retina_size" assets/icons/app-icon.png \
    --out "$iconset/icon_${size}x${size}@2x.png" > /dev/null
done
iconutil -c icns "$iconset" -o "$bundle/Contents/Resources/LeoCard.icns"

python3 - "$bundle/Contents/Info.plist" "$version" "${MACOSX_DEPLOYMENT_TARGET:-11.0}" <<'PY'
import plistlib
import sys

path, version, minimum_system = sys.argv[1:]
with open(path, "wb") as output:
    plistlib.dump({
        "CFBundleDevelopmentRegion": "zh_CN",
        "CFBundleDisplayName": "LeoCard",
        "CFBundleExecutable": "leocard",
        "CFBundleIconFile": "LeoCard.icns",
        "CFBundleIdentifier": "com.leodreamer.leocard",
        "CFBundleInfoDictionaryVersion": "6.0",
        "CFBundleName": "LeoCard",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version,
        "CFBundleVersion": version.split("-")[0].split("+")[0],
        "LSMinimumSystemVersion": minimum_system,
        "NSHighResolutionCapable": True,
        "NSLocalNetworkUsageDescription": "用于搜索局域网房间、创建房间和与其他玩家联机。",
    }, output)
PY
plutil -lint "$bundle/Contents/Info.plist"

# Ad-hoc signing needs no Apple certificate; it does not provide Gatekeeper approval.
# Developer ID signing and notarization can replace this step when credentials exist.
codesign --force --sign - --timestamp=none "$bundle"
codesign --verify --strict --verbose=2 "$bundle"
ditto -c -k --keepParent "$bundle" "dist/$archive"
(
  cd dist
  shasum -a 256 "$archive" > "$archive.sha256"
)
