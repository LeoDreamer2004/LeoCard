#!/usr/bin/env bash
# Sourced by android.sh. All downloaded tools and emulator data stay under target/.
LEOCARD_ANDROID_CACHE="$LEOCARD_ROOT/target/android"
export ANDROID_HOME="${ANDROID_HOME:-$LEOCARD_ANDROID_CACHE/sdk}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_HOME/ndk/29.0.14206865}"
export GRADLE_USER_HOME="$LEOCARD_ANDROID_CACHE/gradle-home"
export ANDROID_USER_HOME="$LEOCARD_ANDROID_CACHE/user"
export ANDROID_AVD_HOME="$LEOCARD_ANDROID_CACHE/avd"
export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH"
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-21-openjdk}"
export PATH="$JAVA_HOME/bin:$PATH"
LEOCARD_GRADLE="$LEOCARD_ANDROID_CACHE/gradle-9.8.1/bin/gradle"
mkdir -p "$ANDROID_USER_HOME" "$ANDROID_AVD_HOME" "$LEOCARD_ANDROID_CACHE/downloads"

android_sdk() {
    if [[ ! -x "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" ]]; then
        curl -fL --retry 2 https://dl.google.com/android/repository/commandlinetools-linux-16111833_latest.zip \
            -o "$LEOCARD_ANDROID_CACHE/downloads/commandlinetools.zip"
        mkdir -p "$ANDROID_HOME/cmdline-tools"
        unzip -q "$LEOCARD_ANDROID_CACHE/downloads/commandlinetools.zip" -d "$ANDROID_HOME/cmdline-tools"
        mv "$ANDROID_HOME/cmdline-tools/cmdline-tools" "$ANDROID_HOME/cmdline-tools/latest"
    fi
}

android_tools() {
    android_sdk
    if [[ ! -x "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/clang" || ! -d "$ANDROID_HOME/platforms/android-36" || ! -x "$ANDROID_HOME/build-tools/36.0.0/apksigner" ]]; then
        sdkmanager --sdk_root="$ANDROID_HOME" 'platform-tools' 'platforms;android-36' 'build-tools;36.0.0' 'ndk;29.0.14206865'
    fi
    if [[ ! -x "$LEOCARD_GRADLE" ]]; then
        local archive="$LEOCARD_ANDROID_CACHE/downloads/gradle-9.8.1-bin.zip"
        curl -fL --retry 2 https://services.gradle.org/distributions/gradle-9.8.1-bin.zip -o "$archive"
        curl -fsL https://services.gradle.org/distributions/gradle-9.8.1-bin.zip.sha256 -o "$archive.sha256"
        (cd "$(dirname "$archive")"; printf '%s  %s\n' "$(cat "$archive.sha256")" "$(basename "$archive")" | sha256sum -c -)
        unzip -q "$archive" -d "$LEOCARD_ANDROID_CACHE"
    fi
    command -v cargo-ndk >/dev/null || cargo install cargo-ndk --locked
}

android_emulator() {
    if [[ ! -x "$ANDROID_HOME/emulator/emulator" || ! -d "$ANDROID_HOME/system-images/android-35/google_apis/x86_64" ]]; then
        sdkmanager --sdk_root="$ANDROID_HOME" 'emulator' 'system-images;android-35;google_apis;x86_64'
    fi
    if ! emulator -list-avds | grep -qx LeoCard; then
        printf 'no\n' | avdmanager create avd --name LeoCard --package 'system-images;android-35;google_apis;x86_64' --force
        cat >> "$ANDROID_AVD_HOME/LeoCard.avd/config.ini" <<'AVD'
hw.lcd.width=1280
hw.lcd.height=720
hw.lcd.density=160
hw.ramSize=2048
hw.gpu.enabled=yes
hw.gpu.mode=auto
hw.keyboard=yes
showDeviceFrame=no
AVD
    fi
    if ! adb devices | grep -q 'emulator-.*device$'; then
        nohup emulator -avd LeoCard -no-snapshot-load -no-boot-anim > "$LEOCARD_ANDROID_CACHE/emulator.log" 2>&1 &
    fi
    timeout 60 adb -e wait-for-device || {
        echo '模拟器连接超时，查看 target/android/emulator.log' >&2
        return 1
    }
    local booted=0
    for ((attempt=0; attempt<180; attempt++)); do
        if [[ "$(adb -e shell getprop sys.boot_completed 2>/dev/null | tr -d '\r')" == 1 ]]; then booted=1; break; fi
        sleep 1
    done
    [[ $booted == 1 ]] || { echo '模拟器启动超时，查看 target/android/emulator.log' >&2; return 1; }
}
