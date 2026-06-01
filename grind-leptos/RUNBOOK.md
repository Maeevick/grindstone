# Runbook

From a bare Mac to Grindstone installed on the phone, then each update.

- Commands run from `grind-leptos/` unless a step says otherwise.
- Steps 5 to 10 have not run on this Mac yet: they wait for the Android toolchain. Step 4 and the Mac app passed here.

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

[ANDROID_TOOLCHAIN.md](ANDROID_TOOLCHAIN.md), down to its checks, all green.

## 3. Code

```sh
git clone git@github.com:Maeevick/grindstone.git
cd grindstone/grind-leptos
```

## 4. Check

```sh
make check
```

Expected: no formatting change, no lint, every test `ok`.

## 5. Android project, once

```sh
cargo tauri android init
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

```sh
adb devices
```

Expected: one line ending in `device`. Otherwise see USB debugging in [ANDROID_TOOLCHAIN.md](ANDROID_TOOLCHAIN.md).

## 8. Build

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

1. Open **Grindstone**.
2. Log a weigh-in: it appears under **Last weigh-ins**.
3. Close the app from the recent apps, then open it again: the weigh-in is still there.

## Update

```sh
git pull
make check
cargo tauri android build --apk --target aarch64
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk
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

## Mac app (optional)

```sh
cargo tauri build --bundles app
cp -R target/release/bundle/macos/Grindstone.app /Applications/
```
