# Android toolchain

What an Android build of Grindstone needs on this Mac, how to install it and how to connect the phone over USB.

The versions follow the Android project that Tauri CLI 2.12.1 generates: Gradle 9.6.1, Android Gradle Plugin 9.3.1, compile and target SDK 37, minimum SDK 24.

## What and why

| Tool | Why |
| --- | --- |
| JDK 21 | Gradle and the Android Gradle Plugin run on it. Plugin 9 needs Java 17 or later; 21 is the long-term release. The JDK 27 already installed is newer than this Gradle release can be counted on, and 11 is too old. |
| Android command-line tools | `sdkmanager`, to install the pieces below without Android Studio |
| `platform-tools` | `adb`, to reach the phone over USB and install the app |
| `platforms;android-37` | the Android API the app compiles against |
| NDK | compiles the Rust part of the app for the phone's processor |
| Rust Android targets | the four Android targets of the Rust compiler |
| build tools | fetched by Gradle on the first build, once the licences are accepted |

- **Phone:** Android 7.0 (API 24) or later.
- **Disk:** about 5 GB: the NDK is about 2 GB and the Gradle caches 1 to 2 GB.

## Install

### 1. JDK 21

```sh
brew install openjdk@21
```

### 2. Android command-line tools

```sh
brew install --cask android-commandlinetools
```

### 3. Environment

Append to `~/.zshrc`:

```sh
export JAVA_HOME="/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home"
export ANDROID_HOME="/opt/homebrew/share/android-commandlinetools"
export NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"
```

Then:

```sh
source ~/.zshrc
```

### 4. Android SDK pieces and licences

```sh
sdkmanager --sdk_root="$ANDROID_HOME" "platform-tools" "platforms;android-37" "ndk;27.2.12479018"
sdkmanager --sdk_root="$ANDROID_HOME" --licenses
```

`ndk;27.2.12479018` is NDK r27c, a long-term release. A newer NDK works too:

```sh
sdkmanager --sdk_root="$ANDROID_HOME" --list | grep "ndk;"
```

If you pick another one, change `NDK_HOME` in `~/.zshrc` to match.

### 5. Rust Android targets

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

## Check

| Command | Expected |
| --- | --- |
| `java -version` | `openjdk version "21…` |
| `echo $ANDROID_HOME` | `/opt/homebrew/share/android-commandlinetools` |
| `ls "$NDK_HOME"` | `source.properties`, `toolchains`, … |
| `adb version` | `Android Debug Bridge version …` |
| `rustup target list --installed \| grep android` | four lines |

## USB debugging

### On the phone, once

1. **Settings → About phone → Build number:** tap it seven times and enter the PIN if asked, until "You are now a developer". On Samsung, the path is **Settings → About phone → Software information → Build number**.
2. **Settings → System → Developer options → USB debugging:** on. On Samsung: **Settings → Developer options**.

### Each connection

1. Plug the phone into the Mac with a USB **data** cable.
2. Unlock the phone. In the "Allow USB debugging?" dialog, tick **Always allow from this computer**, then tap **Allow**.
3. On the Mac:

```sh
adb devices
```

Expected:

```
List of devices attached
R58N12345AB    device
```

### Troubleshooting

| `adb devices` shows | Do |
| --- | --- |
| no device | Try another cable, since some only charge, and another port. On the phone, set the USB mode to **File transfer**. Then `adb kill-server && adb start-server`. |
| `unauthorized` | Unlock the phone and accept the dialog. If it never shows: **Developer options → Revoke USB debugging authorisations**, then unplug and plug again. |
| `offline` | `adb kill-server && adb start-server`, then unplug and plug again. |

- **Xiaomi, Redmi, POCO:** also turn on **Developer options → Install via USB**, which needs a signed-in Mi account.
- **Play Protect** may warn about an app from an unknown developer when installing: **More details → Install anyway**.

### Without a cable (optional, Android 11 or later)

1. **Developer options → Wireless debugging:** on, then **Pair device with pairing code**.
2. On the Mac, with the address, ports and code the phone shows:

```sh
adb pair <phone-ip>:<pairing-port>
adb connect <phone-ip>:<port>
adb devices
```
