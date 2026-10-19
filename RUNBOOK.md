# Runbook

From a bare Mac to Grindstone installed on the phone, then each update.

- Commands run from `grindstone/` unless a step says otherwise.
- Steps 5 and 6 run once. Steps 1 to 10 first ran on 2026-10-03, with the walking skeleton.

## 1. Mac prerequisites

```sh
xcode-select --install
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
```

## 2. Android toolchain

What an Android build of Grindstone needs on this Mac, and how to install it. Connecting the phone comes at step 7.

The versions follow the Android project that Tauri CLI 2.12.1 generates: Gradle 9.6.1, Android Gradle Plugin 9.3.1, compile and target SDK 37, minimum SDK 24.

### What and why

| Tool | Why |
| --- | --- |
| JDK 21 | Gradle and the Android Gradle Plugin run on it. Plugin 9 needs Java 17 or later; 21 is the long-term release. The JDK 27 already installed is newer than this Gradle release can be counted on, and 11 is too old. |
| Android command-line tools | `sdkmanager`, to install the pieces below without Android Studio |
| `platform-tools` | `adb`, to reach the phone over USB and install the app |
| `platforms;android-37.0` | Android 17, the API the app compiles against. Since Android 16, platform packages carry a minor version, hence the `.0`. |
| NDK | compiles the Rust part of the app for the phone's processor |
| Rust Android targets | the four Android targets of the Rust compiler |
| build tools | fetched by Gradle on the first build, once the licences are accepted |

- **Phone:** Android 7.0 (API 24) or later.
- **Disk:** about 5 GB: the NDK is about 2 GB and the Gradle caches 1 to 2 GB.

### Install

#### 1. JDK 21

```sh
brew install openjdk@21
```

#### 2. Android command-line tools

```sh
brew install --cask android-commandlinetools
```

#### 3. Environment

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

#### 4. Android SDK pieces and licences

```sh
sdkmanager --sdk_root="$ANDROID_HOME" "platform-tools" "platforms;android-37.0" "ndk;27.2.12479018"
sdkmanager --sdk_root="$ANDROID_HOME" --licenses
```

- `sdkmanager` prints that it is deprecated in favour of `android sdk`. The warning is harmless: it still installs the packages.
- One unknown package name stops the whole install. To check the names:

```sh
sdkmanager --sdk_root="$ANDROID_HOME" --list | grep -E "platforms;android-37|ndk;27"
```

`ndk;27.2.12479018` is NDK r27c, a long-term release. A newer NDK works too:

```sh
sdkmanager --sdk_root="$ANDROID_HOME" --list | grep "ndk;"
```

If you pick another one, change `NDK_HOME` in `~/.zshrc` to match.

#### 5. Rust Android targets

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

### Check

| Command | Expected |
| --- | --- |
| `java -version` | `openjdk version "21…` |
| `echo $ANDROID_HOME` | `/opt/homebrew/share/android-commandlinetools` |
| `ls "$NDK_HOME"` | `source.properties`, `toolchains`, … |
| `adb version` | `Android Debug Bridge version …` |
| `rustup target list --installed \| grep android` | four lines |

## 3. Code

```sh
git clone git@github.com:Maeevick/grindstone.git
```

## 4. Check

```sh
make check
```

Expected: no formatting change, no lint, every test `ok`.

## 5. Android project, once

```sh
cargo tauri android init
```

The project comes with Tauri's template icon. Grindstone's own is in `src-tauri/icons/android`, made by `cargo tauri icon icon.svg`, with the adaptive foreground from `ic_launcher_foreground.svg` on the app's dark background:

```sh
res=src-tauri/gen/android/app/src/main/res
for density in mdpi hdpi xhdpi xxhdpi xxxhdpi; do cp src-tauri/icons/android/mipmap-$density/*.png $res/mipmap-$density/; done
mkdir -p $res/mipmap-anydpi-v26 && cp src-tauri/icons/android/mipmap-anydpi-v26/ic_launcher.xml $res/mipmap-anydpi-v26/
cp src-tauri/icons/android/values/ic_launcher_background.xml $res/values/
rm $res/drawable/ic_launcher_background.xml $res/drawable-v24/ic_launcher_foreground.xml
git add src-tauri/gen/android
git commit -m "build(android): generate the Android project"
```

## 6. Release signing, once

Create the keystore. Keep its password in your password manager and the file in your backups: an update must be signed with the same key, and changing the key means uninstalling, which erases the app's data.

```sh
keytool -genkey -v -keystore ~/.android/grindstone-release.jks -keyalg RSA -keysize 2048 -validity 10000 -alias grindstone
```

Write `src-tauri/gen/android/keystore.properties`, which git ignores. The password is asked without being shown:

```sh
printf 'Keystore password: ' && read -rs password && echo
printf 'password=%s\nkeyAlias=grindstone\nstoreFile=%s\n' "$password" "$HOME/.android/grindstone-release.jks" > src-tauri/gen/android/keystore.properties
unset password
```

Edit `src-tauri/gen/android/app/build.gradle.kts`. At the top of the file:

```kotlin
import java.io.FileInputStream
import java.util.Properties
```

Inside `android { }`, before `buildTypes`:

```kotlin
    signingConfigs {
        create("release") {
            val keystoreProperties = Properties()
            keystoreProperties.load(FileInputStream(rootProject.file("keystore.properties")))
            keyAlias = keystoreProperties["keyAlias"] as String
            keyPassword = keystoreProperties["password"] as String
            storeFile = file(keystoreProperties["storeFile"] as String)
            storePassword = keystoreProperties["password"] as String
        }
    }
```

Inside `buildTypes { getByName("release") { } }`:

```kotlin
            signingConfig = signingConfigs.getByName("release")
```

```sh
git add src-tauri/gen/android/app/build.gradle.kts
git commit -m "build(android): sign release builds with the personal keystore"
```

## 7. Phone connection

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

### USB troubleshooting

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

## 8. Build

`make android` runs steps 8 and 9 in one go.

```sh
cargo tauri android build --apk --target aarch64
```

Expected, at the end:

```
src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

Gradle prints warnings about the Android Gradle Plugin's deprecated DSL: they come from Tauri 2.12.1's template and are harmless.

## 9. Install

```sh
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
```

Expected: `Success`.

## 10. Smoke test, on the phone

1. Open **Grindstone**: the welcome shows until a profile exists, under the header and the icon bar.
2. **Profile** (the person at the top right): save a profile, goals, fuel targets, whether Hydrate counts in the streak, the usual day and the burn target. Each says from which day it is kept.
3. **Gather** (⚖️): log the day's mass and waist. The bands and the week's pace appear, the day joins the last days, and the ⚖️ tile turns to the mass.
4. **Fuel** (⚡): type a food the catalog lacks, fill "New …", then "Add and log". Put two foods on the plate: the band and the macros show its ghost. Log the plate.
5. **Hydrate** (💧): log a drink, a new one through "Add and log", then open the grimoire: the new drink is on the shelf.
6. **Burn** (🔥): log a workout, then put two exercises in a session, set its sets, and keep it as a circuit.
7. Close the app from the recent apps, then open it again: Home shows the streak and its chart, and today's tiles hold the day's figures.

## 11. The draft's records

The app must have run once, which creates its Documents folder. With the phone plugged in:

```sh
make import-draft-android BACKUP=../grindstone_draft/data/backups/<latest backup>.json
```

Then, in the app, open the Profile and scroll to its settings, then **Import the draft**. The app tells what it filed:
- measurements, profiles and goals;
- exercises and workouts;
- foods and intakes;
- drinks and drinkings.

The draft's coffees become Expresso drinkings of 80 mL each, and its teas 200 mL each. The import replaces those records. It can run again with a newer backup until the switch.

## Update

```sh
git pull
make check android
```

`-r` replaces the app and keeps its data, as long as the APK is signed with the same keystore.

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| `Keystore file … not found for signing config 'release'` | `storeFile` in `src-tauri/gen/android/keystore.properties` does not point at the keystore: write the file again as in step 6. |
| `INSTALL_PARSE_FAILED_NO_CERTIFICATES` | The APK is unsigned (`app-universal-release-unsigned.apk`): finish step 6. |
| `INSTALL_FAILED_UPDATE_INCOMPATIBLE` | The installed app was signed with another key. `adb uninstall com.maeevick.grindstone` erases its data; then install again. |
| `init` or `build` cannot find the SDK or NDK | Open a new terminal after editing `~/.zshrc`, then `echo $ANDROID_HOME $NDK_HOME`. |
| Gradle refuses the Java version | `java -version` must report 21: check `JAVA_HOME`. |
| The app opens on a blank screen | `cd ui && trunk build --release` must succeed, then build and install again. |
