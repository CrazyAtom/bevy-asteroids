#!/usr/bin/env bash
# macOS .app 번들 조립 + zip. 사용법: scripts/package-macos.sh <version>
# 요구 도구: iconutil(내장), rsvg-convert(brew install librsvg).
set -euo pipefail

VERSION="${1:-0.0.0}"
BIN="target/release/bevy-asteroids"
OUT="dist-native"
APP="$OUT/BevyAsteroids.app"
RES="$APP/Contents/Resources"
MACOS="$APP/Contents/MacOS"

[ -f "$BIN" ] || { echo "바이너리 없음: $BIN (먼저 cargo build --release)"; exit 1; }
command -v rsvg-convert >/dev/null || { echo "rsvg-convert 필요: brew install librsvg"; exit 1; }

rm -rf "$OUT"
mkdir -p "$MACOS" "$RES/assets"

# 1) 바이너리
cp "$BIN" "$MACOS/bevy-asteroids"
chmod +x "$MACOS/bevy-asteroids"

# 2) 런타임 에셋(sprites/sounds/music) — SVG 원본(sprites/src)은 제외
cp -R assets/sprites "$RES/assets/sprites"
rm -rf "$RES/assets/sprites/src"
cp -R assets/sounds "$RES/assets/sounds"
cp -R assets/music "$RES/assets/music"

# 3) 아이콘: SVG → iconset PNG들 → .icns
ICONSET="$OUT/AppIcon.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  rsvg-convert -w "$s"   -h "$s"   assets/icon/app-icon.svg -o "$ICONSET/icon_${s}x${s}.png"
  d=$((s*2))
  rsvg-convert -w "$d"   -h "$d"   assets/icon/app-icon.svg -o "$ICONSET/icon_${s}x${s}@2x.png"
done
iconutil -c icns "$ICONSET" -o "$RES/AppIcon.icns"
rm -rf "$ICONSET"

# 4) Info.plist
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Bevy Asteroids</string>
  <key>CFBundleDisplayName</key><string>Bevy Asteroids</string>
  <key>CFBundleIdentifier</key><string>io.github.crazyatom.bevy-asteroids</string>
  <key>CFBundleExecutable</key><string>bevy-asteroids</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# 5) zip
( cd "$OUT" && ditto -c -k --keepParent "BevyAsteroids.app" "BevyAsteroids-macos-arm64.zip" )
echo "완료: $OUT/BevyAsteroids-macos-arm64.zip"
