#!/usr/bin/env bash
# Builds the Rust library for armeabi-v7a and arm64-v8a, then the APKs, and
# copies them to outputs/.
# Usage: scripts/build.sh [debug|release|all]   (default: all)
# Needs: ANDROID_HOME (SDK with platforms;android-35, build-tools;35.0.0 and
# ndk;27.2.12479018, or set ANDROID_NDK_HOME), a JDK 17 (JAVA_HOME or on PATH)
# and cargo with both Android Rust targets installed.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:-all}"

: "${ANDROID_HOME:?set ANDROID_HOME to your Android SDK}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/27.2.12479018}"
command -v cargo >/dev/null 2>&1 || { echo "cargo not found on PATH" >&2; exit 1; }

case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*) HOST=windows-x86_64; EXT=.cmd ;;
  Darwin) HOST=darwin-x86_64; EXT= ;;
  *) HOST=linux-x86_64; EXT= ;;
esac
BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/$HOST/bin"
# Rust target : Android ABI : NDK clang (API 23 = minSdk)
for spec in armv7-linux-androideabi:armeabi-v7a:armv7a-linux-androideabi23-clang \
            aarch64-linux-android:arm64-v8a:aarch64-linux-android23-clang; do
  IFS=: read -r TARGET ABI CC_NAME <<<"$spec"
  CLANG="$BIN/$CC_NAME$EXT"
  [ -f "$CLANG" ] || { echo "NDK clang not found: $CLANG" >&2; exit 1; }
  VAR="CARGO_TARGET_$(echo "$TARGET" | tr 'a-z-' 'A-Z_')_LINKER"
  export "$VAR=$CLANG"
  echo "== Rust ($TARGET -> $ABI, API 23)"
  (cd "$ROOT/rust" && cargo build --release --target "$TARGET" ${CARGO_FLAGS:-})
  mkdir -p "$ROOT/android/app/src/main/jniLibs/$ABI"
  cp "$ROOT/rust/target/$TARGET/release/libythome.so" "$ROOT/android/app/src/main/jniLibs/$ABI/"
done

echo "== Gradle"
cd "$ROOT/android"
printf 'sdk.dir=%s\n' "$(cd "$ANDROID_HOME" && pwd -W 2>/dev/null || pwd)" | sed 's#\\#/#g' > local.properties
TASKS=()
case "$MODE" in
  debug) TASKS=(assembleDebug) ;;
  release) TASKS=(assembleRelease) ;;
  all) TASKS=(assembleDebug assembleRelease lintRelease) ;;
  *) echo "unknown mode $MODE" >&2; exit 2 ;;
esac
./gradlew --no-daemon "${TASKS[@]}"

mkdir -p "$ROOT/outputs"
VER="$(sed -n "s/.*versionName '\(.*\)'.*/\1/p" app/build.gradle)"
# Gradle ABI splits: app-{universal,armeabi-v7a,arm64-v8a}-<type>[-unsigned].apk
for f in app/build/outputs/apk/*/app-*.apk; do
  [ -f "$f" ] || continue
  name="$(basename "$f" .apk)"
  name="${name#app-}"
  name="${name/armeabi-v7a/arm32}"
  name="${name/arm64-v8a/arm64}"
  cp "$f" "$ROOT/outputs/YTHome-$VER-$name.apk"
done
ls -l "$ROOT/outputs"
